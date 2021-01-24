//! State handlers determine how state should be created, modified, and shared.
use std::any::type_name;
#[cfg(feature = "future")]
use std::pin::Pin;
use std::rc::Rc;

use serde::{Deserialize, Serialize};
#[cfg(feature = "future")]
use std::future::Future;
use yew::{
    agent::{AgentLink, Bridge, Bridged, HandlerId},
    format::Json,
    Callback,
};
use yew_services::{storage::Area, StorageService};
#[cfg(feature = "future")]
use yewtil::future::LinkFuture;

use crate::service::{ServiceInput, ServiceOutput, StateService};

pub(crate) type Reduction<T> = Rc<dyn Fn(&mut T)>;
pub(crate) type ReductionOnce<T> = Box<dyn FnOnce(&mut T)>;
pub type Changed = bool;

pub(crate) trait AgentLinkWrapper {
    type Message;
    type Input;
    type Output;

    fn send_message(&self, msg: Self::Message);
    fn send_input(&self, input: Self::Input);
    fn respond(&self, who: HandlerId, output: Self::Output);
    #[cfg(feature = "future")]
    fn send_future(&self, future: Pin<Box<dyn Future<Output = Self::Message>>>);
}

impl<H, SCOPE> AgentLinkWrapper for AgentLink<StateService<H, SCOPE>>
where
    H: StateHandler,
{
    type Message = H::Message;
    type Input = H::Input;
    type Output = H::Output;

    fn send_message(&self, msg: Self::Message) {
        AgentLink::<StateService<H, SCOPE>>::send_message(self, msg)
    }

    fn send_input(&self, input: Self::Input) {
        AgentLink::<StateService<H, SCOPE>>::send_input(self, ServiceInput::Handler(input))
    }

    fn respond(&self, who: HandlerId, output: Self::Output) {
        AgentLink::<StateService<H, SCOPE>>::respond(self, who, ServiceOutput::Handler(output))
    }

    #[cfg(feature = "future")]
    fn send_future(&self, future: Pin<Box<dyn Future<Output = Self::Message>>>) {
        LinkFuture::send_future(self, future)
    }
}

pub struct HandlerLink<H>
where
    H: StateHandler,
{
    link: Rc<dyn AgentLinkWrapper<Message = H::Message, Input = H::Input, Output = H::Output>>,
}

impl<H> Clone for HandlerLink<H>
where
    H: StateHandler,
{
    fn clone(&self) -> Self {
        Self {
            link: self.link.clone(),
        }
    }
}

type HandlerMsg<H> = <H as StateHandler>::Message;
type HandlerInput<H> = <H as StateHandler>::Input;
type HandlerOutput<H> = <H as StateHandler>::Output;

impl<H: StateHandler> HandlerLink<H> {
    pub(crate) fn new(
        link: impl AgentLinkWrapper<
            Message = HandlerMsg<H>,
            Input = HandlerInput<H>,
            Output = HandlerOutput<H>,
        > + 'static,
    ) -> Self {
        Self {
            link: Rc::new(link),
        }
    }

    pub fn send_message<T>(&self, msg: T)
    where
        T: Into<HandlerMsg<H>>,
    {
        self.link.send_message(msg.into())
    }

    pub fn send_input<T>(&self, msg: T)
    where
        T: Into<HandlerInput<H>>,
    {
        self.link.send_input(msg.into())
    }

    pub fn respond<T>(&self, who: HandlerId, output: T)
    where
        T: Into<HandlerOutput<H>>,
    {
        self.link.respond(who, output.into())
    }

    pub fn callback<F, IN, M>(&self, function: F) -> Callback<IN>
    where
        HandlerInput<H>: 'static,
        HandlerOutput<H>: 'static,
        HandlerMsg<H>: 'static,
        M: Into<HandlerMsg<H>>,
        F: Fn(IN) -> M + 'static,
    {
        let link = self.link.clone();
        let cb = move |x| {
            let result = function(x);
            link.send_message(result.into());
        };

        cb.into()
    }

    pub fn callback_once<F, IN, M>(&self, function: F) -> Callback<IN>
    where
        HandlerInput<H>: 'static,
        HandlerOutput<H>: 'static,
        HandlerMsg<H>: 'static,
        M: Into<HandlerMsg<H>>,
        F: FnOnce(IN) -> M + 'static,
    {
        let link = self.link.clone();
        let cb = move |x| {
            let result = function(x);
            link.send_message(result.into());
        };

        Callback::once(cb)
    }

    #[cfg(feature = "future")]
    pub fn send_future<F, M>(&self, future: F)
    where
        M: Into<HandlerMsg<H>>,
        F: Future<Output = M> + 'static,
    {
        let future = async { future.await.into() };
        self.link.send_future(Box::pin(future))
    }

    #[cfg(feature = "future")]
    pub fn callback_future<FN, FU, IN, M>(&self, function: FN) -> yew::Callback<IN>
    where
        HandlerInput<H>: 'static,
        HandlerOutput<H>: 'static,
        HandlerMsg<H>: 'static,
        M: Into<HandlerMsg<H>>,
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

    #[cfg(feature = "future")]
    pub fn callback_once_future<FN, FU, IN, M>(&self, function: FN) -> yew::Callback<IN>
    where
        HandlerInput<H>: 'static,
        HandlerOutput<H>: 'static,
        HandlerMsg<H>: 'static,
        M: Into<HandlerMsg<H>>,
        FU: Future<Output = M> + 'static,
        FN: FnOnce(IN) -> FU + 'static,
    {
        let link = self.link.clone();
        let cb = move |x| {
            let future = function(x);
            let future = async { future.await.into() };
            link.send_future(Box::pin(future));
        };

        Callback::once(cb)
    }
}

/// Determines how state should be created, modified, and shared.
pub trait StateHandler: Sized {
    type Model: Clone;
    type Message;
    type Input;
    type Output;

    /// Create new state.
    fn new(_link: HandlerLink<Self>) -> Self;

    /// Return a reference to current state.
    fn state(&mut self) -> &mut Rc<Self::Model>;

    /// Called after state is changed.
    fn changed(&mut self) {}

    /// Receive messages from components.
    fn update(&mut self, _msg: Self::Message) -> Changed {
        false
    }

    #[allow(unused_variables)]
    fn handle_input(&mut self, msg: Self::Input, _who: HandlerId) -> Changed {
        false
    }
}

/// A direct bridge to given state handler. Unlike a [ServiceBridge](crate::service::ServiceBridge)
/// bridge, it can only send and receive [StateHandler](StateHandler) messages.
pub struct HandlerBridge<H, SCOPE = H>
where
    H: StateHandler + 'static,
    SCOPE: 'static,
{
    bridge: Box<dyn Bridge<StateService<H, SCOPE>>>,
}

impl<H, SCOPE> HandlerBridge<H, SCOPE>
where
    H: StateHandler + 'static,
{
    pub fn new(callback: Callback<H::Output>) -> Self {
        let callback = move |msg: ServiceOutput<H>| match msg {
            ServiceOutput::Handler(msg) => callback.emit(msg),
            // Service should only send messages to those who subscribe. We don't subscribe, so we
            // shouldn't receive any messages here.
            ServiceOutput::Service(_) => unreachable!(),
        };

        Self {
            bridge: StateService::<_, SCOPE>::bridge(callback.into()),
        }
    }

    /// Send a message to the state handler.
    pub fn send(&mut self, msg: H::Input) {
        self.bridge.send(ServiceInput::Handler(msg));
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
    type Input = ();
    type Output = ();

    fn new(_link: HandlerLink<Self>) -> Self {
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
    type Input = ();
    type Output = ();

    fn new(_link: HandlerLink<Self>) -> Self {
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

impl<T: Storable> Storable for Option<T> {
    fn key() -> &'static str {
        T::key()
    }

    fn area() -> Area {
        T::area()
    }
}

impl<T: Storable> Storable for Rc<T> {
    fn key() -> &'static str {
        T::key()
    }

    fn area() -> Area {
        T::area()
    }
}
