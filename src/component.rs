//! Wrapper for components with shared state.
use std::collections::HashSet;
use std::rc::Rc;

use yew::{
    html,
    worker::{Agent, AgentLink, Bridge, Bridged, Context, HandlerId},
    Callback, Component, ComponentLink, Html, ShouldRender,
};

use super::handle::{Handle, SharedState};
use super::handler::{Handler, Reduction};

type StateHandler<T> = <<T as SharedState>::Handle as Handle>::Handler;
type Model<T> = <StateHandler<T> as Handler>::Model;

enum Request<T> {
    /// Apply a state change.
    Apply(Reduction<T>),
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
    T: SharedState + Clone + 'static,
{
    handler: StateHandler<T>,
    subscriptions: HashSet<HandlerId>,
    link: AgentLink<SharedStateService<T>>,
}

impl<T> Agent for SharedStateService<T>
where
    T: SharedState + Clone + 'static,
{
    type Message = ();
    type Reach = Context<Self>;
    type Input = Request<Model<T>>;
    type Output = Response<Model<T>>;

    fn create(link: AgentLink<Self>) -> Self {
        Self {
            handler: StateHandler::<T>::new(),
            subscriptions: Default::default(),
            link,
        }
    }

    fn update(&mut self, _msg: Self::Message) {}

    fn handle_input(&mut self, msg: Self::Input, who: HandlerId) {
        match msg {
            Request::Apply(reduce) => {
                self.handler.apply(reduce);
                for who in self.subscriptions.iter().cloned() {
                    self.link
                        .respond(who, Response::State(self.handler.state()));
                }
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

/// Wrapper for a component with shared state.
pub struct SharedStateComponent<C>
where
    C: Component,
    C::Properties: SharedState + Clone,
{
    props: C::Properties,
    bridge: Box<dyn Bridge<SharedStateService<C::Properties>>>,
    state: Rc<Model<C::Properties>>,
    callback: Callback<Reduction<Model<C::Properties>>>,
}

/// Internal use only.
#[doc(hidden)]
pub enum SharedStateComponentMsg<T> {
    /// Recieve new local state.
    /// IMPORTANT: Changes will **not** be reflected in shared state.
    SetLocal(Rc<T>),
    /// Update shared state.
    Apply(Reduction<T>),
}

impl<C> Component for SharedStateComponent<C>
where
    C: Component,
    C::Properties: SharedState + Clone,
    Model<C::Properties>: Default,
{
    type Message = SharedStateComponentMsg<Model<C::Properties>>;
    type Properties = C::Properties;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        use SharedStateComponentMsg::*;
        // Bridge to receive new state.
        let mut bridge = SharedStateService::bridge(link.callback(|msg| match msg {
            Response::State(state) => SetLocal(state),
        }));
        // Make sure we receive updates to state.
        bridge.send(Request::Subscribe);

        SharedStateComponent {
            props,
            bridge,
            state: Default::default(),
            callback: link.callback(Apply),
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        use SharedStateComponentMsg::*;
        match msg {
            Apply(reduce) => {
                self.bridge.send(Request::Apply(reduce));
                false
            }
            SetLocal(state) => {
                self.state = state;
                true
            }
        }
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props = props;
        true
    }

    fn view(&self) -> Html {
        let mut props = self.props.clone();
        props.handle().__set_local(&self.state, &self.callback);

        html! {
            <C with props />
        }
    }
}

impl<C> std::ops::Drop for SharedStateComponent<C>
where
    C: Component,
    C::Properties: SharedState + Clone,
{
    fn drop(&mut self) {
        self.bridge.send(Request::UnSubscribe);
    }
}
