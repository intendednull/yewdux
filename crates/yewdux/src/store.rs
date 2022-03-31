//! Unique state shared application-wide
pub use yewdux_macros::Store;

/// Globally shared state.
pub trait Store: Clone + 'static {
    /// Initialize this store.
    fn new() -> Self;

    /// Called after state has changed.
    fn changed(&mut self) {}
}

/// A message that can change state.
pub trait Message<S> {
    /// Mutate state using this message.
    fn apply(&self, state: &mut S);
}
