//! Unique state shared application-wide
use std::rc::Rc;

pub use yewdux_macros::Store;

/// Globally shared state.
pub trait Store: 'static {
    /// Create this store.
    fn new() -> Self;

    /// Indicate whether or not subscribers should be notified about this change. Usually this
    /// should be set to `self != old`.
    fn should_notify(&self, old: &Self) -> bool;
}

/// A type that can change state.
pub trait Reducer<S> {
    /// Mutate state.
    fn apply(&self, state: Rc<S>) -> Rc<S>;
}
