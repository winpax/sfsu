use proc_macro2::{Ident, Span, TokenStream};
use proc_macro_crate::{crate_name, FoundCrate};
use proc_macro_error::abort_call_site;
use quote::quote;
use syn::{punctuated::Punctuated, DeriveInput, Token};

struct InnerParams(syn::Type);

impl syn::parse::Parse for InnerParams {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let content;
        syn::parenthesized!(content in input);
        let ret_type = content.parse()?;
        Ok(InnerParams(ret_type))
    }
}

pub fn into_inner(ast: DeriveInput) -> TokenStream {
    let input_name = &ast.ident;

    let data = &ast.data;
    let Ok(attrs) = &ast
        .attrs
        .into_iter()
        .filter(|variant| variant.path().is_ident("inner_type"))
        .try_fold(Vec::new(), |mut vec, attr| -> syn::Result<_> {
            vec.extend(
                attr.parse_args_with(Punctuated::<InnerParams, Token![,]>::parse_terminated)?,
            );
            Ok(vec)
        }) else {
            abort_call_site!("#[derive(IntoInner)] requires the `inner_type` attribute to be specified, with a return type identifier");
        };

    let type_ident = &attrs.first().unwrap().0;

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
}
