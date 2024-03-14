extern crate proc_macro;

use proc_macro::{Punct, Spacing, TokenStream, TokenTree};
use quote::quote;
use syn::{parse_macro_input, ExprMatch, PatRange};

#[proc_macro]
pub fn u_num_it(tokens: TokenStream) -> TokenStream {
    let mut it = tokens.into_iter();
    let range = it.take_while(|t| t != TokenTree::Punct(Punct::new(",", Spacing::Alone)));
}
