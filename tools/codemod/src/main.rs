//! Fase 1: inventaris blok `#[cfg(feature = "model")] impl <Type>` pada src/model.
//! Hanya membaca, tidak mengubah berkas.
use std::collections::BTreeMap;
use syn::{ImplItem, Item, Visibility};

fn is_model_cfg(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|a| {
        a.path().is_ident("cfg") && quote::quote!(#a).to_string().contains("\"model\"")
    })
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.get(1).map(String::as_str) == Some("apply") {
        let ops = std::path::Path::new(&args[2]);
        let model_root = std::path::Path::new(&args[3]);
        let mut used = std::collections::BTreeSet::new();
        for f in &args[4..] { let n = transform(std::path::Path::new(f), model_root, ops, &mut used); if !n.is_empty() { println!("{f}: {n:?}"); } }
        return;
    }
    let root = args.get(1).expect("usage: codemod <src/model> | apply <ops_dir> <files..>").clone();
    let mut total = (0usize, 0usize, 0usize, 0usize); // blok, pub, restricted, private
    let mut per_file: BTreeMap<String, (usize, usize, usize, usize)> = BTreeMap::new();
    let mut trait_impls = 0usize;
    for e in walkdir::WalkDir::new(&root).into_iter().filter_map(Result::ok) {
        if e.path().extension().is_none_or(|x| x != "rs") { continue; }
        let src = std::fs::read_to_string(e.path()).unwrap();
        let file = match syn::parse_file(&src) { Ok(f) => f, Err(err) => { eprintln!("PARSE FAIL {}: {err}", e.path().display()); continue; } };
        let mut stack: Vec<&Item> = file.items.iter().collect();
        while let Some(it) = stack.pop() {
            match it {
                Item::Mod(m) => if let Some((_, items)) = &m.content { stack.extend(items.iter()); },
                Item::Impl(i) if is_model_cfg(&i.attrs) => {
                    if i.trait_.is_some() { trait_impls += 1; continue; }
                    let c = per_file.entry(e.path().display().to_string()).or_default();
                    c.0 += 1; total.0 += 1;
                    for ii in &i.items {
                        if let ImplItem::Fn(f) = ii {
                            match &f.vis {
                                Visibility::Public(_) => { c.1 += 1; total.1 += 1 }
                                Visibility::Inherited => { c.3 += 1; total.3 += 1 }
                                _ => { c.2 += 1; total.2 += 1 }
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }
    println!("inherent cfg(model) impl blocks: {} | fn pub: {} | pub(crate)/restricted: {} | private: {} | trait impls skipped: {}", total.0, total.1, total.2, total.3, trait_impls);
    for (f, c) in &per_file { println!("{:60} blok={} pub={} restr={} priv={}", f.replace("crates/oxcord/src/model/", ""), c.0, c.1, c.2, c.3); }
}

// ---------------------------------------------------------------- transformasi
use proc_macro2::LineColumn;
use quote::ToTokens;
use syn::spanned::Spanned;

fn offset(src: &str, lc: LineColumn) -> usize {
    let mut off = 0;
    for (i, l) in src.split_inclusive('\n').enumerate() {
        if i + 1 == lc.line {
            return off + l.char_indices().nth(lc.column).map(|(b, _)| b).unwrap_or(l.len());
        }
        off += l.len();
    }
    src.len()
}

fn span_of<T: ToTokens>(src: &str, t: &T) -> (usize, usize) {
    let toks: Vec<_> = t.to_token_stream().into_iter().collect();
    (offset(src, toks.first().unwrap().span().start()), offset(src, toks.last().unwrap().span().end()))
}

fn strip_mut(sig: &mut syn::Signature) {
    for inp in sig.inputs.iter_mut() {
        if let syn::FnArg::Typed(pt) = inp {
            if let syn::Pat::Ident(pi) = &mut *pt.pat { pi.mutability = None; }
        }
    }
}

fn is_model_ok(attrs: &[syn::Attribute]) -> bool { !attrs.iter().any(|a| a.path().is_ident("cfg") && quote::quote!(#a).to_string().contains("test")) }

fn movable(f: &syn::ImplItemFn) -> bool {
    matches!(f.vis, Visibility::Public(_)) && f.sig.constness.is_none()
}

fn attr_is(a: &syn::Attribute, names: &[&str]) -> bool { names.iter().any(|n| a.path().is_ident(n)) }

/// Path modul induk (untuk menulis ulang `super::`) dari lokasi berkas.
fn parent_path(file: &std::path::Path, model_root: &std::path::Path) -> (String, String) {
    let rel = file.strip_prefix(model_root).unwrap();
    let mut comps: Vec<String> = rel.with_extension("").iter().map(|c| c.to_string_lossy().to_string()).collect();
    let is_mod = comps.last().map(|c| c == "mod").unwrap_or(false);
    if is_mod { comps.pop(); }
    let me = std::iter::once("crate::model".to_string()).chain(comps.iter().cloned()).collect::<Vec<_>>().join("::");
    let parent = if is_mod || comps.len() <= 1 {
        if comps.is_empty() { "crate".to_string() } else if is_mod { comps[..comps.len()-1].iter().fold("crate::model".to_string(), |a, c| format!("{a}::{c}")) } else { "crate::model".to_string() }
    } else { comps[..comps.len()-1].iter().fold("crate::model".to_string(), |a, c| format!("{a}::{c}")) };
    (me, parent)
}

pub fn transform(model_file: &std::path::Path, model_root: &std::path::Path, ops_dir: &std::path::Path, used: &mut std::collections::BTreeSet<String>) -> Vec<String> {
    let src = std::fs::read_to_string(model_file).unwrap();
    let file = syn::parse_file(&src).unwrap();
    let stem0 = model_file.file_stem().unwrap().to_string_lossy().to_string();
    let stem = if stem0 == "mod" { model_file.parent().unwrap().file_name().unwrap().to_string_lossy().to_string() + "_mod" } else { stem0 };
    let (me, parent) = parent_path(model_file, model_root);
    let mut cuts: Vec<(usize, usize)> = vec![];
    let mut ins: Vec<usize> = vec![];
    let mut helper_names: Vec<(String, Vec<syn::Attribute>)> = vec![];
    let mut traits: BTreeMap<String, (Vec<syn::TraitItem>, Vec<ImplItem>, syn::Type)> = BTreeMap::new();
    for it in &file.items {
        let Item::Impl(i) = it else { continue };
        if i.trait_.is_some() || !is_model_cfg(&i.attrs) || !i.generics.params.is_empty() { continue }
        let tname = match &*i.self_ty {
            syn::Type::Path(p) if p.qself.is_none() && p.path.segments.len() == 1 => p.path.segments[0].ident.to_string(),
            _ => continue,
        };
        let fns: Vec<&syn::ImplItemFn> = i.items.iter().filter_map(|x| if let ImplItem::Fn(f) = x { Some(f) } else { None }).collect();
        let all_fns = fns.len() == i.items.len();
        let impl_cfg: Vec<syn::Attribute> = i.attrs.iter().filter(|a| a.path().is_ident("cfg")).cloned().collect();
        let mv: Vec<&&syn::ImplItemFn> = fns.iter().filter(|f| movable(f)).collect();
        if mv.is_empty() { continue }
        let mut tn = format!("{tname}Ext");
        if used.contains(&tn) && !traits.contains_key(&tn) { tn = format!("{tname}{}Ext", stem.split('_').map(|w| { let mut c = w.chars(); c.next().map(|f| f.to_uppercase().collect::<String>() + c.as_str()).unwrap_or_default() }).collect::<String>()); }
        used.insert(tn.clone());
        let e = traits.entry(tn).or_insert_with(|| (vec![], vec![], (*i.self_ty).clone()));
        for f in &mv {
            let mut dsig = f.sig.clone(); strip_mut(&mut dsig);
            let dattrs: Vec<_> = impl_cfg.iter().chain(f.attrs.iter().filter(|a| !attr_is(a, &["inline"]))).cloned().collect();
            e.0.push(syn::TraitItem::Fn(syn::TraitItemFn { attrs: dattrs, sig: dsig, default: None, semi_token: Some(Default::default()) }));
            let mut m = (***f).clone();
            m.vis = Visibility::Inherited;
            m.attrs.retain(|a| !attr_is(a, &["must_use", "deprecated", "doc"]));
            m.attrs = impl_cfg.iter().cloned().chain(m.attrs.drain(..)).collect();
            e.1.push(ImplItem::Fn(m));
        }
        for f in &fns { if !movable(f) && matches!(f.vis, Visibility::Inherited) { ins.push(span_of(&src, &f.sig).0); } }
        if all_fns && mv.len() == fns.len() { cuts.push(span_of(&src, i)); }
        else { for f in &mv { cuts.push(span_of(&src, **f)); } }
    }
    if traits.is_empty() { return vec![] }
    for it in &file.items {
        if let Item::Fn(f) = it {
            if matches!(f.vis, Visibility::Inherited) && is_model_ok(&f.attrs) { ins.push(span_of(&src, &f.sig).0); helper_names.push((f.sig.ident.to_string(), f.attrs.iter().filter(|a| a.path().is_ident("cfg")).cloned().collect())); }
        }
    }
    let mut new_src = src.clone();
    let mut edits: Vec<(usize, usize, String)> = cuts.into_iter().map(|(s, e)| (s, e, String::new())).collect();
    edits.extend(ins.into_iter().map(|p| (p, p, "pub(crate) ".to_string())));
    edits.sort();
    for (s, e, r) in edits.into_iter().rev() {
        let mut e2 = e;
        if r.is_empty() { while new_src[e2..].starts_with('\n') && new_src[..s].ends_with("\n\n") { e2 += 1; } }
        new_src.replace_range(s..e2, &r);
    }
    std::fs::write(model_file, new_src).unwrap();
    let mut out = proc_macro2::TokenStream::new();
    for it in &file.items {
        let Item::Use(u) = it else { continue };
        if !matches!(u.vis, Visibility::Inherited) { continue }
        let mut t = u.to_token_stream().to_string();
        t = t.replacen("use super ::", &format!("use {parent} ::"), 1).replacen("use self ::", &format!("use {me} ::"), 1);
        if let Ok(Item::Use(u2)) = syn::parse_str::<Item>(&t) { u2.to_tokens(&mut out); }
    }
    out.extend(quote::quote! { use crate::model::prelude::*; });
    if !helper_names.is_empty() {
        for (n, attrs) in &helper_names {
            let t = format!("use {me}::{n};");
            if let Ok(Item::Use(mut u2)) = syn::parse_str::<Item>(&t).map(|x| x) { u2.attrs = attrs.clone(); u2.to_tokens(&mut out); }
        }
    }
    let names: Vec<String> = traits.keys().cloned().collect();
    for (name, (decls, impls, ty)) in traits {
        let id = syn::Ident::new(&name, proc_macro2::Span::call_site());
        out.extend(quote::quote! {
            #[allow(async_fn_in_trait)]
            pub trait #id: Sized { #(#decls)* }
            impl #id for #ty { #(#impls)* }
        });
    }
    let parsed = syn::parse2::<syn::File>(out).unwrap();
    std::fs::create_dir_all(ops_dir).unwrap();
    std::fs::write(ops_dir.join(format!("{stem}.rs")), prettyplease::unparse(&parsed)).unwrap();
    let modrs = ops_dir.join("mod.rs");
    let mut m = std::fs::read_to_string(&modrs).unwrap_or_default();
    let line = format!("mod {stem};\npub use {stem}::*;\n");
    if !m.contains(&line) { m.push_str(&line); }
    std::fs::write(modrs, m).unwrap();
    names
}
