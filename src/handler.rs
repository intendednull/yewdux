//! State handlers determine how state should be created, modified, and shared.
use std::any::type_name;
#[cfg(feature = "future")]
use std::pin::Pin;
use std::rc::Rc;

use serde::{Deserialize, Serialize};
#[cfg(feature = "future")]
use std::future::Future;
use yew::{
    agent::{Agent, AgentLink},
    format::Json,
    services::{storage::Area, StorageService},
    Callback,
};
#[cfg(feature = "future")]
use yewtil::future::LinkFuture;

pub(crate) type Reduction<T> = Rc<dyn Fn(&mut T)>;
pub(crate) type ReductionOnce<T> = Box<dyn FnOnce(&mut T)>;
pub type Changed = bool;

pub(crate) trait AgentLinkWrapper {
    type Message;

    fn send_message(&self, msg: Self::Message);

    #[cfg(feature = "future")]
    fn send_future(&self, future: Pin<Box<dyn Future<Output = Self::Message>>>);
}

impl<AGN: Agent> AgentLinkWrapper for AgentLink<AGN> {
    type Message = AGN::Message;

    fn send_message(&self, msg: Self::Message) {
        AgentLink::<AGN>::send_message(self, msg)
    }

    #[cfg(feature = "future")]
    fn send_future(&self, future: Pin<Box<dyn Future<Output = Self::Message>>>) {
        LinkFuture::send_future(self, future)
    }
}

#[derive(Clone)]
pub struct HandlerLink<MSG> {
    link: Rc<dyn AgentLinkWrapper<Message = MSG>>,
}

impl<MSG> HandlerLink<MSG> {
    pub(crate) fn new(link: impl AgentLinkWrapper<Message = MSG> + 'static) -> Self {
        Self {
            link: Rc::new(link),
        }
    }

    pub fn send_message<T>(&self, msg: T)
    where
        T: Into<MSG>,
    {
        self.link.send_message(msg.into())
    }

    pub fn callback<F, IN, M>(&self, function: F) -> Callback<IN>
    where
        MSG: 'static,
        M: Into<MSG>,
        F: Fn(IN) -> M + 'static,
    {
        let link = self.link.clone();
        let cb = move |x| {
            let result = function(x);
            link.send_message(result.into());
        };

        cb.into()
    }

    #[cfg(feature = "future")]
    pub fn send_future<F, M>(&self, future: F)
    where
        M: Into<MSG>,
        F: Future<Output = M> + 'static,
    {
        let future = async { future.await.into() };
        self.link.send_future(Box::pin(future))
    }

    #[cfg(feature = "future")]
    pub fn callback_future<FN, FU, IN, M>(&self, function: FN) -> yew::Callback<IN>
    where
        MSG: 'static,
        M: Into<MSG>,
        FU: Future<Output = M> + 'static,
        FN: Fn(IN) -> FU + 'static,
    {
        let link = self.link.clone();
        let cb = move |x| {
            let future = function(x);
            let future = async { future.await.into() };
            link.send_future(Box::pin(future));
        };

        cb.into()
    }
}

impl<A: Agent> From<AgentLink<A>> for HandlerLink<<A as Agent>::Message> {
    fn from(link: AgentLink<A>) -> Self {
        Self::new(link)
    }
}

/// Determines how state should be created, modified, and shared.
pub trait StateHandler {
    type Model: Clone;
    type Message;

    /// Create new state.
    fn new(_link: HandlerLink<Self::Message>) -> Self;

    /// Return a reference to current state.
    fn state(&mut self) -> &mut Rc<Self::Model>;

    /// Called after state is changed.
    fn changed(&mut self) {}

    /// Receive messages from components.
    fn update(&mut self, _msg: Self::Message) -> Changed {
        false
    }
}

/// Handler for basic shared state.
#[derive(Default, Clone)]
pub struct SharedHandler<T> {
    state: Rc<T>,
}

impl<T> StateHandler for SharedHandler<T>
where
    T: Clone + Default,
{
    type Model = T;
    type Message = ();

    fn new(_link: HandlerLink<Self::Message>) -> Self {
        Default::default()
    }

    fn state(&mut self) -> &mut Rc<Self::Model> {
        &mut self.state
    }
}

/// Allows state to be stored persistently in local or session storage.
pub trait Storable: Serialize + for<'a> Deserialize<'a> {
    /// The key used to save and load state from storage.
    fn key() -> &'static str {
        type_name::<Self>()
    }
    /// The area to store state.
    fn area() -> Area {
        Area::Local
    }
}

impl<T: Storable> Storable for Option<T> {
    fn key() -> &'static str {
        T::key()
    }

    fn area() -> Area {
        T::area()
    }
}

/// Handler for shared state with persistent storage.
///
/// If persistent storage is disabled it just behaves like a `SharedHandler`.
#[derive(Default)]
pub struct StorageHandler<T> {
    state: Rc<T>,
    storage: Option<StorageService>,
}

impl<T> StorageHandler<T>
where
    T: Storable + Default,
{
    pub fn new() -> Self {
        let mut this: Self = Default::default();
        this.storage = StorageService::new(T::area()).ok();
        this.load_state();
        this
    }

    pub fn load_state(&mut self) {
        let result = self.storage.as_mut().map(|s| s.restore(T::key()));
        if let Some(Json(Ok(state))) = result {
            self.state = state;
        }
    }

    pub fn save_state(&mut self) {
        if let Some(storage) = &mut self.storage {
            storage.store(T::key(), Json(&self.state));
        }
    }
}

impl<T> StateHandler for StorageHandler<T>
where
    T: Default + Clone + Storable,
{
    type Model = T;
    type Message = ();

    fn new(_link: HandlerLink<Self::Message>) -> Self {
        Self::new()
    }

    fn state(&mut self) -> &mut Rc<Self::Model> {
        &mut self.state
    }

    fn changed(&mut self) {
        self.save_state();
    }
}

impl<T> Clone for StorageHandler<T>
where
    T: Default + Clone + Storable,
{
    fn clone(&self) -> Self {
        let mut new = Self::new();
        new.state = self.state.clone();
        new
    }
}
