#![doc = include_str!("../../../README.md")]
#![allow(clippy::needless_doctest_main)]

// pub mod component;
mod context;
pub mod dispatch;
pub mod functional;
pub mod mrc;
pub mod storage;
pub mod store;
pub mod util;

// Used by macro.
#[doc(hidden)]
pub use log;

pub mod prelude {
    //! Default exports

    pub use crate::{
        dispatch::Dispatch,
        functional::use_store,
        store::{Reducer, Store},
    };
}
