//! Unique state shared application-wide
use std::rc::Rc;

pub use yewdux_macros::Store;

/// Globally shared state.
pub trait Store: 'static {
    /// Create this store.
    fn new() -> Self;

    /// Indicate whether or not state has changed.
    fn changed(&self, other: &Self) -> bool;
}

/// A type that can change state.
pub trait Reducer<S> {
    /// Mutate state.
    fn apply(&self, state: Rc<S>) -> Rc<S>;
}
