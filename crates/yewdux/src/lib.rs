//! # Yewdux
//!
//! Simple state management for [Yew](https://yew.rs) applications.
//!
//! This is the development branch. Latest stable release may be found
//! [here](https://github.com/intendednull/yewdux/tree/0.7.0).
//!
//! See the [book](https://intendednull.github.io/yewdux/) for more details.
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
        functional::{
            use_selector, use_selector_eq, use_selector_eq_with_deps, use_selector_with_deps,
            use_store, use_store_value,
        },
        listener::{init_listener, Listener},
        store::{Reducer, Store},
    };
}
