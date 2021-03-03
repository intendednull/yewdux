//! Wrapper for components with shared state.
use std::rc::Rc;

use yew::prelude::*;

use crate::dispatch::{Dispatch, DispatchProp};
use crate::store::Store;

type PropStore<PROPS> = <PROPS as DispatchProp>::Store;
type Model<T> = <PropStore<T> as Store>::Model;

#[doc(hidden)]
pub enum Msg<PROPS>
where
    PROPS: DispatchProp,
{
    /// Recieve new local state.
    State(Rc<Model<PROPS>>),
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
pub struct WithDispatch<C>
where
    C: Component,
    C::Properties: DispatchProp + Clone,
{
    props: C::Properties,
    is_ready: bool,
}

impl<C> Component for WithDispatch<C>
where
    C: Component,
    C::Properties: DispatchProp + Clone,
{
    type Message = Msg<C::Properties>;
    type Properties = C::Properties;

    fn create(mut props: Self::Properties, link: ComponentLink<Self>) -> Self {
        *props.dispatch() = Dispatch::new(link.callback(Msg::State));
        Self {
            props,
            is_ready: false,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        use Msg::*;
        match msg {
            State(state) => {
                self.props.dispatch().state = Some(state);
                self.is_ready = true;
                true
            }
        }
    }

    fn change(&mut self, mut props: Self::Properties) -> ShouldRender {
        *props.dispatch() = self.props.dispatch().clone();
        self.props = props;
        true
    }

    fn view(&self) -> Html {
        // Dispatch is ready when both fields are set.
        // Only render wrapped component when we're ready.
        if self.is_ready {
            let props = self.props.clone();
            html! {
                <C with props />
            }
        } else {
            Default::default()
        }
    }
}
