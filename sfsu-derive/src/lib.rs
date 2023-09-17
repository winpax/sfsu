use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use proc_macro_crate::{crate_name, FoundCrate};
use proc_macro_error::{abort_call_site, proc_macro_error};
use quote::{quote, ToTokens};
use syn::{parse_macro_input, DeriveInput};

mod inner;

#[proc_macro_derive(Runnable)]
#[proc_macro_error]
pub fn derive_into_inner(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    inner::into_inner(ast).into()
}

#[proc_macro_derive(RawEnum)]
#[proc_macro_error]
pub fn derive_raw_enum(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let struct_name = {
        let original_ident = &input.ident;
        let og_ident_span = original_ident.span();
        Ident::new(&format!("{}Raw", original_ident), og_ident_span)
    };

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

    let strum = match crate_name("strum").expect("strum is present in `Cargo.toml`") {
        FoundCrate::Itself => Ident::new("strum", Span::call_site()),
        FoundCrate::Name(name) => Ident::new(&name, Span::call_site()),
    };

    quote! {
        // TODO: Better way of doing this? or add support for meta in proc macro
        #[derive(Debug, Clone, #strum::Display, #strum::IntoStaticStr, #strum::EnumIter, PartialEq, Eq)]
        #[strum(serialize_all = "kebab-case")]
        pub enum #struct_name {
            #(#variants),*
        }

        impl From<String> for #struct_name {
            fn from(string: String) -> Self {
                match string.as_str() {
                    #(#command_names => #struct_name::#variants,)*
                    _ => panic!("Invalid command name: {}", string),
                }
            }
        }
    }
    .into()
}
