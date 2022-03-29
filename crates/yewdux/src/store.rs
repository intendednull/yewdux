/// Globally shared state.
pub trait Store: Clone + 'static {
    /// Initialize this store.
    fn new() -> Self;

    /// Called after state has changed.
    fn changed(&mut self) {}
}

/// A message that can change state.
pub trait Message<S> {
    /// Mutate state based on this message.
    fn update(&self, state: &mut S);
}
