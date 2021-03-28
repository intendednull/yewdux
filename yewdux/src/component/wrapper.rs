//! Wrapper for components with shared state.
use std::rc::Rc;

use yew::prelude::*;

use crate::dispatch::{DispatchProps, DispatchPropsMut};
use crate::store::Store;

type PropStore<PROPS> = <PROPS as DispatchPropsMut>::Store;
type Model<T> = <PropStore<T> as Store>::Model;

#[doc(hidden)]
pub enum Msg<PROPS>
where
    PROPS: DispatchPropsMut,
{
    /// Recieve new local state.
    State(Rc<Model<PROPS>>),
}

/// Component wrapper for managing message passing.
///
/// Wraps any component with properties that implement
/// [DispatchProps](crate::dispatch::DispatchPropsMut):
/// ```
/// pub type App = WithDispatch<MyComponent>;
/// ```
pub struct WithDispatch<C>
where
    C: Component,
    C::Properties: DispatchPropsMut + Clone,
{
    props: C::Properties,
    is_ready: bool,
}

impl<C> Component for WithDispatch<C>
where
    C: Component,
    C::Properties: DispatchPropsMut + Clone,
{
    type Message = Msg<C::Properties>;
    type Properties = C::Properties;

    fn create(mut props: Self::Properties, link: ComponentLink<Self>) -> Self {
        *props.dispatch() = DispatchProps::new(link.callback(Msg::State));
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
