use proc_macro::TokenStream;
use proc_macro_error::proc_macro_error;
use syn::{parse_macro_input, DeriveInput};

mod hooks;
mod inner;

#[proc_macro_derive(Runnable)]
#[proc_macro_error]
pub fn derive_into_inner(input: TokenStream) -> TokenStream {
    inner::into_inner(parse_macro_input!(input as DeriveInput)).into()
}

#[proc_macro_derive(RawEnum)]
#[proc_macro_error]
pub fn derive_raw_enum(input: TokenStream) -> TokenStream {
    hooks::hook_enum(parse_macro_input!(input as DeriveInput)).into()
}
