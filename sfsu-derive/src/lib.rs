use proc_macro::TokenStream;
use proc_macro_error::proc_macro_error;
use syn::{parse_macro_input, DeriveInput};

mod deprecateable;
mod hooks;
mod inner;
mod keyvalue;

#[proc_macro_derive(Runnable)]
#[proc_macro_error]
pub fn derive_into_inner(input: TokenStream) -> TokenStream {
    inner::into_inner(parse_macro_input!(input as DeriveInput)).into()
}

#[proc_macro_derive(Hooks, attributes(no_hook))]
#[proc_macro_error]
pub fn derive_hook_enum(input: TokenStream) -> TokenStream {
    hooks::hook_enum(parse_macro_input!(input as DeriveInput)).into()
}

#[proc_macro_derive(Deprecateable, attributes(deprecated))]
#[proc_macro_error]
#[deprecated(note = "Use `sfsu::deprecate` instead")]
pub fn derive_deprecateable(input: TokenStream) -> TokenStream {
    deprecateable::deprecateable(parse_macro_input!(input as DeriveInput)).into()
}

#[proc_macro_derive(KeyValue, attributes(deprecated))]
#[proc_macro_error]
pub fn derive_key_value(input: TokenStream) -> TokenStream {
    keyvalue::keyvalue(parse_macro_input!(input as DeriveInput)).into()
}
