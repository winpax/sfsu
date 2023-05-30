use proc_macro2::TokenStream;
use proc_macro_error::{abort, abort_call_site};
use quote::quote;
use syn::{punctuated::Punctuated, DeriveInput, Token};

mod kw {
    use syn::custom_keyword;

    custom_keyword!(ret);
}

struct InnerParams(syn::Type);

impl syn::parse::Parse for InnerParams {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(kw::ret) {
            input.parse::<kw::ret>()?;
            input.parse::<Token![=]>()?;
            let ret_type = input.parse()?;
            Ok(InnerParams(ret_type))
        } else {
            panic!()
        }
    }
}

pub fn into_inner(ast: DeriveInput) -> TokenStream {
    let input_name = &ast.ident;

    let data = &ast.data;
    let params = ast
        .attrs
        .into_iter()
        .filter(|variant| variant.path().is_ident("inner"))
        .try_fold(Vec::new(), |mut vec, attr| -> syn::Result<_> {
            vec.extend(
                attr.parse_args_with(Punctuated::<InnerParams, Token![,]>::parse_terminated)?,
            );
            Ok(vec)
        });

    let attrs = match &params {
        Ok(x) => x,
        Err(e) => {
            abort!(
                e.span(),
                "Failed to parse #[inner] attribute arguments: {}",
                e
            )
        }
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

    quote! {
        impl #input_name {
            fn into_inner(self) -> #type_ident {
                match self {
                    #(Self::#variants (a) => a,)*
                    _ => panic!("Invalid command name"),
                }
            }
        }
    }
}
