extern crate proc_macro;
extern crate proc_macro2;
extern crate syn;
#[macro_use]
extern crate quote;
extern crate exonum;

use proc_macro2::TokenStream;

#[proc_macro_derive(Schema)]
pub fn schema_custom_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input: TokenStream = input.into();

    let output: TokenStream = input;

    output.into()
}
