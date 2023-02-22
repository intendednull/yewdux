//! # Yewdux
//!
//! Simple state management for [Yew](https://yew.rs) applications.
//!
//! See the [book](https://intendednull.github.io/yewdux/) for more details.
//!
//! ## Example
//!
//! ```rust
//! use yew::prelude::*;
//! use yewdux::prelude::*;
//!
//! #[derive(Default, Clone, PartialEq, Eq, Store)]
//! struct State {
//!     count: u32,
//! }
//!
//! #[function_component]
//! fn App() -> Html {
//!     let (state, dispatch) = use_store::<State>();
//!     let onclick = dispatch.reduce_mut_callback(|state| state.count += 1);
//!
//!     html! {
//!         <>
//!         <p>{ state.count }</p>
//!         <button {onclick}>{"+1"}</button>
//!         </>
//!     }
//! }
//! ```
#![allow(clippy::needless_doctest_main)]

mod context;
pub mod dispatch;
pub mod functional;
pub mod listener;
pub mod mrc;
#[cfg(target_arch = "wasm32")]
pub mod storage;
pub mod store;
mod subscriber;

// Used by macro.
#[doc(hidden)]
pub use log;

#[cfg(feature = "future")]
#[doc(hidden)]
pub use async_trait::async_trait;

pub mod prelude {
    //! Default exports

    pub use crate::{
        dispatch::Dispatch,
        functional::{
            use_selector, use_selector_eq, use_selector_eq_with_deps, use_selector_with_deps,
            use_store, use_store_value,
        },
        listener::{init_listener, Listener},
        store::{Reducer, Store},
    };

    #[cfg(feature = "future")]
    pub use crate::store::AsyncReducer;
    #[cfg(feature = "future")]
    pub use yewdux_macros::async_reducer;
}
