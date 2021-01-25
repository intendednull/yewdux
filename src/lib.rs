pub mod component;
pub mod handle;
pub mod handler;
pub mod service;
pub mod reducer_handler;

pub use yew_services::storage::Area;

pub use component::{SharedStateComponent, StateView};
pub use handle::{LinkHandle, SharedHandle, SharedState, StateHandle, StorageHandle};
pub use handler::Storable;
