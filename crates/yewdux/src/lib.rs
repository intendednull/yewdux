#![doc = include_str!("../../../README.md")]
#![allow(clippy::needless_doctest_main)]

// pub mod component;
mod context;
pub mod dispatch;
pub mod functional;
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
        functional::{use_selector, use_selector_eq, use_store, use_store_value},
        store::{Reducer, Store},
    };
}
