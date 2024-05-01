use proc_macro::TokenStream;
use proc_macro_error::proc_macro_error;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

mod hooks;
mod inner;
mod keyvalue;

#[proc_macro_derive(Runnable)]
#[proc_macro_error]
pub fn derive_into_inner(input: TokenStream) -> TokenStream {
    inner::into_inner(parse_macro_input!(input as DeriveInput)).into()
}

#[proc_macro_derive(Hooks, attributes(no_hook, hook_name, command_name))]
#[proc_macro_error]
pub fn derive_hook_enum(input: TokenStream) -> TokenStream {
    hooks::hook_enum(parse_macro_input!(input as DeriveInput)).into()
}

#[proc_macro_derive(KeyValue, attributes(deprecated))]
#[proc_macro_error]
pub fn derive_key_value(input: TokenStream) -> TokenStream {
    keyvalue::keyvalue(parse_macro_input!(input as DeriveInput)).into()
}

#[proc_macro]
#[proc_macro_error]
pub fn generate(input: TokenStream) -> TokenStream {
    let ident = syn::parse_macro_input!(input as syn::Ident);
    let bright_ident = syn::Ident::new(&format!("bright_{}", ident), ident.span());

    quote! {
        #[macro_export]
        #[doc = concat!("Colorize a string with the `", stringify!(#ident), "` color.")]
        macro_rules! #ident {
            ($($arg:tt)*) => {{
                eprintln!("{}", console::style(format_args!($($arg)*)).#ident())
            }};
        }

        #[macro_export]
        #[doc = concat!("Colorize a string with the `", stringify!(#bright_ident), "` color.")]
        macro_rules! #bright_ident {
            ($($arg:tt)*) => {{
                eprintln!("{}", console::style(format_args!($($arg)*)).#ident().bright())
            }};
        }

        pub use #ident;
        pub use #bright_ident;
    }
    .into()
}
