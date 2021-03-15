//! Shared state container
pub mod basic;
mod link;
pub mod persistent;
pub mod reducer;

use std::rc::Rc;

pub use yew::agent::HandlerId;

pub use link::StoreLink;

pub type Changed = bool;
pub(crate) type Reduction<T> = Rc<dyn Fn(&mut T)>;
pub(crate) type ReductionOnce<T> = Box<dyn FnOnce(&mut T)>;

/// A container for shared state.
pub trait Store: Sized + 'static {
    type Model: Clone;
    type Message;
    type Input;
    type Output;

    /// Initialize this store.
    fn new(_link: StoreLink<Self>) -> Self;

    /// Return a mutable reference to current state.
    fn state(&mut self) -> &mut Rc<Self::Model>;

    /// Called after state has changed.
    fn changed(&mut self) {}

    /// Handle store message, returning whether state has changed.
    fn update(&mut self, _msg: Self::Message) -> Changed {
        false
    }

    /// Handle store input message, returning whether state has changed.
    #[allow(unused_variables)]
    fn handle_input(&mut self, msg: Self::Input, _who: HandlerId) -> Changed {
        false
    }
}
