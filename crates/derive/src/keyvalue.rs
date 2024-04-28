use proc_macro2::{Span, TokenStream};
use proc_macro_error::abort_call_site;
use quote::quote;
use syn::{DataStruct, DeriveInput, Ident};

pub fn keyvalue(input: DeriveInput) -> TokenStream {
    let ident = input.ident;
    let data = input.data;

    let fields = match data {
        syn::Data::Struct(DataStruct { fields, .. }) => fields
            .into_iter()
            .map(|field| match field.ty.clone() {
                syn::Type::Path(v) => {
                    let is_option = v
                        .path
                        .segments
                        .first()
                        .unwrap()
                        .ident
                        .to_string()
                        .starts_with("Option");

                    let ident = field.ident.unwrap();

                    if is_option {
                        quote! {
                            if let Some(data) = self.#ident {
                                keys.push(stringify!(#ident));
                                values.push(data.to_string());
                            }
                        }
                    } else {
                        quote! {
                            keys.push(stringify!(#ident));
                            values.push(self.#ident.to_string())
                        }
                    }
                }
                _ => unimplemented!(),
            })
            .collect::<Vec<_>>(),
        _ => abort_call_site!("Can only be derived on structs with fields"),
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
        impl #sprinkles::KeyValue for #ident {
            fn into_pairs(self) -> (Vec<&'static str>, Vec<String>) {
                let mut keys = vec![];
                let mut values = vec![];
                #(#fields;)*
                (keys, values)
            }
        }
    }
}
