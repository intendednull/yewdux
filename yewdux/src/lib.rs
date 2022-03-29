//! # Yewdux
//!
//! Shared state containers for [Yew](https://yew.rs/docs/en/) applications.
//!
//! State management in Yew can get cumbersome, especially when you need to give many (potentially
//! isolated) components mutable access to shared state. Normally you would need to write individual
//! properties and callbacks for each component to propagate changes -- too much typing if you as me!
//! Yewdux provides an ergonomic interface for shared state containers. They can be accessed from any
//! component or agent, live for entire application lifetime, and are clone-on-write by default.
//!
#![allow(clippy::needless_doctest_main)]

// pub mod component;
// pub mod service;
pub mod context;
pub mod dispatch;
pub mod store;
mod util;
// pub mod store;

pub mod prelude {
    //! Everything you need to use Yewdux.

    pub use crate::{
        dispatch::{self, Dispatch},
        store::Store,
    };
}
