use darling::FromDeriveInput;
use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;

#[derive(FromDeriveInput, Default)]
#[darling(default, attributes(store))]
struct Opts {
    storage: Option<String>,
}

pub(crate) fn derive(input: DeriveInput) -> TokenStream {
    let opts = Opts::from_derive_input(&input).expect("Invalid options");
    let ident = input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let impl_ = match opts.storage {
        Some(storage) => {
            let area = match storage.as_ref() {
                "local" => quote! { ::yewdux::storage::Area::Local },
                "session" => quote! { ::yewdux::storage::Area::Session },
                x => panic!(
                    "'{}' is not a valid option. Must be 'local' or 'session'.",
                    x
                ),
            };

            quote! {
                fn new() -> Self {
                    ::yewdux::listener::init_listener(
                        ::yewdux::storage::StorageListener::<Self>::new(#area)
                    );

                    match ::yewdux::storage::load(#area) {
                        Ok(val) => val.unwrap_or_default(),
                        Err(err) => {
                            ::yewdux::log::error!("Error loading state from storage: {:?}", err);

                            Default::default()
                        }
                    }
                }
            }
        }
        None => quote! {
            fn new() -> Self {
                Default::default()
            }
        },
    };

    quote! {
        #[automatically_derived]
        impl #impl_generics ::yewdux::store::Store for #ident #ty_generics #where_clause {
            #impl_

            fn changed(&self, other: &Self) -> bool {
                self != other
            }
        }
    }
}
