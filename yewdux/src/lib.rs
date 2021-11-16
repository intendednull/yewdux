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
//! ## Example
//! ```no_run
//! use std::rc::Rc;
//!
//! use yew::prelude::*;
//! use yewdux::prelude::*;
//!
//! #[derive(Default, Clone)]
//! struct State {
//!     count: u32,
//! }
//!
//! struct App {
//!     /// Our local version of state.
//!     state: Rc<State>,
//!     dispatch: Dispatch<BasicStore<State>>,
//! }
//!
//! enum Msg {
//!     /// Message to receive new state.
//!     State(Rc<State>),
//! }
//!
//! impl Component for App {
//!     type Message = Msg;
//!     type Properties = ();
//!
//!     fn create(ctx: &Context<Self>) -> Self {
//!         // Create Dispatch with a bridge that receives new state.
//!         let dispatch = Dispatch::bridge_state(ctx.link().callback(Msg::State));
//!         // Magically increment our counter for this example.
//!         // NOTE: Changes aren't immediate! We won't see new state until we receive it in our update
//!         // method.
//!         dispatch.reduce(|s: &mut State| s.count += 1);
//!
//!         Self {
//!             dispatch,
//!             state: Default::default(),
//!         }
//!     }
//!
//!     fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
//!         match msg {
//!             Msg::State(state) => {
//!                 // Receive new state and re-render.
//!                 self.state = state;
//!                 true
//!             }
//!         }
//!     }
//!
//!     fn view(&self, _ctx: &Context<Self>) -> Html {
//!         let count = self.state.count;
//!         // We can modify state with callbacks too!
//!         let onclick = self.dispatch.reduce_callback(|s| s.count += 1);
//!
//!         html! {
//!             <>
//!             <h1>{ count }</h1>
//!             <button {onclick}>{"+1"}</button>
//!             </>
//!         }
//!     }
//! }
//!
//!
//! pub fn main() {
//!     yew::start_app::<App>();
//! }
//! ```
#![allow(clippy::needless_doctest_main)]

pub mod component;
pub mod dispatch;
pub mod service;
pub mod store;

pub mod prelude {
    //! Everything you need to use Yewdux.

    pub use yew_agent::HandlerId;

    pub use crate::component::WithDispatch;
    pub use crate::dispatch::{Dispatch, DispatchProps, Dispatcher, WithDispatchProps};
    pub use crate::store::{
        basic::BasicStore,
        persistent::{Area, Persistent, PersistentStore},
        reducer::{Reducer, ReducerStore},
        Changed, Store, StoreLink,
    };
}
