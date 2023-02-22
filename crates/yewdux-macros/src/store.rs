use darling::FromDeriveInput;
use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;

#[derive(FromDeriveInput, Default)]
#[darling(default, attributes(store))]
struct Opts {
    storage: Option<String>,
    storage_tab_sync: bool,
}

pub(crate) fn derive(input: DeriveInput) -> TokenStream {
    let opts = Opts::from_derive_input(&input).expect("Invalid options");
    let ident = input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let impl_ = match opts.storage {
        #[cfg(target_arch = "wasm32")]
        Some(storage) => {
            let area = match storage.as_ref() {
                "local" => quote! { ::yewdux::storage::Area::Local },
                "session" => quote! { ::yewdux::storage::Area::Session },
                x => panic!(
                    "'{}' is not a valid option. Must be 'local' or 'session'.",
                    x
                ),
            };

            let sync = if opts.storage_tab_sync {
                quote! {
                    if let Err(err) = ::yewdux::storage::init_tab_sync::<Self>(#area) {
                        ::yewdux::log::error!("Unable to init tab sync for storage: {:?}", err);
                    }
                }
            } else {
                quote!()
            };

            quote! {
                fn new() -> Self {
                    ::yewdux::listener::init_listener(
                        ::yewdux::storage::StorageListener::<Self>::new(#area)
                    );

                    #sync

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
        _ => quote! {
            fn new() -> Self {
                Default::default()
            }
        },
    };

    quote! {
        #[automatically_derived]
        impl #impl_generics ::yewdux::store::Store for #ident #ty_generics #where_clause {
            #impl_

            fn should_notify(&self, other: &Self) -> bool {
                self != other
            }
        }
    }
}
