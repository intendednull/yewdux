//! Wrapper for components with shared state.
use std::collections::HashSet;
use std::rc::Rc;

use either::*;

use yew::{
    agent::{Agent, AgentLink, Bridge, Bridged, Context, Dispatcher, HandlerId},
    prelude::*,
};

use crate::handle::{SharedState, StateHandle, WrapperHandle};
use crate::handler::{HandlerLink, Reduction, ReductionOnce, StateHandler};

pub enum Request<H: StateHandler> {
    /// Apply a state change.
    Apply(Reduction<H::Model>),
    /// Apply a state change once.
    ApplyOnce(ReductionOnce<H::Model>),
}

pub enum Response<H>
where
    H: StateHandler,
{
    /// Update subscribers with current state.
    State(Rc<H::Model>),
    Link(HandlerLink<H>),
}

/// Context agent for managing shared state. In charge of applying changes to state then notifying
/// subscribers of new state.
pub struct SharedStateService<HANDLER, SCOPE>
where
    HANDLER: StateHandler + Clone + 'static,
    SCOPE: 'static,
{
    handler: HANDLER,
    subscriptions: HashSet<HandlerId>,
    link: AgentLink<SharedStateService<HANDLER, SCOPE>>,
    #[allow(dead_code)]
    self_dispatcher: Dispatcher<Self>,
}

impl<HANDLER, SCOPE> Agent for SharedStateService<HANDLER, SCOPE>
where
    HANDLER: StateHandler + Clone + 'static,
    SCOPE: 'static,
{
    type Message = HANDLER::Message;
    type Reach = Context<Self>;
    type Input = Either<Request<HANDLER>, HANDLER::Input>;
    type Output = Either<Response<HANDLER>, HANDLER::Output>;

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
            Left(Request::Apply(reduce)) => {
                reduce(Rc::make_mut(self.handler.state()));
                self.handler.changed();
            }
            Left(Request::ApplyOnce(reduce)) => {
                reduce(Rc::make_mut(self.handler.state()));
                self.handler.changed();
            }
            Right(msg) => self.handler.handle_input(msg, who),
        }

        self.notify_subscribers();
    }

    fn connected(&mut self, who: HandlerId) {
        // Add component to subscriptions.
        self.subscriptions.insert(who);
        // Send it current state.
        let state = Rc::clone(self.handler.state());
        self.link.respond(who, Left(Response::State(state)));
        self.link.respond(
            who,
            Left(Response::Link(HandlerLink::new(self.link.clone()))),
        );
    }

    fn disconnected(&mut self, who: HandlerId) {
        self.subscriptions.remove(&who);
    }
}

impl<HANDLER, SCOPE> SharedStateService<HANDLER, SCOPE>
where
    HANDLER: StateHandler + Clone + 'static,
    SCOPE: 'static,
{
    fn notify_subscribers(&mut self) {
        let state = self.handler.state();
        for who in self.subscriptions.iter().cloned() {
            self.link
                .respond(who, Left(Response::State(Rc::clone(state))));
        }
    }
}

type PropHandle<SHARED> = <SHARED as SharedState>::Handle;
type PropHandler<SHARED> = <PropHandle<SHARED> as StateHandle>::Handler;
type Model<T> = <PropHandler<T> as StateHandler>::Model;

#[doc(hidden)]
pub enum SharedStateComponentMsg<SHARED>
where
    SHARED: SharedState,
    <SHARED as SharedState>::Handle: WrapperHandle,
    PropHandler<SHARED>: 'static,
{
    /// Recieve new local state.
    /// IMPORTANT: Changes will **not** be reflected in shared state.
    SetLocal(Rc<Model<SHARED>>),
    SetLink(HandlerLink<PropHandler<SHARED>>),
    /// Update shared state.
    Apply(Reduction<Model<SHARED>>),
    ApplyOnce(ReductionOnce<Model<SHARED>>),
    /// Do nothing.
    Ignore,
}

/// Component wrapper for managing messages and state handles.
///
/// Wraps any component with properties that implement `SharedState`:
/// ```
/// pub type MyComponent = SharedStateComponent<MyComponentModel>;
/// ```
///
/// A scope may be provided to specify where the state is shared:
/// ```
/// // This will only share state with other components using `FooScope`.
/// pub struct FooScope;
/// pub type MyComponent = SharedStateComponent<MyComponentModel, FooScope>;
/// ```
///
/// # Important
/// By default `StorageHandle` and `GlobalHandle` have different scopes. Though not enforced,
/// components with different handles should not use the same scope.
pub struct SharedStateComponent<C, SCOPE = PropHandler<<C as Component>::Properties>>
where
    C: Component,
    C::Properties: SharedState + Clone,
    PropHandler<C::Properties>: Clone,
    PropHandle<C::Properties>: WrapperHandle,
    SCOPE: 'static,
{
    props: C::Properties,
    bridge: Box<dyn Bridge<SharedStateService<PropHandler<C::Properties>, SCOPE>>>,
    link_set: bool,
    state_set: bool,
}

impl<C, SCOPE> Component for SharedStateComponent<C, SCOPE>
where
    C: Component,
    PropHandler<C::Properties>: Clone,
    C::Properties: SharedState + Clone,
    <C::Properties as SharedState>::Handle: Clone + WrapperHandle,
{
    type Message = SharedStateComponentMsg<C::Properties>;
    type Properties = C::Properties;

    fn create(mut props: Self::Properties, link: ComponentLink<Self>) -> Self {
        use SharedStateComponentMsg::*;
        // Bridge to receive new state.
        let callback = link.callback(|msg| match msg {
            Left(Response::State(state)) => SetLocal(state),
            Left(Response::Link(link)) => SetLink(link),
            Right(_) => Ignore,
        });
        let bridge = SharedStateService::bridge(callback);

        props
            .handle()
            .set_callbacks(link.callback(Apply), link.callback(ApplyOnce));

        SharedStateComponent {
            props,
            bridge,
            state_set: Default::default(),
            link_set: Default::default(),
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        use SharedStateComponentMsg::*;
        match msg {
            Apply(reduce) => {
                self.bridge.send(Left(Request::Apply(reduce)));
                false
            }
            ApplyOnce(reduce) => {
                self.bridge.send(Left(Request::ApplyOnce(reduce)));
                false
            }
            SetLocal(state) => {
                self.props.handle().set_state(state);
                self.state_set = true;
                true
            }
            SetLink(link) => {
                self.props.handle().set_link(link);
                self.link_set = true;
                true
            }
            Ignore => false,
        }
    }

    fn change(&mut self, mut props: Self::Properties) -> ShouldRender {
        *props.handle() = self.props.handle().clone();
        self.props = props;
        true
    }

    fn view(&self) -> Html {
        if self.link_set && self.state_set {
            let props = self.props.clone();
            html! {
                <C with props />
            }
        } else {
            html! {}
        }
    }
}
