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
//! #[function_component(App)]
//! fn app() -> Html {
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
//!     yew::start_app::<App>();
//! }
//! ```
//!
#![allow(clippy::needless_doctest_main)]

// pub mod component;
mod context;
pub mod dispatch;
pub mod functional;
pub mod listener;
pub mod mrc;
pub mod storage;
pub mod store;
mod subscriber;

// Used by macro.
#[doc(hidden)]
pub use log;

pub mod prelude {
    //! Default exports

    pub use crate::{
        dispatch::Dispatch,
        functional::use_store,
        listener::{init_listener, Listener},
        store::{Reducer, Store},
    };
}
