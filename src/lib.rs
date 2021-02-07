pub mod component;
pub mod handle;
pub mod handler;
pub mod service;

pub use yew_services::storage::Area;

pub use component::{SharedStateComponent, StateView};
pub use handle::{Handle, SharedHandle, SharedState, StateHandle, StorageHandle};
pub use handler::Storable;
