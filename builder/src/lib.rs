extern crate proc_macro;

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};
use quote::quote;

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;
    let b_name = format!("{}Builder", name);
    let b_ident = syn::Ident::new(&b_name, name.span());

    let expanded = quote! {
        struct #b_ident {
        }
        impl #name {
            fn builder() -> #b_ident {
                #b_ident {
                }
            }
        }
    };

    expanded.into()
}
