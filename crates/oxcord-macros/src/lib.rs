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
//! | `cooldown_secs = N` | integer | Per-command cooldown duration |
//! | `cooldown_scope = "..."` | string | `"global"`, `"user"` (default), `"guild"`, `"channel"`, or `"member"` |
//! | `required_permissions = "A \| B"` | string | Permissions the invoking user must have (slash only — see [`oxcord_framework::Command::required_permissions`]) |
//! | `param("name", "description"[, "required" \| "optional"])` | list | Override a slash option's description (and, optionally, whether Discord requires it; prefix parsing still follows the `Option<T>` type). Repeat per parameter |
//! | `required_bot_permissions = "A \| B"` | string | Permissions the bot must have (slash only — see [`oxcord_framework::Command::required_bot_permissions`]) |
//!
//! Any parameter after `ctx` becomes a typed command argument: extracted from
//! the matching named slash option, or positionally from prefix text, and
//! registered with Discord as a slash-command option automatically. Wrap the
//! type in `Option<..>` to make it optional.
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
    FnArg,
    GenericArgument,
    Ident,
    ItemFn,
    Lit,
    LitStr,
    Meta,
    MetaList,
    MetaNameValue,
    Pat,
    PathArguments,
    Token,
    Type,
    parse_macro_input,
};

/// If `ty` is `Option<Inner>`, returns `Inner`. Otherwise `None`.
fn option_inner(ty: &Type) -> Option<&Type> {
    let Type::Path(type_path) = ty else {
        return None;
    };
    let segment = type_path.path.segments.last()?;
    if segment.ident != "Option" {
        return None;
    }
    let PathArguments::AngleBracketed(args) = &segment.arguments else {
        return None;
    };
    args.args.iter().find_map(|arg| match arg {
        GenericArgument::Type(inner) => Some(inner),
        _ => None,
    })
}

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
    cooldown_secs: Option<u64>,
    cooldown_scope: Option<String>,
    required_permissions: Option<String>,
    required_bot_permissions: Option<String>,
    /// `param("nama", "deskripsi"[, "required" | "optional"])`: menimpa deskripsi dan status wajib opsi.
    params: Vec<ParamOverride>,
}

struct ParamOverride {
    name: String,
    description: String,
    required: Option<bool>,
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
                        "cooldown_scope" => attrs.cooldown_scope = Some(s.value()),
                        "required_permissions" => attrs.required_permissions = Some(s.value()),
                        "required_bot_permissions" => {
                            attrs.required_bot_permissions = Some(s.value())
                        },
                        other => {
                            return Err(syn::Error::new(
                                s.span(),
                                format!("unknown string attribute: `{other}`"),
                            ));
                        },
                    }
                },
                Meta::NameValue(MetaNameValue {
                    path,
                    value:
                        Expr::Lit(ExprLit {
                            lit: Lit::Int(n), ..
                        }),
                    ..
                }) => {
                    let key = path.get_ident().map(|i| i.to_string()).unwrap_or_default();
                    match key.as_str() {
                        "cooldown_secs" => {
                            attrs.cooldown_secs = Some(n.base10_parse::<u64>()?);
                        },
                        other => {
                            return Err(syn::Error::new(
                                n.span(),
                                format!("unknown integer attribute: `{other}`"),
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
                    if key == "param" {
                        let lits = syn::parse2::<AliasArgs>(tokens.clone())?.0;
                        let (name, description) = match (lits.first(), lits.get(1)) {
                            (Some(n), Some(d)) => (n.clone(), d.clone()),
                            _ => {
                                return Err(syn::Error::new(
                                    path.get_ident().map_or(Span::call_site(), |i| i.span()),
                                    "param(...) needs a name and a description",
                                ));
                            },
                        };
                        let required = match lits.get(2).map(String::as_str) {
                            None => None,
                            Some("required") => Some(true),
                            Some("optional") => Some(false),
                            Some(other) => {
                                return Err(syn::Error::new(
                                    path.get_ident().map_or(Span::call_site(), |i| i.span()),
                                    format!(
                                        "param(...) third value must be \"required\" or \"optional\", got `{other}`"
                                    ),
                                ));
                            },
                        };
                        attrs.params.push(ParamOverride {
                            name,
                            description,
                            required,
                        });
                    } else if key == "aliases" {
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

    let cooldown_expr = attrs.cooldown_secs.map(|secs| {
        let scope_ident = Ident::new(
            match attrs.cooldown_scope.as_deref().unwrap_or("user") {
                "global" => "Global",
                "user" => "User",
                "guild" => "Guild",
                "channel" => "Channel",
                "member" => "Member",
                other => {
                    return quote! {
                        ::std::compile_error!(::std::concat!("unknown cooldown_scope: `", #other, "`"));
                    };
                },
            },
            Span::call_site(),
        );
        quote! {
            .cooldown(::oxcord_framework::CooldownConfig::new(
                ::oxcord_framework::CooldownScope::#scope_ident,
                ::std::time::Duration::from_secs(#secs),
            ))
        }
    });

    let required_permissions_expr = attrs.required_permissions.as_deref().map(|spec| {
        quote! { .required_permissions(::oxcord_framework::parse_permissions(#spec)) }
    });
    let required_bot_permissions_expr = attrs.required_bot_permissions.as_deref().map(|spec| {
        quote! { .required_bot_permissions(::oxcord_framework::parse_permissions(#spec)) }
    });

    // The inner async fn keeps its original name; we emit a constructor fn
    // with the same name that returns a Command.
    let inner_fn_name = Ident::new(&format!("__{fn_name}_inner"), fn_name.span());
    let mut inner_fn = func.clone();
    inner_fn.sig.ident = inner_fn_name.clone();

    // ---------------------------------------------------------------------
    // Extra parameters (anything after `ctx: Context<'_>`) become typed
    // command arguments, pulled out of the slash options (by name) or the
    // prefix text (by position) via `Context::extract`.
    // ---------------------------------------------------------------------
    let mut extract_stmts = Vec::new();
    let mut call_idents = Vec::new();
    let mut option_specs = Vec::new();

    for (position, input) in func.sig.inputs.iter().skip(1).enumerate() {
        let FnArg::Typed(pat_type) = input else {
            return syn::Error::new_spanned(
                input,
                "command parameters must be simple `name: Type` bindings",
            )
            .to_compile_error()
            .into();
        };
        let Pat::Ident(pat_ident) = pat_type.pat.as_ref() else {
            return syn::Error::new_spanned(
                &pat_type.pat,
                "command parameters must be simple identifiers",
            )
            .to_compile_error()
            .into();
        };
        let arg_ident = &pat_ident.ident;
        let arg_name_str = arg_ident.to_string();
        let over = attrs.params.iter().find(|p| p.name == arg_name_str);
        let arg_description =
            over.map_or_else(|| format!("{arg_name_str} parameter"), |p| p.description.clone());
        let ty = pat_type.ty.as_ref();

        let (stmt, option_ty, required) = if let Some(inner) = option_inner(ty) {
            let stmt = quote! {
                let #arg_ident: #ty = match ctx.extract::<#inner>(#arg_name_str, #position) {
                    ::std::result::Result::Ok(v) => v,
                    ::std::result::Result::Err(e) => {
                        return ::std::boxed::Box::pin(async move { ::std::result::Result::Err(e) });
                    },
                };
            };
            (stmt, quote! { #inner }, false)
        } else {
            let stmt = quote! {
                let #arg_ident: #ty = match ctx.extract::<#ty>(#arg_name_str, #position) {
                    ::std::result::Result::Ok(::std::option::Option::Some(v)) => v,
                    ::std::result::Result::Ok(::std::option::Option::None) => {
                        return ::std::boxed::Box::pin(async move {
                            ::std::result::Result::Err(::oxcord_framework::FrameworkError::Other(
                                ::std::format!("missing required argument `{}`", #arg_name_str),
                            ))
                        });
                    },
                    ::std::result::Result::Err(e) => {
                        return ::std::boxed::Box::pin(async move { ::std::result::Result::Err(e) });
                    },
                };
            };
            (stmt, quote! { #ty }, true)
        };
        // Hanya memengaruhi pendaftaran slash; jalur prefix tetap memakai tipe `Option`/non-`Option`.
        let required = over.and_then(|p| p.required).unwrap_or(required);
        extract_stmts.push(stmt);
        call_idents.push(arg_ident.clone());
        option_specs.push(quote! {
            ::oxcord_framework::CommandOptionSpec {
                name: ::std::string::ToString::to_string(#arg_name_str),
                description: ::std::string::ToString::to_string(#arg_description),
                kind: <#option_ty as ::oxcord_framework::FrameworkArgument>::option_type(),
                required: #required,
                choices: <#option_ty as ::oxcord_framework::FrameworkArgument>::choices(),
            }
        });
    }

    let options_expr = if option_specs.is_empty() {
        quote! {}
    } else {
        quote! { .options(::std::vec![#(#option_specs),*]) }
    };

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
                #(#extract_stmts)*
                ::std::boxed::Box::pin(#inner_fn_name(ctx, #(#call_idents),*))
            }

            ::oxcord_framework::Command::builder(#cmd_name)
                .description(#description)
                .kind(#kind)
                #aliases_expr
                #guild_only
                #owner_only
                #category_expr
                #cooldown_expr
                #required_permissions_expr
                #required_bot_permissions_expr
                #options_expr
                .handler(__handler as ::oxcord_framework::CommandFn)
                .build()
        }
    };

    output.into()
}

/// Derive [`oxcord_framework::ChoiceParameter`] dan [`oxcord_framework::FrameworkArgument`]
/// untuk enum unit yang dipakai sebagai pilihan tetap pada slash command.
#[proc_macro_derive(ChoiceParameter, attributes(choice))]
pub fn derive_choice_parameter(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::DeriveInput);
    let enum_ident = &input.ident;
    let syn::Data::Enum(data_enum) = &input.data else {
        return syn::Error::new_spanned(
            &input.ident,
            "#[derive(ChoiceParameter)] hanya mendukung enum",
        )
        .to_compile_error()
        .into();
    };

    let mut list_entries = Vec::new();
    let mut from_index_arms = Vec::new();
    let mut from_name_arms = Vec::new();
    let mut name_arms = Vec::new();

    for (idx, variant) in data_enum.variants.iter().enumerate() {
        if !matches!(variant.fields, syn::Fields::Unit) {
            return syn::Error::new_spanned(
                &variant.ident,
                "#[derive(ChoiceParameter)] hanya mendukung varian unit tanpa field",
            )
            .to_compile_error()
            .into();
        }
        let var_ident = &variant.ident;
        let var_name = var_ident.to_string();
        // `#[choice("Label")]` mengganti nama yang tampil di Discord; nilai tetap nama varian.
        let mut label = var_name.clone();
        for attr in &variant.attrs {
            if attr.path().is_ident("choice") {
                match attr.parse_args::<LitStr>() {
                    Ok(l) => label = l.value(),
                    Err(e) => return e.to_compile_error().into(),
                }
            }
        }
        list_entries.push(quote! {
            ::oxcord_framework::ChoiceParameterInfo {
                name: ::std::borrow::Cow::Borrowed(#label),
                value: ::std::borrow::Cow::Borrowed(#var_name),
                localizations: ::std::vec::Vec::new(),
            }
        });
        from_index_arms.push(quote! { #idx => ::std::option::Option::Some(Self::#var_ident), });
        from_name_arms.push(quote! { s if s.eq_ignore_ascii_case(#var_name) => ::std::option::Option::Some(Self::#var_ident), });
        name_arms.push(quote! { Self::#var_ident => #var_name, });
    }

    let expanded = quote! {
        impl ::oxcord_framework::ChoiceParameter for #enum_ident {
            fn list() -> ::std::vec::Vec<::oxcord_framework::ChoiceParameterInfo> {
                ::std::vec![#(#list_entries),*]
            }

            fn from_index(index: usize) -> ::std::option::Option<Self> {
                match index {
                    #(#from_index_arms)*
                    _ => ::std::option::Option::None,
                }
            }

            fn from_name(name: &str) -> ::std::option::Option<Self> {
                match name {
                    #(#from_name_arms)*
                    _ => ::std::option::Option::None,
                }
            }

            fn name(&self) -> &'static str {
                match self {
                    #(#name_arms)*
                }
            }
        }

        impl ::oxcord_framework::FrameworkArgument for #enum_ident {
            fn from_option(
                value: &::oxcord_model::application::CommandDataOptionValue,
            ) -> ::std::result::Result<Self, ::oxcord_framework::FrameworkError> {
                match value {
                    ::oxcord_model::application::CommandDataOptionValue::String(s) => {
                        <Self as ::oxcord_framework::ChoiceParameter>::from_name(s).ok_or_else(|| {
                            ::oxcord_framework::FrameworkError::ArgumentParse {
                                input: ::std::option::Option::Some(s.clone()),
                                error: ::std::boxed::Box::new(::std::io::Error::new(
                                    ::std::io::ErrorKind::InvalidInput,
                                    "unknown choice value",
                                )),
                            }
                        })
                    },
                    other => ::std::result::Result::Err(::oxcord_framework::FrameworkError::Other(
                        ::std::format!("expected String choice, got {:?}", other.kind()),
                    )),
                }
            }

            fn from_token(token: &str) -> ::std::result::Result<Self, ::oxcord_framework::FrameworkError> {
                <Self as ::oxcord_framework::ChoiceParameter>::from_name(token).ok_or_else(|| {
                    ::oxcord_framework::FrameworkError::ArgumentParse {
                        input: ::std::option::Option::Some(token.to_owned()),
                        error: ::std::boxed::Box::new(::std::io::Error::new(
                            ::std::io::ErrorKind::InvalidInput,
                            "unknown choice token",
                        )),
                    }
                })
            }

            fn option_type() -> ::oxcord_model::application::CommandOptionType {
                ::oxcord_model::application::CommandOptionType::String
            }

            fn choices() -> ::std::vec::Vec<(::std::string::String, ::std::string::String)> {
                <Self as ::oxcord_framework::ChoiceParameter>::list()
                    .into_iter()
                    .map(|c| (c.name.into_owned(), c.value.into_owned()))
                    .collect()
            }
        }
    };
    expanded.into()
}
