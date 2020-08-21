//! Wrapper for components with shared state.
use std::collections::HashSet;
use std::rc::Rc;

use yew::{
    html,
    worker::{Agent, AgentLink, Bridge, Bridged, Context, HandlerId},
    Component, ComponentLink, Html, ShouldRender,
};

use super::handle::{Handle, SharedState};
use super::handler::{Handler, Reduction, ReductionOnce};

enum Request<T> {
    /// Apply a state change.
    Apply(Reduction<T>),
    /// Apply a state change once.
    ApplyOnce(ReductionOnce<T>),
    /// Subscribe to be notified when state changes.
    Subscribe,
    /// Remove subscription.
    UnSubscribe,
}

enum Response<T> {
    /// Update subscribers with current state.
    State(Rc<T>),
}

/// Context agent for managing shared state. In charge of applying changes to state then notifying
/// subscribers of new state.
struct SharedStateService<T>
where
    T: Handler + Clone + 'static,
{
    handler: T,
    subscriptions: HashSet<HandlerId>,
    link: AgentLink<SharedStateService<T>>,
}

impl<T> Agent for SharedStateService<T>
where
    T: Handler + Clone + 'static,
{
    type Message = ();
    type Reach = Context<Self>;
    type Input = Request<<T as Handler>::Model>;
    type Output = Response<<T as Handler>::Model>;

    fn create(link: AgentLink<Self>) -> Self {
        Self {
            handler: <T as Handler>::new(),
            subscriptions: Default::default(),
            link,
        }
    }

    fn update(&mut self, _msg: Self::Message) {}

    fn handle_input(&mut self, msg: Self::Input, who: HandlerId) {
        match msg {
            Request::Apply(reduce) => {
                self.handler.apply(reduce);
                self.notify_subscibers();
            }
            Request::ApplyOnce(reduce) => {
                self.handler.apply_once(reduce);
                self.notify_subscibers();
            }
            Request::Subscribe => {
                self.subscriptions.insert(who);
                self.link
                    .respond(who, Response::State(self.handler.state()));
            }
            Request::UnSubscribe => {
                self.subscriptions.remove(&who);
            }
        }
    }
}

impl<T> SharedStateService<T>
where
    T: Handler + Clone + 'static,
{
    fn notify_subscibers(&self) {
        for who in self.subscriptions.iter().cloned() {
            self.link
                .respond(who, Response::State(self.handler.state()));
        }
    }
}

type StateHandler<T> = <<T as SharedState>::Handle as Handle>::Handler;
type Model<T> = <StateHandler<T> as Handler>::Model;

/// Wrapper for a component with shared state.
pub struct SharedStateComponent<C>
where
    C: Component,
    C::Properties: SharedState + Clone,
    StateHandler<C::Properties>: Clone,
{
    props: C::Properties,
    bridge: Box<dyn Bridge<SharedStateService<StateHandler<C::Properties>>>>,
}

/// Internal use only.
#[doc(hidden)]
pub enum SharedStateComponentMsg<T> {
    /// Recieve new local state.
    /// IMPORTANT: Changes will **not** be reflected in shared state.
    SetLocal(Rc<T>),
    /// Update shared state.
    Apply(Reduction<T>),
    ApplyOnce(ReductionOnce<T>),
}

impl<C> Component for SharedStateComponent<C>
where
    C: Component,
    C::Properties: SharedState + Clone,
    Model<C::Properties>: Default,
    StateHandler<C::Properties>: Clone,
{
    type Message = SharedStateComponentMsg<Model<C::Properties>>;
    type Properties = C::Properties;

    fn create(mut props: Self::Properties, link: ComponentLink<Self>) -> Self {
        use SharedStateComponentMsg::*;
        // Bridge to receive new state.
        let mut bridge = SharedStateService::bridge(link.callback(|msg| match msg {
            Response::State(state) => SetLocal(state),
        }));
        // Make sure we receive updates to state.
        bridge.send(Request::Subscribe);

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

impl<C> std::ops::Drop for SharedStateComponent<C>
where
    C: Component,
    C::Properties: SharedState + Clone,
    StateHandler<C::Properties>: Clone,
{
    fn drop(&mut self) {
        self.bridge.send(Request::UnSubscribe);
    }
}
