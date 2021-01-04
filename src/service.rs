//! Wrapper for components with shared state.
use std::collections::HashSet;
use std::rc::Rc;

use yew::{
    agent::{Agent, AgentLink, Context, Dispatcher, HandlerId},
    prelude::*,
};

use crate::handler::{HandlerLink, Reduction, ReductionOnce, StateHandler};

/// Message send to [StateService](StateService).
pub enum ServiceRequest<H>
where
    H: StateHandler,
{
    /// Apply a state change.
    Apply(Reduction<H::Model>),
    /// Apply a state change once.
    ApplyOnce(ReductionOnce<H::Model>),
    /// Registers the sender to be [notified](ServiceResponse) when state changes.
    Subscribe,
}

/// Message sent to [StateService](StateService) subscribers.
pub enum ServiceResponse<H>
where
    H: StateHandler,
{
    /// Current state, sent every time state changes.
    State(Rc<H::Model>),
    /// Link to state handler. Sent once on [subscribe](ServiceRequest::Subscribe).
    Link(HandlerLink<H>),
}

/// Input message for either [StateService](StateService) or
/// [StateHandler](crate::handler::StateHandler).
pub enum ServiceInput<H>
where
    H: StateHandler,
{
    Service(ServiceRequest<H>),
    Handler(H::Input),
}

/// Output message from either [StateService](StateService) or
/// [StateHandler](crate::handler::StateHandler).
pub enum ServiceOutput<H>
where
    H: StateHandler,
{
    Service(ServiceResponse<H>),
    Handler(H::Output),
}

/// Context agent for managing shared state. In charge of applying changes to state then notifying
/// subscribers of new state.
pub struct StateService<HANDLER, SCOPE = HANDLER>
where
    HANDLER: StateHandler + 'static,
    SCOPE: 'static,
{
    handler: HANDLER,
    subscriptions: HashSet<HandlerId>,
    link: AgentLink<StateService<HANDLER, SCOPE>>,
    #[allow(dead_code)]
    self_dispatcher: Dispatcher<Self>,
}

impl<HANDLER, SCOPE> Agent for StateService<HANDLER, SCOPE>
where
    HANDLER: StateHandler + 'static,
    SCOPE: 'static,
{
    type Message = HANDLER::Message;
    type Reach = Context<Self>;
    type Input = ServiceInput<HANDLER>;
    type Output = ServiceOutput<HANDLER>;

    fn create(link: AgentLink<Self>) -> Self {
        Self {
            handler: <HANDLER as StateHandler>::new(HandlerLink::new(link.clone())),
            subscriptions: Default::default(),
            self_dispatcher: Self::dispatcher(),
            link,
        }
    }

    fn update(&mut self, msg: Self::Message) {
        let changed = self.handler.update(msg);
        if changed {
            self.handler.changed();
            self.notify_subscribers();
        }
    }

    fn handle_input(&mut self, msg: Self::Input, who: HandlerId) {
        match msg {
            ServiceInput::Service(msg) => match msg {
                ServiceRequest::Apply(reduce) => {
                    reduce(Rc::make_mut(self.handler.state()));
                    self.handler.changed();
                }
                ServiceRequest::ApplyOnce(reduce) => {
                    reduce(Rc::make_mut(self.handler.state()));
                    self.handler.changed();
                }
                ServiceRequest::Subscribe => {
                    // Add component to subscriptions.
                    self.subscriptions.insert(who);
                    // Send current state.
                    let state = Rc::clone(self.handler.state());
                    self.link
                        .respond(who, ServiceOutput::Service(ServiceResponse::State(state)));
                    // Send handler link.
                    self.link.respond(
                        who,
                        ServiceOutput::Service(ServiceResponse::Link(HandlerLink::new(
                            self.link.clone(),
                        ))),
                    );
                }
            },
            ServiceInput::Handler(msg) => {
                let changed = self.handler.handle_input(msg, who);
                if changed {
                    self.handler.changed();
                    self.notify_subscribers();
                }
            }
        }

        self.notify_subscribers();
    }

    fn disconnected(&mut self, who: HandlerId) {
        self.subscriptions.remove(&who);
    }
}

impl<HANDLER, SCOPE> StateService<HANDLER, SCOPE>
where
    HANDLER: StateHandler + 'static,
    SCOPE: 'static,
{
    fn notify_subscribers(&mut self) {
        let state = self.handler.state();
        for who in self.subscriptions.iter().cloned() {
            self.link.respond(
                who,
                ServiceOutput::Service(ServiceResponse::State(Rc::clone(state))),
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
    H: StateHandler + 'static,
    SCOPE: 'static,
{
    bridge: Box<dyn Bridge<StateService<H, SCOPE>>>,
}

impl<H, SCOPE> ServiceBridge<H, SCOPE>
where
    H: StateHandler + 'static,
{
    /// Create a new bridge, automatically [subscribing](ServiceRequest::Subscribe).
    pub fn new(callback: Callback<ServiceOutput<H>>) -> Self {
        let mut bridge = StateService::bridge(callback);
        bridge.send(ServiceInput::Service(ServiceRequest::Subscribe));

        Self { bridge }
    }

    /// Send message to service.
    pub fn send_service(&mut self, msg: ServiceRequest<H>) {
        self.bridge.send(ServiceInput::Service(msg));
    }

    /// Send message to handler.
    pub fn send_handler(&mut self, msg: H::Input) {
        self.bridge.send(ServiceInput::Handler(msg));
    }
}

impl<H> From<ServiceRequest<H>> for ServiceInput<H>
where
    H: StateHandler,
{
    fn from(msg: ServiceRequest<H>) -> Self {
        ServiceInput::Service(msg)
    }
}

impl<H> From<ServiceResponse<H>> for ServiceOutput<H>
where
    H: StateHandler,
{
    fn from(msg: ServiceResponse<H>) -> Self {
        ServiceOutput::Service(msg)
    }
}
