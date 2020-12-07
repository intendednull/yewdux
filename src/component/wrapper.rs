//! Wrapper for components with shared state.
use std::rc::Rc;

use yew::{
    agent::{Bridge, Bridged},
    prelude::*,
};

use crate::handle::{SharedState, StateHandle, WrapperHandle};
use crate::handler::{HandlerLink, Reduction, ReductionOnce, StateHandler};
use crate::service::*;

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
    bridge: Box<dyn Bridge<StateService<PropHandler<C::Properties>, SCOPE>>>,
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
            ServiceOutput::Service(ServiceResponse::State(state)) => SetLocal(state),
            ServiceOutput::Service(ServiceResponse::Link(link)) => SetLink(link),
            ServiceOutput::Handler(_) => Ignore,
        });
        let bridge = StateService::bridge(callback);

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
