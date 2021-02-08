//! Wrapper for components with shared state.
use std::rc::Rc;

use yew::{
    agent::{Bridge, Bridged},
    prelude::*,
};

use crate::dispatcher::{Dispatcher, DispatcherProp};
use crate::service::*;
use crate::store::{Reduction, ReductionOnce, Store, StoreLink};

type PropStore<PROPS> = <PROPS as DispatcherProp>::Store;
type Model<T> = <PropStore<T> as Store>::Model;

/// Provides mutable access for wrapper component to update
trait DispatcherMut: DispatcherProp {
    fn set_state(&mut self, state: Rc<<Self::Store as Store>::Model>);
    fn set_callbacks(
        &mut self,
        callback: Callback<Reduction<<Self::Store as Store>::Model>>,
        callback_once: Callback<ReductionOnce<<Self::Store as Store>::Model>>,
    );
    fn set_link(&mut self, _link: StoreLink<Self::Store>) {}
}

impl<STORE> DispatcherMut for Dispatcher<STORE>
where
    STORE: Store,
{
    fn set_state(&mut self, state: Rc<<Self::Store as Store>::Model>) {
        self.state = Some(state);
    }

    fn set_link(&mut self, link: StoreLink<Self::Store>) {
        self.link = Some(link);
    }

    fn set_callbacks(
        &mut self,
        callback: Callback<Reduction<<Self::Store as Store>::Model>>,
        callback_once: Callback<ReductionOnce<<Self::Store as Store>::Model>>,
    ) {
        self.callback = callback;
        self.callback_once = callback_once;
    }
}

#[doc(hidden)]
pub enum Msg<PROPS>
where
    PROPS: DispatcherProp,
{
    /// Recieve new local state.
    /// IMPORTANT: Changes will **not** be reflected in shared state.
    SetLocal(Rc<Model<PROPS>>),
    SetLink(StoreLink<PropStore<PROPS>>),
    /// Update shared state.
    Apply(Reduction<Model<PROPS>>),
    ApplyOnce(ReductionOnce<Model<PROPS>>),
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
pub struct WithDispatcher<C, SCOPE = PropStore<<C as Component>::Properties>>
where
    C: Component,
    C::Properties: DispatcherProp + Clone,
    SCOPE: 'static,
{
    props: C::Properties,
    bridge: Box<dyn Bridge<StoreService<PropStore<C::Properties>, SCOPE>>>,
    link_set: bool,
    state_set: bool,
}

impl<C, SCOPE> Component for WithDispatcher<C, SCOPE>
where
    C: Component,
    C::Properties: DispatcherProp + Clone,
{
    type Message = Msg<C::Properties>;
    type Properties = C::Properties;

    fn create(mut props: Self::Properties, link: ComponentLink<Self>) -> Self {
        use Msg::*;
        // Bridge to receive new state.
        let callback = link.callback(|msg| match msg {
            ServiceOutput::Service(ServiceResponse::State(state)) => SetLocal(state),
            ServiceOutput::Service(ServiceResponse::Link(link)) => SetLink(link),
            ServiceOutput::Store(_) => Ignore,
        });
        let mut bridge = StoreService::bridge(callback);
        // Subscribe to state changes.
        bridge.send(ServiceInput::Service(ServiceRequest::Subscribe));
        // Connect our component callbacks.
        props
            .dispatcher()
            .set_callbacks(link.callback(Apply), link.callback(ApplyOnce));

        Self {
            props,
            bridge,
            state_set: Default::default(),
            link_set: Default::default(),
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        use Msg::*;
        match msg {
            Apply(reduce) => {
                self.bridge
                    .send(ServiceInput::Service(ServiceRequest::Apply(reduce)));
                false
            }
            ApplyOnce(reduce) => {
                self.bridge
                    .send(ServiceInput::Service(ServiceRequest::ApplyOnce(reduce)));
                false
            }
            SetLocal(state) => {
                self.props.dispatcher().set_state(state);
                self.state_set = true;
                true
            }
            SetLink(link) => {
                self.props.dispatcher().set_link(link);
                self.link_set = true;
                true
            }
            Ignore => false,
        }
    }

    fn change(&mut self, mut props: Self::Properties) -> ShouldRender {
        *props.dispatcher() = self.props.dispatcher().clone();
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
            Default::default()
        }
    }
}
