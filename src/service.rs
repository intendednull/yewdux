//! Wrapper for components with shared state.
use std::collections::HashSet;
use std::rc::Rc;

use yew::{
    agent::{Agent, AgentLink, Context, Dispatcher, HandlerId},
    prelude::*,
};

use crate::store::{Reduction, ReductionOnce, Store, StoreLink};

/// Message send to [StateService](StateService).
pub enum ServiceRequest<H>
where
    H: Store,
{
    /// Apply a state change.
    Apply(Reduction<H::Model>),
    /// Apply a state change once.
    ApplyOnce(ReductionOnce<H::Model>),
}

/// Message sent to [StateService](StateService) subscribers.
pub enum ServiceResponse<H>
where
    H: Store,
{
    /// Current state, sent every time state changes.
    State(Rc<H::Model>),
}

/// Input message for either [StateService](StateService) or
/// [StateHandler](crate::handler::StateHandler).
pub enum ServiceInput<H>
where
    H: Store,
{
    Service(ServiceRequest<H>),
    Store(H::Input),
}

/// Output message from either [StateService](StateService) or
/// [StateHandler](crate::handler::StateHandler).
pub enum ServiceOutput<H>
where
    H: Store,
{
    Service(ServiceResponse<H>),
    Store(H::Output),
}

/// Context agent for managing shared state. In charge of applying changes to state then notifying
/// subscribers of new state.
pub struct StoreService<STORE, SCOPE = STORE>
where
    STORE: Store + 'static,
    SCOPE: 'static,
{
    store: STORE,
    subscriptions: HashSet<HandlerId>,
    link: AgentLink<StoreService<STORE, SCOPE>>,
    #[allow(dead_code)]
    self_dispatcher: Dispatcher<Self>,
}

impl<STORE, SCOPE> Agent for StoreService<STORE, SCOPE>
where
    STORE: Store + 'static,
    SCOPE: 'static,
{
    type Message = STORE::Message;
    type Reach = Context<Self>;
    type Input = ServiceInput<STORE>;
    type Output = ServiceOutput<STORE>;

    fn create(link: AgentLink<Self>) -> Self {
        Self {
            store: <STORE as Store>::new(StoreLink::new(link.clone())),
            subscriptions: Default::default(),
            self_dispatcher: Self::dispatcher(),
            link,
        }
    }

    fn update(&mut self, msg: Self::Message) {
        let changed = self.store.update(msg);
        if changed {
            self.store.changed();
            self.notify_subscribers();
        }
    }

    fn handle_input(&mut self, msg: Self::Input, who: HandlerId) {
        match msg {
            ServiceInput::Service(msg) => match msg {
                ServiceRequest::Apply(reduce) => {
                    reduce(Rc::make_mut(self.store.state()));
                    self.store.changed();
                }
                ServiceRequest::ApplyOnce(reduce) => {
                    reduce(Rc::make_mut(self.store.state()));
                    self.store.changed();
                }
            },
            ServiceInput::Store(msg) => {
                let changed = self.store.handle_input(msg, who);
                if changed {
                    self.store.changed();
                    self.notify_subscribers();
                }
            }
        }

        self.notify_subscribers();
    }

    fn connected(&mut self, who: HandlerId) {
        // Add component to subscriptions.
        self.subscriptions.insert(who);
        // Send current state.
        let state = self.store.state().clone();
        self.link
            .respond(who, ServiceOutput::Service(ServiceResponse::State(state)));
    }

    fn disconnected(&mut self, who: HandlerId) {
        self.subscriptions.remove(&who);
    }
}

impl<STORE, SCOPE> StoreService<STORE, SCOPE>
where
    STORE: Store + 'static,
    SCOPE: 'static,
{
    fn notify_subscribers(&mut self) {
        let state = self.store.state();
        for who in self.subscriptions.iter().cloned() {
            self.link.respond(
                who,
                ServiceOutput::Service(ServiceResponse::State(state.clone())),
            );
        }
    }
}

/// A bridge to a [StateService]. This allows message passing with state handlers, as well as their
/// parent service. Useful when you want to access the [events](ServiceResponse) emitted by
/// [StateService].
///
/// [StateService]: StateService
pub struct ServiceBridge<H, SCOPE = H>
where
    H: Store + 'static,
    SCOPE: 'static,
{
    bridge: Box<dyn Bridge<StoreService<H, SCOPE>>>,
}

impl<H, SCOPE> ServiceBridge<H, SCOPE>
where
    H: Store + 'static,
{
    /// Create a new bridge, automatically [subscribing](ServiceRequest::Subscribe).
    pub fn new(callback: Callback<ServiceOutput<H>>) -> Self {
        Self {
            bridge: StoreService::bridge(callback),
        }
    }

    /// Send message to service.
    pub fn send_service(&mut self, msg: ServiceRequest<H>) {
        self.bridge.send(ServiceInput::Service(msg));
    }

    /// Send message to handler.
    pub fn send_store(&mut self, msg: H::Input) {
        self.bridge.send(ServiceInput::Store(msg));
    }
}

impl<H> From<ServiceRequest<H>> for ServiceInput<H>
where
    H: Store,
{
    fn from(msg: ServiceRequest<H>) -> Self {
        ServiceInput::Service(msg)
    }
}

impl<H> From<ServiceResponse<H>> for ServiceOutput<H>
where
    H: Store,
{
    fn from(msg: ServiceResponse<H>) -> Self {
        ServiceOutput::Service(msg)
    }
}
