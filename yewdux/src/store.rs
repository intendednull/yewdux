/// A container for shared state.
pub trait Store: Clone + 'static {
    type Message;

    /// Initialize this store.
    fn new() -> Self;

    /// Called after state has changed.
    fn changed(&mut self) {}

    /// Handle store message, returning whether state has changed.
    fn update(&mut self, _msg: Self::Message) {}
}
