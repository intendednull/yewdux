pub mod component;
pub mod handle;
mod handler;

pub use yew::services::storage::Area;

pub use component::{SharedStateComponent, StateView};
pub use handle::{SharedHandle, SharedState, StorageHandle};
pub use handler::Storable;
