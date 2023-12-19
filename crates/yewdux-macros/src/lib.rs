use proc_macro::TokenStream;
use proc_macro_error::proc_macro_error;
use syn::{parse_macro_input, DeriveInput};

mod store;

#[proc_macro_derive(Store, attributes(store))]
#[proc_macro_error]
pub fn store(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    store::derive(input).into()
}
