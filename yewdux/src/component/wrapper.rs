//! Wrapper for components with shared state.
use std::rc::Rc;

use yew::prelude::*;

use crate::dispatch::{Dispatch, DispatchProps, Dispatched};
use crate::store::Store;

type PropStore<PROPS> = <PROPS as Dispatched>::Store;
type Model<T> = <PropStore<T> as Store>::Model;

#[doc(hidden)]
pub enum Msg<PROPS>
where
    PROPS: Dispatched,
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
    C::Properties: Dispatched + Clone,
{
    is_ready: bool,
    _mark: std::marker::PhantomData<C>,
}

impl<C> Component for WithDispatch<C>
where
    C: Component,
    C::Properties: Dispatched + Clone,
{
    type Message = Msg<C::Properties>;
    type Properties = C::Properties;

    fn create(ctx: &Context<Self>) -> Self {
        *ctx.props().dispatch().dispatch.borrow_mut() =
            Dispatch::bridge_state(ctx.link().callback(Msg::State));
        Self {
            is_ready: false,
            _mark: Default::default(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        use Msg::*;
        match msg {
            State(state) => {
                *ctx.props().dispatch().state.borrow_mut() = Some(state);
                self.is_ready = true;
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        // Dispatch is ready when both fields are set.
        // Only render wrapped component when we're ready.
        if self.is_ready {
            let props = ctx.props().clone();
            html! {
                <C with props />
            }
        } else {
            Default::default()
        }
    }
}
