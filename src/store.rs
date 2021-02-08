//! State handlers determine how state should be created, modified, and shared.
pub mod default;
mod link;
pub mod storage;

use std::rc::Rc;

use yew::{
    agent::{Bridge, Bridged, HandlerId},
    Callback,
};

use crate::service::{ServiceInput, ServiceOutput, StoreService};
pub use default::DefaultStore;
pub use link::StoreLink;
pub use storage::{Storage, StorageModel};

pub type ShouldNotify = bool;
pub(crate) type Reduction<T> = Rc<dyn Fn(&mut T)>;
pub(crate) type ReductionOnce<T> = Box<dyn FnOnce(&mut T)>;

/// Determines how state should be created, modified, and shared.
pub trait Store: Sized {
    type Model: Clone;
    type Message;
    type Input;
    type Output;

    /// Create new state.
    fn new(_link: StoreLink<Self>) -> Self;

    /// Return a reference to current state.
    fn state(&mut self) -> &mut Rc<Self::Model>;

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

/// A direct bridge to given state handler. Unlike a [ServiceBridge](crate::service::ServiceBridge)
/// bridge, it can only send and receive [StateHandler](StateHandler) messages.
pub struct StoreBridge<H, SCOPE = H>
where
    H: Store + 'static,
    SCOPE: 'static,
{
    bridge: Box<dyn Bridge<StoreService<H, SCOPE>>>,
}

impl<H, SCOPE> StoreBridge<H, SCOPE>
where
    H: Store + 'static,
{
    pub fn new(callback: Callback<H::Output>) -> Self {
        let callback = move |msg: ServiceOutput<H>| match msg {
            ServiceOutput::Store(msg) => callback.emit(msg),
            // Service should only send messages to those who subscribe. We don't subscribe, so we
            // shouldn't receive any messages here.
            ServiceOutput::Service(_) => unreachable!(),
        };

        Self {
            bridge: StoreService::<_, SCOPE>::bridge(callback.into()),
        }
    }

    /// Send a message to the state handler.
    pub fn send(&mut self, msg: H::Input) {
        self.bridge.send(ServiceInput::Store(msg));
    }
}
