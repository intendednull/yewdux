//! State handlers determine how state should be created, modified, and shared.
pub mod basic;
mod link;
pub mod persistent;
pub mod reducer;

use std::rc::Rc;

pub use yew::agent::HandlerId;

pub use link::StoreLink;

pub type ShouldNotify = bool;
pub(crate) type Reduction<T> = Rc<dyn Fn(&mut T)>;
pub(crate) type ReductionOnce<T> = Box<dyn FnOnce(&mut T)>;

/// Determines how state should be created, modified, and shared.
pub trait Store: Sized + 'static {
    type Model;
    type Message;
    type Input;
    type Output;

    /// Create new state.
    fn new(_link: StoreLink<Self>) -> Self;

    /// Mutable reference to current state.
    fn state_mut(&mut self) -> &mut Self::Model;

    /// Reference to current state.
    fn state(&self) -> Rc<Self::Model>;

    /// Called after state is changed.
    fn changed(&mut self) {}

    /// Receive messages from components.
    fn update(&mut self, _msg: Self::Message) -> ShouldNotify {
        false
    }

    #[allow(unused_variables)]
    fn handle_input(&mut self, msg: Self::Input, _who: HandlerId) -> ShouldNotify {
        false
    }
}
