#![allow(clippy::needless_doctest_main)]

mod context;
pub mod dispatch;
pub mod listener;
pub mod mrc;
pub mod store;
pub mod subscriber;

pub mod prelude {
    //! Default exports

    pub use crate::{
        dispatch::Dispatch,
        listener::{init_listener, Listener},
        store::{Reducer, Store},
    };
}
