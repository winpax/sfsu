use proc_macro2::TokenStream;
use quote::quote;

pub fn generate_colour_macros(colour: syn::Ident) -> TokenStream {
    let ident = colour;
    let ident_bright = syn::Ident::new(&format!("bright_{ident}"), ident.span());

    let println = syn::Ident::new(&format!("println_{ident}"), ident.span());
    let println_bright = syn::Ident::new(&format!("println_{ident_bright}"), ident.span());
    let eprintln = syn::Ident::new(&format!("e{println}"), ident.span());
    let eprintln_bright = syn::Ident::new(&format!("e{println_bright}"), ident.span());

    quote! {
        #[macro_export]
        #[doc = concat!("Create a colored string with the `", stringify!(#ident), "` color.")]
        macro_rules! #ident {
            ($($arg:tt)*) => {{
                console::style(format_args!($($arg)*)).#ident()
            }};
        }

        #[macro_export]
        #[doc = concat!("Create a colored string with the `", stringify!(#ident_bright), "` color.")]
        macro_rules! #ident_bright {
            ($($arg:tt)*) => {{
                $crate::output::colours::#ident!($($arg)*).bright()
            }};
        }

        #[macro_export]
        #[doc = concat!("Print a colored string with the `", stringify!(#ident), "` color.")]
        macro_rules! #println {
            ($($arg:tt)*) => {{
                println!("{}", $crate::output::colours::#ident!($($arg)*))
            }};
        }

        #[macro_export]
        #[doc = concat!("Print a colored string with the `", stringify!(#ident_bright), "` color.")]
        macro_rules! #println_bright {
            ($($arg:tt)*) => {{
                println!("{}", $crate::output::colours::#ident_bright!($($arg)*))
            }};
        }

        #[macro_export]
        #[doc = concat!("Print a colored string to stderr with the `", stringify!(#ident), "` color.")]
        macro_rules! #eprintln {
            ($($arg:tt)*) => {{
                eprintln!("{}", $crate::output::colours::#ident!($($arg)*))
            }};
        }

        #[macro_export]
        #[doc = concat!("Print a colored string to stderr with the `", stringify!(#ident_bright), "` color.")]
        macro_rules! #eprintln_bright {
            ($($arg:tt)*) => {{
                eprintln!("{}", $crate::output::colours::#ident_bright!($($arg)*))
            }};
        }

        // pub use #ident;
        // pub use #ident_bright;
        // pub use #println;
        // pub use #println_bright;
        // pub use #eprintln;
        // pub use #eprintln_bright;
    }
}
