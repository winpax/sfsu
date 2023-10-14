use proc_macro2::{Ident, Span, TokenStream};
use proc_macro_crate::{crate_name, FoundCrate};
use proc_macro_error::{abort, abort_call_site};
use quote::{quote, ToTokens};
use syn::{spanned::Spanned, DeriveInput};

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
                if variant.attrs.iter().any(|attr| match attr.meta {
                    syn::Meta::Path(ref p) => p.is_ident("no_hook"),
                    _ => abort!(
                        attr.span(),
                        "Expected path-style (i.e #[no_hook]), found other style attribute macro"
                    ),
                }) {
                    None
                } else {
                    Some(variant.ident.to_token_stream())
                }
            })
            .collect::<Vec<_>>(),
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
}
