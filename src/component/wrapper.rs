//! Wrapper for components with shared state.
use std::collections::HashSet;
use std::rc::Rc;

use yew::{
    agent::{Agent, AgentLink, Bridge, Bridged, Context, HandlerId},
    prelude::*,
};

use crate::handle::{Handle, SharedState};
use crate::handler::{Reduction, ReductionOnce, StateHandler};

enum Request<T> {
    /// Apply a state change.
    Apply(Reduction<T>),
    /// Apply a state change once.
    ApplyOnce(ReductionOnce<T>),
}

enum Response<T> {
    /// Update subscribers with current state.
    State(Rc<T>),
}

/// Context agent for managing shared state. In charge of applying changes to state then notifying
/// subscribers of new state.
struct SharedStateService<HANDLER, SCOPE>
where
    HANDLER: StateHandler + Clone + 'static,
    SCOPE: 'static,
{
    handler: HANDLER,
    subscriptions: HashSet<HandlerId>,
    link: AgentLink<SharedStateService<HANDLER, SCOPE>>,
}

impl<HANDLER, SCOPE> Agent for SharedStateService<HANDLER, SCOPE>
where
    HANDLER: StateHandler + Clone + 'static,
    SCOPE: 'static,
{
    type Message = HANDLER::Message;
    type Reach = Context<Self>;
    type Input = Request<HANDLER::Model>;
    type Output = Response<HANDLER::Model>;

    fn create(link: AgentLink<Self>) -> Self {
        Self {
            handler: <HANDLER as StateHandler>::new(),
            subscriptions: Default::default(),
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

    fn handle_input(&mut self, msg: Self::Input, _who: HandlerId) {
        match msg {
            Request::Apply(reduce) => {
                reduce(Rc::make_mut(self.handler.state()));
                self.handler.changed();
            }
            Request::ApplyOnce(reduce) => {
                reduce(Rc::make_mut(self.handler.state()));
                self.handler.changed();
            }
        }

        self.notify_subscribers();
    }

    fn connected(&mut self, who: HandlerId) {
        // Add component to subscriptions.
        self.subscriptions.insert(who);
        // Send it current state.
        let state = Rc::clone(self.handler.state());
        self.link.respond(who, Response::State(state));
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
            self.link.respond(who, Response::State(Rc::clone(state)));
        }
    }
}

type PropHandler<T> = <<T as SharedState>::Handle as Handle>::Handler;
type Model<T> = <PropHandler<T> as StateHandler>::Model;

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
    SCOPE: 'static,
{
    props: C::Properties,
    bridge: Box<dyn Bridge<SharedStateService<PropHandler<C::Properties>, SCOPE>>>,
}

#[doc(hidden)]
pub enum SharedStateComponentMsg<T> {
    /// Recieve new local state.
    /// IMPORTANT: Changes will **not** be reflected in shared state.
    SetLocal(Rc<T>),
    /// Update shared state.
    Apply(Reduction<T>),
    ApplyOnce(ReductionOnce<T>),
}

impl<C, SCOPE> Component for SharedStateComponent<C, SCOPE>
where
    C: Component,
    C::Properties: SharedState + Clone,
    Model<C::Properties>: Default,
    PropHandler<C::Properties>: Clone,
{
    type Message = SharedStateComponentMsg<Model<C::Properties>>;
    type Properties = C::Properties;

    fn create(mut props: Self::Properties, link: ComponentLink<Self>) -> Self {
        use SharedStateComponentMsg::*;
        // Bridge to receive new state.
        let callback = link.callback(|msg| match msg {
            Response::State(state) => SetLocal(state),
        });
        let bridge = SharedStateService::bridge(callback);

        props
            .handle()
            .set_local_callback(link.callback(Apply), link.callback(ApplyOnce));

        SharedStateComponent { props, bridge }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        use SharedStateComponentMsg::*;
        match msg {
            Apply(reduce) => {
                self.bridge.send(Request::Apply(reduce));
                false
            }
            ApplyOnce(reduce) => {
                self.bridge.send(Request::ApplyOnce(reduce));
                false
            }
            SetLocal(state) => {
                self.props.handle().set_local_state(state);
                true
            }
        }
    }

    fn change(&mut self, mut props: Self::Properties) -> ShouldRender {
        props.handle().set_local(self.props.handle());
        self.props = props;
        true
    }

    fn view(&self) -> Html {
        let props = self.props.clone();
        html! {
            <C with props />
        }
    }
}
