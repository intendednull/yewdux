pub mod component;
pub mod dispatch;
pub mod service;
pub mod store;

pub mod prelude {
    pub use yew::agent::HandlerId;
    pub use yew_services::storage::Area;

    pub use crate::component::{StateView, WithDispatch};
    pub use crate::dispatch::{Dispatch, DispatchProps, DispatchPropsMut, Dispatcher};
    pub use crate::store::{
        basic::BasicStore,
        persistent::{Persistent, PersistentStore},
        reducer::{Reducer, ReducerStore},
        ShouldNotify, Store, StoreLink,
    };
}
