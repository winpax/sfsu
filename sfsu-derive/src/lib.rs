use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use proc_macro_crate::{crate_name, FoundCrate};
use proc_macro_error::{abort_call_site, proc_macro_error};
use quote::{quote, ToTokens};
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(IntoInner, attributes(inner_type))]
#[proc_macro_error]
pub fn derive_into_inner(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let input_name = &input.ident;

    let data = &input.data;
    let attrs = &input.attrs;

    let type_ident = {
        let type_ident = attrs.iter().find(|attr| attr.path().is_ident("inner_type"));

        match type_ident {
            Some(type_name) => type_name,
            None => {
                abort_call_site!("#[derive(IntoInner)] requires the `inner_type` attribute to be specified, with a return type identifier");
            }
        }
    };

    let mut variants = vec![];

    match data {
        syn::Data::Enum(ref e) => {
            for v in &e.variants {
                variants.push(v.ident.clone());
            }
        }
        _ => abort_call_site!("Can only be derived for enums"),
    };

    let paste = match crate_name("paste").expect("paste is present in `Cargo.toml`") {
        FoundCrate::Itself => Ident::new("paste", Span::call_site()),
        FoundCrate::Name(name) => Ident::new(&name, Span::call_site()),
    };

    quote! {
        #paste::paste! {
            impl #input_name {
                fn into_inner(self) -> #type_ident {
                    match self {
                        #(#(Self::#variants) (a) => a)*
                        _ => panic!("Invalid command name"),
                    }
                }
            }
        }
    }
    .into()
}

#[proc_macro_derive(RawEnum)]
#[proc_macro_error]
pub fn derive_raw_enum(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let input_name = &input.ident;

    let data = &input.data;

    let mut variants = vec![];

    match data {
        syn::Data::Enum(ref e) => {
            for v in &e.variants {
                variants.push(v.ident.to_token_stream());
            }
        }
        _ => abort_call_site!("Can only be derived for enums"),
    };

    let command_names = variants
        .iter()
        .map(|variant| heck::AsKebabCase(variant.to_string()).to_string())
        .collect::<Vec<_>>();

    let paste = match crate_name("paste").expect("heck is present in `Cargo.toml`") {
        FoundCrate::Itself => Ident::new("paste", Span::call_site()),
        FoundCrate::Name(name) => Ident::new(&name, Span::call_site()),
    };

    let strum = match crate_name("strum").expect("strum is present in `Cargo.toml`") {
        FoundCrate::Itself => Ident::new("strum", Span::call_site()),
        FoundCrate::Name(name) => Ident::new(&name, Span::call_site()),
    };

    quote! {
        #paste::paste! {
            #[derive(Debug, Clone, #strum::Display, #strum::IntoStaticStr, #strum::EnumIter, PartialEq, Eq)]
            #[strum(serialize_all = "kebab-case")]
            pub enum [<#input_name Raw>] {
                #(#variants),*
            }

            impl From<String> for [<#input_name Raw>] {
                fn from(string: String) -> Self {
                    match string.as_str() {
                        #(#command_names => [<#input_name Raw>]::#variants,)*
                        _ => panic!("Invalid command name: {}", string),
                    }
                }
            }
        }
    }
    .into()
}
