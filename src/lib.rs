//! Location agnostic shared state for yew components.
//! ## Usage
//!
//! Give your component `GlobalHandle` properties and wrap it with `SharedStateComponent`.
//! This may be done for any `T` that implements `Clone` + `Default`.
//! ```rust
//! struct Model {
//!     handle: GlobalHandle<T>,
//! }
//!
//! impl Component for Model {
//!     type Properties = GlobalHandle<T>;
//!     ...
//! }
//!
//! type MyComponent = SharedStateComponent<Model>;
//! ```
//!
//! Access current state with `state`.
//! ```rust
//! let state: &T = self.handle.state();
//! ```
//!
//! Modify shared state from anywhere using `reduce`
//! ```rust
//! // GlobalHandle<MyAppState>
//! self.handle.reduce(|state| state.user = new_user);
//! ```
//!
//! or from a callback with `reduce_callback`.
//! ```rust
//! // GlobalHandle<usize>
//! let onclick = self.handle.reduce_callback(|state| *state += 1);
//! html! {
//!     <button onclick = onclick>{"+1"}</button>
//! }
//! ```
//!
//! `reduce_callback_with` provides the fired event.
//! ```rust
//! let oninput = self
//!     .handle
//!     .reduce_callback_with(|state, i: InputData| state.user.name = i.value);
//!
//! html! {
//!     <input type="text" placeholder = "Enter your name" oninput = oninput />
//! }
//! ```
//!
//! ### Properties with Shared State
//!
//! Get shared state in custom props with `SharedState`.
//! ```rust
//! #[derive(Clone, Properties)]
//! pub struct Props {
//!     #[prop_or_default]
//!     pub handle: GlobalHandle<AppState>,
//! }
//!
//! impl SharedState for Props {
//!     type Handle = GlobalHandle<AppState>;
//!
//!     fn handle(&mut self) -> &mut Self::Handle {
//!         &mut self.handle
//!     }
//! }
//! ```
//!
//! ### State Persistence
//!
//! Persistent storage requires that `T` also implement `Serialize`,
//! `Deserialize`, and `Storable`.
//! ```rust
//! use serde::{Serialize, Deserialize};
//! use yew_state::Storable;
//! use yew::services::storage::Area;
//!
//! #[derive(Serialize, Deserialize)]
//! struct T;
//!
//! impl Storable for T {
//!     fn key() -> &'static str {
//!         "myapp.storage.t"
//!     }
//!
//!     fn area() -> Area {
//!         // or Area::Session
//!         Area::Local
//!     }
//! }
//! ```
//!
//! Then use `StorageHandle` instead of `GlobalHandle`.
pub mod component;
mod handle;
mod handler;

pub use yew::services::storage::Area;

pub use component::SharedStateComponent;
pub use handle::{GlobalHandle, Handle, SharedState, StorageHandle};
pub use handler::Storable;
