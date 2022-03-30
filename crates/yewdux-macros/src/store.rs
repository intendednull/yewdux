use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;

pub(crate) fn macro_fn(input: DeriveInput) -> TokenStream {
    let ident = input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    quote! {
        #[automatically_derived]
        impl #impl_generics ::yewdux::store::Store for #ident #ty_generics #where_clause {
            fn new() -> Self {
                Default::default()
            }
        }
    }
}
