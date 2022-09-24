#![doc = include_str!("../README.md")]
use proc_macro::TokenStream;
use proc_macro2::TokenTree;
use quote::quote;

#[proc_macro_attribute]
pub fn run(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = proc_macro2::TokenStream::from(item);

    let mut ident = None;
    for item in input
        .clone()
        .into_iter()
        .collect::<Vec<TokenTree>>()
        .windows(2)
    {
        if let TokenTree::Ident(ident0) = &item[0] {
            if ident0 == "struct" {
                if let TokenTree::Ident(ident1) = &item[1] {
                    ident = Some(ident1.clone());
                    break;
                }
            }
        }
    }

    let ident = ident.unwrap();
    let to_ret = quote! {
        fn main() {
            #ident::run();
        }
        #input
    };

    proc_macro::TokenStream::from(to_ret)
}
