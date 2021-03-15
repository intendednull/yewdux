pub mod component;
pub mod dispatch;
pub mod service;
pub mod store;

pub use yew_services::storage::Area;

pub use component::{StateView, WithDispatch};
pub use dispatch::{Dispatch, DispatchPropsMut};
pub use store::{
    basic::BasicStore,
    persistent::{Persistent, PersistentStore},
    reducer::{Reducer, ReducerStore},
    ShouldNotify,
};
