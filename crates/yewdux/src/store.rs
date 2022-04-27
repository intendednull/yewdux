//! Unique state shared application-wide
use std::rc::Rc;

pub use yewdux_macros::Store;

/// Globally shared state.
pub trait Store: PartialEq + 'static {
    /// Initialize this store.
    fn new() -> Self;

    /// Called after state has changed.
    fn changed(&mut self) {}
}

/// A type that can change state.
pub trait Reducer<S> {
    /// Mutate state.
    fn apply(&self, state: Rc<S>) -> Rc<S>;
}
