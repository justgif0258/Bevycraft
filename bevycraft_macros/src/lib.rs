use {
    proc_macro::TokenStream,
    proc_macro_crate::{crate_name, FoundCrate},
    quote::{format_ident, quote},
    syn::{
        parse::Parse, parse_macro_input, DeriveInput, Expr, ExprLit, Ident, Lit, MetaNameValue, Token,
        Type, Visibility,
    },
};

struct Entry {
    vis: Visibility,
    kind: Ident,
    name: Ident,
    ty: Type,
    key: Expr,
    def: Expr,
}

struct RegisterInput(Vec<Entry>);

impl Parse for RegisterInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut entries = Vec::new();

        while !input.is_empty() {
            let vis: Visibility = input.parse()?;

            let kind = if input.peek(Token![static]) {
                let token: Token![static] = input.parse()?;
                Ident::new("static", token.span)
            } else if input.peek(Token![const]) {
                let token: Token![const] = input.parse()?;
                Ident::new("const", token.span)
            } else {
                let token: Ident = input.parse()?;
                return Err(syn::Error::new(token.span(), "expected static or const"));
            };

            let name: Ident = input.parse()?;
            input.parse::<Token![:]>()?;
            let ty: Type = input.parse()?;
            input.parse::<Token![=]>()?;

            let kw: Ident = input.parse()?;

            if kw != "register" {
                return Err(syn::Error::new(kw.span(), "expected 'register(...)'"));
            }

            let args;
            syn::parenthesized!(args in input);
            let key: Expr = args.parse()?;
            args.parse::<Token![,]>()?;
            let def: Expr = args.parse()?;

            input.parse::<Token![;]>()?;

            entries.push(Entry {
                vis,
                kind,
                name,
                ty,
                key,
                def,
            });
        }

        Ok(Self(entries))
    }
}

#[proc_macro]
pub fn context(input: TokenStream) -> TokenStream {
    let RegisterInput(entries) = parse_macro_input!(input as RegisterInput);
    let bevycraft_core = bevycraft_core();

    let decls: Vec<proc_macro2::TokenStream> = entries
        .iter()
        .map(|e| {
            let (vis, kind, name, ty, key, def) = (&e.vis, &e.kind, &e.name, &e.ty, &e.key, &e.def);
            quote! {
                #vis #kind #name: #bevycraft_core::prelude::Holder<#ty> = #bevycraft_core::prelude::Holder::new(#key, #def);
            }
        })
        .collect();

    let mut groups: Vec<(String, Vec<&Entry>)> = Vec::new();
    for entry in &entries {
        let key = quote!(#(entry.key)).to_string();

        match groups.iter_mut().find(|(k, _)| k == &key) {
            Some(g) => g.1.push(entry),
            None => groups.push((key, vec![entry])),
        }
    }

    let reg_blocks: Vec<proc_macro2::TokenStream> = groups
        .iter()
        .map(|(_, group)| {
            let ty = &group[0].ty;
            let ops = group.iter().map(|e| {
                let name = &e.name;
                quote! { #name.registrar(registry); }
            });
            quote! {
                {
                    let registry = &mut *#bevycraft_core::prelude::Registrar::<#ty>::write_to_registry();

                    #(#ops)*
                }
            }
        })
        .collect();

    quote! {
        #(#decls)*

        const _: () = {
            #[::ctor::ctor(unsafe)]
            fn __register() {
                #(#reg_blocks)*
            }
        };
    }
    .into()
}

#[proc_macro_derive(Registrar, attributes(registrar))]
pub fn derive_registrar(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let bevycraft_core = bevycraft_core();

    let default_key: Option<String> = input
        .attrs
        .iter()
        .find(|a| a.path().is_ident("registrar"))
        .and_then(|a| a.parse_args::<MetaNameValue>().ok())
        .filter(|nv| nv.path.is_ident("default"))
        .and_then(|nv| match &nv.value {
            Expr::Lit(ExprLit {
                lit: Lit::Str(s), ..
            }) => Some(s.value()),
            _ => None,
        });

    let (registry_type, registry_init, extra_where) = match &default_key {
        Some(key) => (
            quote! { #bevycraft_core::prelude::DefaultedRegistry<#name> },
            quote! {
                #bevycraft_core::prelude::DefaultedRegistry::new(
                    #bevycraft_core::prelude::AssetLocation::parse(#key),
                    <#name as ::std::default::Default>::default(),
                )
            },
            quote! { where #name: ::std::default::Default },
        ),
        None => (
            quote! { #bevycraft_core::prelude::OrderedRegistry<#name> },
            quote! { #bevycraft_core::prelude::OrderedRegistry::new() },
            quote! {},
        ),
    };

    quote! {
        const _: () = {
            static __REGISTRY: ::std::sync::LazyLock<
                ::parking_lot::RwLock<#registry_type>
            > = ::std::sync::LazyLock::new(|| {
                ::parking_lot::RwLock::new(#registry_init)
            });

            static __LOCK: ::std::sync::atomic::AtomicBool = ::std::sync::atomic::AtomicBool::new(false);

            impl #bevycraft_core::prelude::RegistrarOps<#name> for #bevycraft_core::prelude::Registrar<#name> #extra_where {
                #[inline]
                fn read_from_registry<'a>() -> ::parking_lot::RwLockReadGuard<'a, #registry_type> {
                    __REGISTRY.read()
                }

                fn write_to_registry<'a>() -> ::parking_lot::RwLockWriteGuard<'a, #registry_type> {
                    if __LOCK.load(::std::sync::atomic::Ordering::Acquire) {
                        panic!("Tried writing to {}'s registry while it was locked", stringify!(#name));
                    }

                    __REGISTRY.write()
                }
            }
        };
    }
    .into()
}

fn bevycraft_core() -> proc_macro2::TokenStream {
    match crate_name("bevycraft_core").expect("bevycraft_core wasn't found in your Cargo.toml") {
        FoundCrate::Itself => quote! { crate },
        FoundCrate::Name(name) => {
            let ident = format_ident!("{}", name);
            quote! { ::#ident }
        }
    }
}
