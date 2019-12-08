extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

fn ty_is_option(ty: &syn::Type) -> Option<&syn::Type> {
    if let syn::Type::Path(ref p) = ty {
        if !(p.path.segments.len() == 1 && p.path.segments[0].ident == "Option") {
            return None;
        }

        if let syn::PathArguments::AngleBracketed(ref inner_ty) = p.path.segments[0].arguments {
            if inner_ty.args.len() != 1 {
                return None;
            }

            let inner_ty = inner_ty.args.first().unwrap();
            if let syn::GenericArgument::Type(ref t) = inner_ty {
                return Some(t);
            }
        }
    }
    None
}

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = &input.ident;
    let b_name = format!("{}Builder", struct_name);
    let b_ident = syn::Ident::new(&b_name, struct_name.span());

    let fields = if let syn::Data::Struct(syn::DataStruct {
        fields: syn::Fields::Named(syn::FieldsNamed { ref named, .. }),
        ..
    }) = input.data
    {
        named
    } else {
        unimplemented!()
    };

    let optionized = fields.iter().map(|f| {
        let name = &f.ident;
        let ty = &f.ty;
        if ty_is_option(ty).is_some() {
            quote! { #name: #ty }
        } else {
            quote! {
                #name: std::option::Option<#ty>
            }
        }
    });

    let methods = fields.iter().map(|f| {
        let name = &f.ident;
        let ty = &f.ty;
        if let Some(inner_ty) = ty_is_option(ty) {
            quote! {
                pub fn #name(&mut self, #name: #inner_ty) -> &mut Self {
                    self.#name = Some(#name);
                    self
                }
            }
        } else {
            quote! {
                pub fn #name(&mut self, #name: #ty) -> &mut Self {
                    self.#name = Some(#name);
                    self
                }
            }
        }
    });

    let build_fields = fields.iter().map(|f| {
        let name = &f.ident;
        if ty_is_option(&f.ty).is_some() {
            return quote! {
                #name: self.#name.clone()
            };
        }
        quote! {
            #name: self.#name.clone().ok_or(concat!(stringify!(#name), " is not set"))?
        }
    });

    let builder_names = fields.iter().map(|f| {
        let name = &f.ident;
        quote! {
            #name: None
        }
    });

    let expanded = quote! {
        struct #b_ident {
            #(#optionized,)*
        }
        impl #b_ident {
            #(#methods)*
            pub fn build(&mut self) -> Result<#struct_name, Box<dyn std::error::Error>> {
                Ok(#struct_name {
                    #(#build_fields,)*
                })
            }
        }
        impl #struct_name {
            fn builder() -> #b_ident {
                #b_ident {
                    #(#builder_names,)*
                }
            }
        }
    };

    expanded.into()
}
