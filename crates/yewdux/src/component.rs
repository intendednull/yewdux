//! Wrapper for components with shared state.
use std::rc::Rc;

use yew::prelude::*;

use crate::dispatch::{Dispatch, WithDispatchProps};
use crate::store::Store;

type PropStore<PROPS> = <PROPS as WithDispatchProps>::Store;
type Model<T> = <PropStore<T> as Store>::Model;

#[doc(hidden)]
pub enum Msg<PROPS>
where
    PROPS: WithDispatchProps,
{
    /// Recieve new local state.
    State(Rc<Model<PROPS>>),
}

/// Component wrapper for managing message passing.
///
/// Wraps any component with properties that implement
/// [DispatchProps](crate::dispatch::DispatchPropsMut):
/// ```
/// # use yewdux::component::WithDispatch;
/// # struct MyComponent;
/// pub type App = WithDispatch<MyComponent>;
/// ```
pub struct WithDispatch<C>
where
    C: Component,
    C::Properties: WithDispatchProps + Clone,
{
    state: Option<Rc<<<C::Properties as WithDispatchProps>::Store as Store>::Model>>,
}

impl<C> Component for WithDispatch<C>
where
    C: Component,
    C::Properties: WithDispatchProps + Clone,
{
    type Message = Msg<C::Properties>;
    type Properties = C::Properties;

    fn create(ctx: &Context<Self>) -> Self {
        *ctx.props().dispatch().dispatch.borrow_mut() =
            Dispatch::bridge_state(ctx.link().callback(Msg::State));
        Self {
            state: Default::default(),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        use Msg::*;
        match msg {
            State(state) => {
                self.state = Some(state);
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        // Dispatch is ready when both fields are set.
        // Only render wrapped component when we're ready.
        if let Some(state) = &self.state {
            let props = ctx.props().clone();
            *props.dispatch().state.borrow_mut() = Some(Rc::clone(state));
            html! {
                <C ..props />
            }
        } else {
            Default::default()
        }
    }
}
