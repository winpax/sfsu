use proc_macro2::TokenStream;
use proc_macro_error::abort;
use quote::quote;
use syn::{spanned::Spanned, DeriveInput, ExprLit, Lit};

pub fn deprecateable(input: DeriveInput) -> TokenStream {
    let deprecated = input.attrs.iter().find_map(|attr| match attr.meta {
        syn::Meta::Path(ref p) => {
            if p.is_ident("deprecated") {
                Some(None)
            } else {
                None
            }
        }
        syn::Meta::NameValue(ref p) => {
            if p.path.is_ident("deprecated") {
                match p.value {
                    syn::Expr::Lit(ExprLit {
                        lit: Lit::Str(ref msg_lit),
                        ..
                    }) => Some(Some(msg_lit.value())),
                    _ => abort!(
                        p.value.span(),
                        "Invalid deprecation message. Should be string literal"
                    ),
                }
            } else {
                None
            }
        }
        _ => None,
    });

    let deprecation_message = if let Some(Some(ref msg)) = deprecated {
        quote! {
            Some(#msg.to_string())
        }
    } else {
        quote! {
            None
        }
    };

    if deprecated.is_some() {
        let struct_name = &input.ident;
        quote! {
            impl sfsu::Deprecateable for #struct_name {
                fn is_deprecated(&self) -> bool {
                    true
                }

                fn deprecation_message(&self) -> Option<String> {
                    #deprecation_message
                }
            }
        }
    } else {
        let struct_name = &input.ident;
        quote! {
            impl sfsu::Deprecateable for #struct_name {
                fn is_deprecated(&self) -> bool {
                    false
                }
            }
        }
    }
}
