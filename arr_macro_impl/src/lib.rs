extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(MyMacro)]
pub fn my_macro_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let expanded = quote! {
        impl #name {
            pub fn my_macro_method() {
                println!("MyMacro derived for {}", stringify!(#name));
            }
        }
    };
    TokenStream::from(expanded)
}

