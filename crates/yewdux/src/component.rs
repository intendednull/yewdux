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
    /// Receive new local state.
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
///
/// ## Modern Higher-Order Component Pattern
///
/// For improved SSR support and better integration with modern Yew applications,
/// a function component Higher-Order Component (HOC) pattern is recommended.
/// This pattern uses `YewduxRoot` for context and function components with hooks:
///
/// ```
/// use std::rc::Rc;
/// use yew::prelude::*;
/// use yewdux::prelude::*;
///
/// // 1. Define your store
/// #[derive(Default, Clone, PartialEq, Store)]
/// struct State {
///     count: u32,
/// }
///
/// // 2. Create props for your struct component
/// #[derive(Properties, Clone, PartialEq)]
/// struct CounterProps {
///     dispatch: Dispatch<State>,
/// }
///
/// // 3. Define a message type for state updates
/// enum Msg {
///     StateChanged(Rc<State>),
/// }
///
/// // 4. Create your struct component
/// struct Counter {
///     state: Rc<State>,
///     dispatch: Dispatch<State>,
/// }
///
/// impl Component for Counter {
///     type Properties = CounterProps;
///     type Message = Msg;
///
///     fn create(ctx: &Context<Self>) -> Self {
///         // Subscribe to state changes
///         let callback = ctx.link().callback(Msg::StateChanged);
///         let dispatch = ctx.props().dispatch.clone().subscribe_silent(callback);
///         
///         Self {
///             state: dispatch.get(),
///             dispatch,
///         }
///     }
///
///     fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
///         match msg {
///             Msg::StateChanged(state) => {
///                 self.state = state;
///                 true
///             }
///         }
///     }
///
///     fn view(&self, _ctx: &Context<Self>) -> Html {
///         let count = self.state.count;
///         let onclick = self.dispatch.reduce_mut_callback(|s| s.count += 1);
///         
///         html! {
///             <>
///                 <h1>{ count }</h1>
///                 <button {onclick}>{"+1"}</button>
///             </>
///         }
///     }
/// }
///
/// // 5. Create a Higher-Order Component (HOC) wrapper
/// #[function_component]
/// fn CounterHoc() -> Html {
///     // Use the hook to get the dispatch
///     let dispatch = use_dispatch::<State>();
///     
///     html! {
///         <Counter {dispatch} />
///     }
/// }
///
/// // 6. Use the HOC in your app with YewduxRoot
/// #[function_component]
/// fn App() -> Html {
///     html! {
///         <YewduxRoot>
///             <CounterHoc />
///         </YewduxRoot>
///     }
/// }
/// ```
///
/// This pattern:
/// - Uses `YewduxRoot` for better SSR support
/// - Provides a clean separation between state management and components
/// - Allows struct components to benefit from Yewdux's state management
/// - Makes testing easier by separating concerns
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
