pub mod component;
pub mod dispatcher;
pub mod service;
pub mod store;

pub mod prelude {
    use super::*;

    pub use yew_services::storage::Area;

    pub use component::{StateView, WithDispatcher};
    pub use dispatcher::{Dispatcher, DispatcherProp};
    pub use store::StorageModel;
}
