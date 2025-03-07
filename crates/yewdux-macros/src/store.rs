use darling::{util::PathList, FromDeriveInput};
use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;

#[derive(FromDeriveInput, Default)]
#[darling(default, attributes(store))]
struct Opts {
    storage: Option<String>,
    storage_tab_sync: bool,
    listener: PathList,
    derived_from: PathList,
    derived_from_mut: PathList,
}

pub(crate) fn derive(input: DeriveInput) -> TokenStream {
    let opts = Opts::from_derive_input(&input).expect("Invalid options");
    let ident = input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let extra_listeners: Vec<_> = opts
        .listener
        .iter()
        .map(|path| {
            quote! {
                ::yewdux::listener::init_listener(
                    || #path, cx
                );
            }
        })
        .collect();
        
    let derived_from_init: Vec<_> = opts
        .derived_from
        .iter()
        .map(|source_type| {
            quote! {
                cx.derived_from::<#source_type, Self>();
            }
        })
        .collect();
        
    let derived_from_mut_init: Vec<_> = opts
        .derived_from_mut
        .iter()
        .map(|source_type| {
            quote! {
                cx.derived_from_mut::<#source_type, Self>();
            }
        })
        .collect();

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

            let sync = if opts.storage_tab_sync {
                quote! {
                    if let Err(err) = ::yewdux::storage::init_tab_sync::<Self>(#area, cx) {
                        ::yewdux::log::error!("Unable to init tab sync for storage: {:?}", err);
                    }
                }
            } else {
                quote!()
            };

            quote! {
                #[cfg(target_arch = "wasm32")]
                fn new(cx: &::yewdux::Context) -> Self {
                    ::yewdux::listener::init_listener(
                        || ::yewdux::storage::StorageListener::<Self>::new(#area),
                        cx
                    );
                    #(#extra_listeners)*
                    #(#derived_from_init)*
                    #(#derived_from_mut_init)*

                    #sync

                    match ::yewdux::storage::load(#area) {
                        Ok(val) => val.unwrap_or_default(),
                        Err(err) => {
                            ::yewdux::log::error!("Error loading state from storage: {:?}", err);

                            Default::default()
                        }
                    }

                }

                #[cfg(not(target_arch = "wasm32"))]
                fn new(cx: &::yewdux::Context) -> Self {
                    #(#extra_listeners)*
                    #(#derived_from_init)*
                    #(#derived_from_mut_init)*
                    Default::default()
                }
            }
        }
        None => quote! {
            fn new(cx: &::yewdux::Context) -> Self {
                #(#extra_listeners)*
                #(#derived_from_init)*
                #(#derived_from_mut_init)*
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
