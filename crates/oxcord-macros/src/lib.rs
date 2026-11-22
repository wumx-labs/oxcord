//! Proc-macro helpers for `oxcord-framework`.
//!
//! # `#[command]`
//!
//! Annotate an `async fn` with `#[oxcord_macros::command(...)]` to generate
//! the boilerplate that wraps it into an [`oxcord_framework::Command`].
//!
//! ## Supported attributes
//!
//! | Attribute | Type | Description |
//! |---|---|---|
//! | `slash_command` | flag | Register as a slash command |
//! | `prefix_command` | flag | Register as a prefix command |
//! | `name = "..."` | string | Override the command name (default: fn name) |
//! | `description = "..."` | string | One-line description for Discord |
//! | `aliases(a, b)` | list | Prefix aliases |
//! | `guild_only` | flag | Restrict to guilds |
//! | `owner_only` | flag | Restrict to owners |
//! | `category = "..."` | string | Group in the help page |
//!
//! ## Example
//!
//! ```rust,ignore
//! #[oxcord_macros::command(slash_command, prefix_command, description = "Ping the bot")]
//! async fn ping(ctx: oxcord_framework::Context<'_>) -> oxcord_framework::Result<()> {
//!     ctx.say("Pong!").await
//! }
//! ```

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::parse::{Parse, ParseStream, Result as SynResult};
use syn::punctuated::Punctuated;
use syn::{
    Expr,
    ExprLit,
    Ident,
    ItemFn,
    Lit,
    LitStr,
    Meta,
    MetaList,
    MetaNameValue,
    Token,
    parse_macro_input,
};

// ---------------------------------------------------------------------------
// Attribute parsing
// ---------------------------------------------------------------------------

#[derive(Default)]
struct CommandAttrs {
    slash_command: bool,
    prefix_command: bool,
    name: Option<String>,
    description: Option<String>,
    aliases: Vec<String>,
    guild_only: bool,
    owner_only: bool,
    category: Option<String>,
}

impl Parse for CommandAttrs {
    fn parse(input: ParseStream<'_>) -> SynResult<Self> {
        let mut attrs = CommandAttrs::default();
        let metas = Punctuated::<Meta, Token![,]>::parse_terminated(input)?;
        for meta in metas {
            match &meta {
                Meta::Path(path) => {
                    let ident = path.get_ident().map(|i| i.to_string()).unwrap_or_default();
                    match ident.as_str() {
                        "slash_command" => attrs.slash_command = true,
                        "prefix_command" => attrs.prefix_command = true,
                        "guild_only" => attrs.guild_only = true,
                        "owner_only" => attrs.owner_only = true,
                        other => {
                            return Err(syn::Error::new(
                                path.get_ident().map_or(Span::call_site(), |i| i.span()),
                                format!("unknown flag attribute: `{other}`"),
                            ));
                        },
                    }
                },
                Meta::NameValue(MetaNameValue {
                    path,
                    value:
                        Expr::Lit(ExprLit {
                            lit: Lit::Str(s), ..
                        }),
                    ..
                }) => {
                    let key = path.get_ident().map(|i| i.to_string()).unwrap_or_default();
                    match key.as_str() {
                        "name" => attrs.name = Some(s.value()),
                        "description" => attrs.description = Some(s.value()),
                        "category" => attrs.category = Some(s.value()),
                        other => {
                            return Err(syn::Error::new(
                                s.span(),
                                format!("unknown string attribute: `{other}`"),
                            ));
                        },
                    }
                },
                Meta::List(MetaList {
                    path,
                    tokens,
                    ..
                }) => {
                    let key = path.get_ident().map(|i| i.to_string()).unwrap_or_default();
                    if key == "aliases" {
                        // parse as comma-separated string literals
                        let lits = syn::parse2::<AliasArgs>(tokens.clone())?;
                        attrs.aliases = lits.0;
                    } else {
                        return Err(syn::Error::new(
                            path.get_ident().map_or(Span::call_site(), |i| i.span()),
                            format!("unknown list attribute: `{key}`"),
                        ));
                    }
                },
                _ => {
                    return Err(syn::Error::new(Span::call_site(), "unsupported attribute form"));
                },
            }
        }
        Ok(attrs)
    }
}

struct AliasArgs(Vec<String>);

impl Parse for AliasArgs {
    fn parse(input: ParseStream<'_>) -> SynResult<Self> {
        let lits = Punctuated::<LitStr, Token![,]>::parse_terminated(input)?;
        Ok(AliasArgs(lits.iter().map(LitStr::value).collect()))
    }
}

// ---------------------------------------------------------------------------
// `#[command]` macro
// ---------------------------------------------------------------------------

/// Mark an `async fn` as an Oxcord command.
///
/// See the [module docs][crate] for the full list of supported attributes.
#[proc_macro_attribute]
pub fn command(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attrs = parse_macro_input!(attr as CommandAttrs);
    let func = parse_macro_input!(item as ItemFn);

    let fn_name = &func.sig.ident;
    let fn_name_str = fn_name.to_string();

    let cmd_name = attrs.name.as_deref().unwrap_or(&fn_name_str).to_string();
    let description = attrs.description.clone().unwrap_or_else(|| format!("/{cmd_name}"));
    let aliases = &attrs.aliases;

    // Determine CommandKind
    let kind = match (attrs.slash_command, attrs.prefix_command) {
        (true, true) => quote! { ::oxcord_framework::CommandKind::Both },
        (true, false) | (false, false) => quote! { ::oxcord_framework::CommandKind::Slash },
        (false, true) => quote! { ::oxcord_framework::CommandKind::Prefix },
    };

    let guild_only = if attrs.guild_only {
        quote! { .guild_only(true) }
    } else {
        quote! {}
    };

    let owner_only = if attrs.owner_only {
        quote! { .owner_only(true) }
    } else {
        quote! {}
    };

    let category_expr = if let Some(cat) = &attrs.category {
        quote! { .category(#cat) }
    } else {
        quote! {}
    };

    let aliases_expr = if aliases.is_empty() {
        quote! {}
    } else {
        quote! { .aliases([#(#aliases),*]) }
    };

    // The inner async fn keeps its original name; we emit a constructor fn
    // with the same name that returns a Command.
    let inner_fn_name = Ident::new(&format!("__{fn_name}_inner"), fn_name.span());
    let mut inner_fn = func.clone();
    inner_fn.sig.ident = inner_fn_name.clone();

    let output = quote! {
        // The original async function, renamed to avoid name collision.
        #inner_fn

        /// Returns the [`::oxcord_framework::Command`] for this handler.
        ///
        /// Call this function and push the result into [`::oxcord_framework::FrameworkOptions::commands`].
        #[allow(non_snake_case)]
        pub fn #fn_name() -> ::oxcord_framework::Command {
            // Plain fn pointer — satisfies `for<'a> fn(Context<'a>) -> Pin<Box<...>>`.
            fn __handler<'a>(
                ctx: ::oxcord_framework::Context<'a>,
            ) -> ::std::pin::Pin<
                ::std::boxed::Box<
                    dyn ::std::future::Future<
                        Output = ::std::result::Result<(), ::oxcord_framework::FrameworkError>,
                    > + ::std::marker::Send
                    + 'a,
                >,
            > {
                ::std::boxed::Box::pin(#inner_fn_name(ctx))
            }

            ::oxcord_framework::Command::builder(#cmd_name)
                .description(#description)
                .kind(#kind)
                #aliases_expr
                #guild_only
                #owner_only
                #category_expr
                .handler(__handler as ::oxcord_framework::CommandFn)
                .build()
        }
    };

    output.into()
}
