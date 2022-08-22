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
//!
//! fn main() {
//!     yew::Renderer::<App>::new().render();
//! }
//! ```
#![allow(clippy::needless_doctest_main)]

pub mod dispatch;
pub mod functional;
pub mod storage;

// Used by macro.
#[doc(hidden)]
pub use log;

pub use anyflux::{self, listener, mrc, store, subscriber};

pub mod prelude {
    //! Default exports

    pub use crate::{
        dispatch::DispatchExt,
        functional::{
            use_selector, use_selector_eq, use_selector_eq_with_deps, use_selector_with_deps,
            use_store, use_store_value,
        },
    };

    pub use yewdux_macros::Store;

    pub use anyflux::{mrc, prelude::*};
}
