//! State handlers determine how state should be created, modified, and shared.
use std::pin::Pin;
use std::rc::Rc;

use std::future::Future;
use yew::Callback;

use yew_agent::{AgentLink, HandlerId};

use super::Store;
use crate::service::{ServiceInput, ServiceOutput, StoreService};

pub struct StoreLink<STORE>
where
    STORE: Store,
{
    link: Rc<
        dyn AgentLinkWrapper<
            Message = STORE::Message,
            Input = STORE::Input,
            Output = STORE::Output,
        >,
    >,
}

impl<STORE> Clone for StoreLink<STORE>
where
    STORE: Store,
{
    fn clone(&self) -> Self {
        Self {
            link: self.link.clone(),
        }
    }
}

type StoreMsg<H> = <H as Store>::Message;
type StoreInput<H> = <H as Store>::Input;
type StoreOutput<H> = <H as Store>::Output;

impl<STORE: Store> StoreLink<STORE> {
    pub(crate) fn new(
        link: impl AgentLinkWrapper<
                Message = StoreMsg<STORE>,
                Input = StoreInput<STORE>,
                Output = StoreOutput<STORE>,
            > + 'static,
    ) -> Self {
        Self {
            link: Rc::new(link),
        }
    }

    pub fn send_message<T>(&self, msg: T)
    where
        T: Into<StoreMsg<STORE>>,
    {
        self.link.send_message(msg.into())
    }

    pub fn send_input<T>(&self, msg: T)
    where
        T: Into<StoreInput<STORE>>,
    {
        self.link.send_input(msg.into())
    }

    pub fn respond<T>(&self, who: HandlerId, output: T)
    where
        T: Into<StoreOutput<STORE>>,
    {
        self.link.respond(who, output.into())
    }

    pub fn callback<F, IN, M>(&self, function: F) -> Callback<IN>
    where
        StoreInput<STORE>: 'static,
        StoreOutput<STORE>: 'static,
        StoreMsg<STORE>: 'static,
        M: Into<StoreMsg<STORE>>,
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
        StoreInput<STORE>: 'static,
        StoreOutput<STORE>: 'static,
        StoreMsg<STORE>: 'static,
        M: Into<StoreMsg<STORE>>,
        F: FnOnce(IN) -> M + 'static,
    {
        let link = self.link.clone();
        let cb = move |x| {
            let result = function(x);
            link.send_message(result.into());
        };

        Callback::once(cb)
    }

    pub fn send_future<F, M>(&self, future: F)
    where
        M: Into<StoreMsg<STORE>>,
        F: Future<Output = M> + 'static,
    {
        let future = async { future.await.into() };
        self.link.send_future(Box::pin(future))
    }

    pub fn callback_future<FN, FU, IN, M>(&self, function: FN) -> yew::Callback<IN>
    where
        StoreInput<STORE>: 'static,
        StoreOutput<STORE>: 'static,
        StoreMsg<STORE>: 'static,
        M: Into<StoreMsg<STORE>>,
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

    pub fn callback_once_future<FN, FU, IN, M>(&self, function: FN) -> yew::Callback<IN>
    where
        StoreInput<STORE>: 'static,
        StoreOutput<STORE>: 'static,
        StoreMsg<STORE>: 'static,
        M: Into<StoreMsg<STORE>>,
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

pub(crate) trait AgentLinkWrapper {
    type Message;
    type Input;
    type Output;

    fn send_message(&self, msg: Self::Message);
    fn send_input(&self, input: Self::Input);
    fn respond(&self, who: HandlerId, output: Self::Output);
    fn send_future(&self, future: Pin<Box<dyn Future<Output = Self::Message>>>);
}

impl<H, SCOPE> AgentLinkWrapper for AgentLink<StoreService<H, SCOPE>>
where
    H: Store,
{
    type Message = H::Message;
    type Input = H::Input;
    type Output = H::Output;

    fn send_message(&self, msg: Self::Message) {
        AgentLink::<StoreService<H, SCOPE>>::send_message(self, msg)
    }

    fn send_input(&self, input: Self::Input) {
        AgentLink::<StoreService<H, SCOPE>>::send_input(self, ServiceInput::Store(input))
    }

    fn respond(&self, who: HandlerId, output: Self::Output) {
        AgentLink::<StoreService<H, SCOPE>>::respond(self, who, ServiceOutput::Store(output))
    }

    fn send_future(&self, future: Pin<Box<dyn Future<Output = Self::Message>>>) {
        AgentLink::<StoreService<H, SCOPE>>::send_future(self, future)
    }
}
