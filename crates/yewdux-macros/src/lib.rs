use proc_macro::TokenStream;
use proc_macro_error::proc_macro_error;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

mod store;

#[proc_macro_derive(Store, attributes(store))]
#[proc_macro_error]
pub fn store(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    store::derive(input).into()
}

#[proc_macro_attribute]
pub fn async_reducer(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let item = proc_macro2::TokenStream::from(item);
    quote! {
        #[::yewdux::async_trait(?Send)]
        #item
    }
    .into()
}
