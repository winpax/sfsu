use proc_macro::TokenStream;
use proc_macro_error::{abort_call_site, proc_macro_error};
use quote::{quote, ToTokens};
use syn::{parse_macro_input, DeriveInput};

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

    quote! {
        paste::paste! {
            #[derive(Debug, Clone, strum::Display, strum::IntoStaticStr, strum::EnumIter, PartialEq, Eq)]
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
