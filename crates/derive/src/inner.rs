use proc_macro2::{Ident, Span, TokenStream};
use proc_macro_error::abort_call_site;
use quote::quote;
use syn::DeriveInput;

pub fn into_inner(ast: DeriveInput) -> TokenStream {
    let input_name = &ast.ident;

    let data = &ast.data;

    let mut variants = vec![];

    match data {
        syn::Data::Enum(ref e) => {
            for v in &e.variants {
                variants.push(v.ident.clone());
            }
        }
        _ => abort_call_site!("Can only be derived for enums"),
    };

    let sprinkles =
        match proc_macro_crate::crate_name("sprinkles").expect("sprinkles library to exist") {
            proc_macro_crate::FoundCrate::Itself => quote! { crate },
            proc_macro_crate::FoundCrate::Name(name) => {
                let ident = Ident::new(&name, Span::call_site());
                quote!( #ident )
            }
        };

    quote! {
        impl #input_name {
            pub async fn run(self, ctx: &impl #sprinkles::contexts::ScoopContext<#sprinkles::config::Scoop>) -> anyhow::Result<()> {
                match self {
                    #(Self::#variants (a) => a.run(ctx).await,)*
                }
            }
        }
    }
}
