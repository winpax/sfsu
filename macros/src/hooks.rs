use proc_macro2::{Ident, Span, TokenStream};
use proc_macro_crate::{crate_name, FoundCrate};
use proc_macro_error2::abort_call_site;
use quote::quote;
use syn::DeriveInput;

pub struct Variant {
    pub name: Ident,
    pub command_name: String,
    pub hook_name: String,
}

impl Variant {
    pub fn into_tuple(self) -> (Ident, String, String) {
        (self.name, self.command_name, self.hook_name)
    }

    pub fn unzip(variants: impl Iterator<Item = Self>) -> (Vec<Ident>, Vec<String>, Vec<String>) {
        let mut names = Vec::new();
        let mut command_names = Vec::new();
        let mut hook_names = Vec::new();

        for variant in variants {
            let (name, command_name, hook_name) = variant.into_tuple();
            names.push(name);
            command_names.push(command_name);
            hook_names.push(hook_name);
        }

        (names, command_names, hook_names)
    }
}

pub fn hook_enum(input: DeriveInput) -> TokenStream {
    let struct_name = {
        let original_ident = &input.ident;
        let og_ident_span = original_ident.span();
        Ident::new(&format!("{}Hooks", original_ident), og_ident_span)
    };

    let data = &input.data;

    let variants = match data {
        syn::Data::Enum(ref e) => e
            .variants
            .iter()
            .filter_map(|variant| {
                // TODO: Refactor this to use one attribute i.e #[hook(...)]
                let attrs = &variant.attrs;
                if attrs.iter().any(|attr| match attr.meta {
                    syn::Meta::Path(ref p) => p.is_ident("no_hook"),
                    _ => false,
                }) {
                    None
                } else {
                    let mut variant = Variant {
                        name: variant.ident.clone(),
                        command_name: heck::AsKebabCase(variant.ident.to_string()).to_string(),
                        hook_name: heck::AsKebabCase(variant.ident.to_string()).to_string(),
                    };

                    for attr in attrs.iter() {
                        if let syn::Meta::NameValue(ref nv) = attr.meta {
                            if nv.path.is_ident("hook_name") {
                                if let syn::Expr::Lit(syn::ExprLit {
                                    lit: syn::Lit::Str(ref s),
                                    ..
                                }) = nv.value
                                {
                                    variant.hook_name = s.value();
                                }
                            }

                            if nv.path.is_ident("command_name") {
                                if let syn::Expr::Lit(syn::ExprLit {
                                    lit: syn::Lit::Str(ref s),
                                    ..
                                }) = nv.value
                                {
                                    variant.command_name = s.value();
                                }
                            }
                        }
                    }

                    Some(variant)
                }
            })
            .collect::<Vec<_>>(),
        _ => abort_call_site!("Can only be derived for enums"),
    };

    let (variants, command_names, hook_names) = Variant::unzip(variants.into_iter());

    let quork = match crate_name("quork").expect("quork is present in `Cargo.toml`") {
        FoundCrate::Itself => Ident::new("quork", Span::call_site()),
        FoundCrate::Name(name) => Ident::new(&name, Span::call_site()),
    };

    quote! {
        // TODO: Better way of doing this? or add support for meta in proc macro
        #[derive(Debug, Copy, Clone, #quork::macros::ListVariants, PartialEq, Eq)]
        pub enum #struct_name {
            #(#variants),*
        }

        impl #struct_name {
            pub const fn command<'a>(self) -> &'a str {
                match self {
                    #(#struct_name::#variants => #command_names,)*
                }
            }

            pub const fn hook<'a>(self) -> &'a str {
                match self {
                    #(#struct_name::#variants => #hook_names,)*
                }
            }
        }

        // impl std::fmt::Display for #struct_name {
        //     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        //         write!(f, "{}", self.hook())
        //     }
        // }

        impl From<String> for #struct_name {
            fn from(string: String) -> Self {
                match string.as_str() {
                    #(#hook_names => #struct_name::#variants,)*
                    _ => panic!("Invalid command name: {}", string),
                }
            }
        }
    }
}
