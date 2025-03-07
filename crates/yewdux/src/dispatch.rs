//!  This module defines how you can interact with your [`Store`].
//!
//!  ```
//!  use yewdux::prelude::*;
//!
//!  #[derive(Default, Clone, PartialEq, Store)]
//!  struct State {
//!     count: usize,
//!  }
//!
//!  // Create a context - in a real application, you'd typically get this from a parent component
//!  let cx = yewdux::Context::new();
//!  let dispatch = Dispatch::<State>::new(&cx);
//!  
//!  // Update the state
//!  dispatch.reduce_mut(|state| state.count = 1);
//!
//!  // Get the current state
//!  let state = dispatch.get();
//!
//!  assert!(state.count == 1);
//!  ```
//!
//! ## Usage with YewduxRoot
//!
//! For applications with server-side rendering (SSR) support, the recommended
//! approach is to use `YewduxRoot` to provide context:
//!
//! ```
//! use std::rc::Rc;
//! use yew::prelude::*;
//! use yewdux::prelude::*;
//!
//! // Define your store
//! #[derive(Default, Clone, PartialEq, Store)]
//! struct State {
//!     count: u32,
//! }
//!
//! // Function component using hooks to access state
//! #[function_component]
//! fn Counter() -> Html {
//!     // Get both state and dispatch from the context
//!     let (state, dispatch) = use_store::<State>();
//!     let onclick = dispatch.reduce_mut_callback(|state| state.count += 1);
//!     
//!     html! {
//!         <>
//!             <p>{ state.count }</p>
//!             <button {onclick}>{"+1"}</button>
//!         </>
//!     }
//! }
//!
//! // Root component that sets up the YewduxRoot context
//! #[function_component]
//! fn App() -> Html {
//!     html! {
//!         <YewduxRoot>
//!             <Counter />
//!         </YewduxRoot>
//!     }
//! }
//! ```
//!

use std::{future::Future, rc::Rc};

use yew::Callback;

use crate::{
    context::Context,
    store::{Reducer, Store},
    subscriber::{Callable, SubscriberId},
};

/// The primary interface to a [`Store`].
pub struct Dispatch<S: Store> {
    pub(crate) _subscriber_id: Option<Rc<SubscriberId<S>>>,
    pub(crate) cx: Context,
}

impl<S: Store> std::fmt::Debug for Dispatch<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Dispatch")
            .field("_subscriber_id", &self._subscriber_id)
            .finish()
    }
}

#[cfg(any(doc, feature = "doctests", target_arch = "wasm32"))]
impl<S: Store> Default for Dispatch<S> {
    fn default() -> Self {
        Self::global()
    }
}

impl<S: Store> Dispatch<S> {
    /// Create a new dispatch with the global context (thread local).
    ///
    /// This is only available for wasm. For SSR, see the YewduxRoot pattern.
    #[cfg(any(doc, feature = "doctests", target_arch = "wasm32"))]
    pub fn global() -> Self {
        Self::new(&Context::global())
    }

    /// Create a new dispatch with access to the given context.
    pub fn new(cx: &Context) -> Self {
        Self {
            _subscriber_id: Default::default(),
            cx: cx.clone(),
        }
    }

    /// Get the context used by this dispatch.
    pub fn context(&self) -> &Context {
        &self.cx
    }

    /// Spawn a future with access to this dispatch.
    #[cfg(feature = "future")]
    pub fn spawn_future<F, FU>(&self, f: F)
    where
        F: FnOnce(Self) -> FU,
        FU: Future<Output = ()> + 'static,
    {
        yew::platform::spawn_local(f(self.clone()));
    }

    /// Create a callback that will spawn a future with access to this dispatch.
    #[cfg(feature = "future")]
    pub fn future_callback<E, F, FU>(&self, f: F) -> Callback<E>
    where
        F: Fn(Self) -> FU + 'static,
        FU: Future<Output = ()> + 'static,
    {
        let dispatch = self.clone();
        Callback::from(move |_| dispatch.spawn_future(&f))
    }

    /// Create a callback that will spawn a future with access to this dispatch and the emitted
    /// event.
    #[cfg(feature = "future")]
    pub fn future_callback_with<E, F, FU>(&self, f: F) -> Callback<E>
    where
        F: Fn(Self, E) -> FU + 'static,
        FU: Future<Output = ()> + 'static,
    {
        let dispatch = self.clone();
        Callback::from(move |e| dispatch.spawn_future(|dispatch| f(dispatch, e)))
    }

    /// Create a dispatch that subscribes to changes in state. Latest state is sent immediately,
    /// and on every subsequent change. Automatically unsubscribes when this dispatch is dropped.
    /// 
    /// ## Higher-Order Component Pattern with YewduxRoot
    /// 
    /// ```
    /// use std::rc::Rc;
    ///
    /// use yew::prelude::*;
    /// use yewdux::prelude::*;
    ///
    /// #[derive(Default, Clone, PartialEq, Eq, Store)]
    /// struct State {
    ///     count: u32,
    /// }
    ///
    /// // Props for our struct component
    /// #[derive(Properties, PartialEq, Clone)]
    /// struct CounterProps {
    ///     dispatch: Dispatch<State>,
    /// }
    ///
    /// // Message type for state updates
    /// enum Msg {
    ///     StateChanged(Rc<State>),
    /// }
    ///
    /// // Our struct component that uses the state
    /// struct Counter {
    ///     state: Rc<State>,
    ///     dispatch: Dispatch<State>,
    /// }
    ///
    /// impl Component for Counter {
    ///     type Message = Msg;
    ///     type Properties = CounterProps;
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
    /// // Higher-Order Component (HOC) that accesses the context
    /// #[function_component]
    /// fn CounterHoc() -> Html {
    ///     // Use the hook to get the dispatch from context
    ///     let dispatch = use_dispatch::<State>();
    ///     
    ///     html! {
    ///         <Counter {dispatch} />
    ///     }
    /// }
    ///
    /// // App component with YewduxRoot for SSR support
    /// #[function_component]
    /// fn App() -> Html {
    ///     html! {
    ///         <YewduxRoot>
    ///             <CounterHoc />
    ///         </YewduxRoot>
    ///     }
    /// }
    /// ```
    pub fn subscribe<C: Callable<S>>(self, on_change: C) -> Self {
        let id = self.cx.subscribe(on_change);

        Self {
            _subscriber_id: Some(Rc::new(id)),
            cx: self.cx,
        }
    }

    /// Create a dispatch that subscribes to changes in state. Similar to [Self::subscribe],
    /// however state is **not** sent immediately. Automatically unsubscribes when this dispatch is
    /// dropped.
    pub fn subscribe_silent<C: Callable<S>>(self, on_change: C) -> Self {
        let id = self.cx.subscribe_silent(on_change);

        Self {
            _subscriber_id: Some(Rc::new(id)),
            cx: self.cx,
        }
    }

    /// Get the current state.
    pub fn get(&self) -> Rc<S> {
        self.cx.get::<S>()
    }

    /// Apply a [`Reducer`](crate::store::Reducer) immediately.
    ///
    /// ```
    /// # use std::rc::Rc;
    /// # use yew::prelude::*;
    /// # use yewdux::prelude::*;
    /// #[derive(Default, Clone, PartialEq, Eq, Store)]
    /// struct State {
    ///     count: u32,
    /// }
    ///
    /// struct AddOne;
    /// impl Reducer<State> for AddOne {
    ///     fn apply(self, state: Rc<State>) -> Rc<State> {
    ///         State {
    ///             count: state.count + 1,
    ///         }
    ///         .into()
    ///     }
    /// }
    ///
    /// # fn main() {
    /// # // Context handling code is omitted for clarity
    /// # let cx = yewdux::Context::new();
    /// # let dispatch = Dispatch::<State>::new(&cx);
    /// // Apply a reducer to update the state
    /// dispatch.apply(AddOne);
    /// # ;
    /// # }
    /// ```
    pub fn apply<R: Reducer<S>>(&self, reducer: R) {
        self.cx.reduce(reducer);
    }

    /// Create a callback that applies a [`Reducer`](crate::store::Reducer).
    ///
    /// ```
    /// # use std::rc::Rc;
    /// # use yew::prelude::*;
    /// # use yewdux::prelude::*;
    /// #[derive(Default, Clone, PartialEq, Eq, Store)]
    /// struct State {
    ///     count: u32,
    /// }
    ///
    /// struct AddOne;
    /// impl Reducer<State> for AddOne {
    ///     fn apply(self, state: Rc<State>) -> Rc<State> {
    ///         State {
    ///             count: state.count + 1,
    ///         }
    ///         .into()
    ///     }
    /// }
    ///
    /// # fn main() {
    /// # // Context handling code is omitted for clarity
    /// # let cx = yewdux::Context::new();
    /// # let dispatch = Dispatch::<State>::new(&cx);
    /// // Create a callback that will update the state when triggered
    /// let onclick = dispatch.apply_callback(|_| AddOne);
    /// html! {
    ///     <button {onclick}>{"+1"}</button>
    /// }
    /// # ;
    /// # }
    /// ```
    pub fn apply_callback<E, M, F>(&self, f: F) -> Callback<E>
    where
        M: Reducer<S>,
        F: Fn(E) -> M + 'static,
    {
        let context = self.cx.clone();
        Callback::from(move |e| {
            let msg = f(e);
            context.reduce(msg);
        })
    }

    /// Set state to given value immediately.
    ///
    /// ```
    /// # use yew::prelude::*;
    /// # use yewdux::prelude::*;
    /// # #[derive(Default, Clone, PartialEq, Eq, Store)]
    /// # struct State {
    /// #     count: u32,
    /// # }
    /// # fn main() {
    /// # // Context handling code is omitted for clarity
    /// # let cx = yewdux::Context::new();
    /// # let dispatch = Dispatch::<State>::new(&cx);
    /// // Set the state to a new value
    /// dispatch.set(State { count: 0 });
    /// # }
    /// ```
    pub fn set(&self, val: S) {
        self.cx.set(val);
    }

    /// Set state using value from callback.
    ///
    /// ```
    /// # use yew::prelude::*;
    /// # use yewdux::prelude::*;
    /// # #[derive(Default, Clone, PartialEq, Eq, Store)]
    /// # struct State {
    /// #     count: u32,
    /// # }
    /// # #[hook]
    /// # fn use_foo() {
    /// let dispatch = use_dispatch::<State>();
    /// let onchange = dispatch.set_callback(|event: Event| {
    ///     let value = event.target_unchecked_into::<web_sys::HtmlInputElement>().value();
    ///     State { count: value.parse().unwrap() }
    /// });
    /// html! {
    ///     <input type="number" placeholder="Enter a number" {onchange}  />
    /// }
    /// # ;
    /// # }
    /// ```
    pub fn set_callback<E, F>(&self, f: F) -> Callback<E>
    where
        F: Fn(E) -> S + 'static,
    {
        let context = self.cx.clone();
        Callback::from(move |e| {
            let val = f(e);
            context.set(val);
        })
    }

    /// Change state immediately.
    ///
    /// ```
    /// # use yew::prelude::*;
    /// # use yewdux::prelude::*;
    /// # #[derive(Default, Clone, PartialEq, Eq, Store)]
    /// # struct State {
    /// #     count: u32,
    /// # }
    /// # fn main() {
    /// # // Context handling code is omitted for clarity
    /// # let cx = yewdux::Context::new();
    /// # let dispatch = Dispatch::<State>::new(&cx);
    /// // Transform the current state into a new state
    /// dispatch.reduce(|state| State { count: state.count + 1 }.into());
    /// # }
    /// ```
    pub fn reduce<F>(&self, f: F)
    where
        F: FnOnce(Rc<S>) -> Rc<S>,
    {
        self.cx.reduce(f);
    }

    /// Create a callback that changes state.
    ///
    /// ```
    /// # use yew::prelude::*;
    /// # use yewdux::prelude::*;
    /// # #[derive(Default, Clone, PartialEq, Eq, Store)]
    /// # struct State {
    /// #     count: u32,
    /// # }
    /// # fn main() {
    /// # // Context handling code is omitted for clarity
    /// # let cx = yewdux::Context::new();
    /// # let dispatch = Dispatch::<State>::new(&cx);
    /// // Create a callback that will transform the state when triggered
    /// let onclick = dispatch.reduce_callback(|state| State { count: state.count + 1 }.into());
    /// html! {
    ///     <button {onclick}>{"+1"}</button>
    /// }
    /// # ;
    /// # }
    /// ```
    pub fn reduce_callback<F, E>(&self, f: F) -> Callback<E>
    where
        F: Fn(Rc<S>) -> Rc<S> + 'static,
        E: 'static,
    {
        let context = self.cx.clone();
        Callback::from(move |_| {
            context.reduce(&f);
        })
    }

    /// Similar to [Self::reduce_callback] but also provides the fired event.
    ///
    /// ```
    /// # use yew::prelude::*;
    /// # use yewdux::prelude::*;
    /// # #[derive(Default, Clone, PartialEq, Eq, Store)]
    /// # struct State {
    /// #     count: u32,
    /// # }
    /// # fn main() {
    /// # // Context handling code is omitted for clarity
    /// # let cx = yewdux::Context::new();
    /// # let dispatch = Dispatch::<State>::new(&cx);
    /// // Create a callback that will transform the state using the event data
    /// let onchange = dispatch.reduce_callback_with(|state, event: Event| {
    ///     let value = event.target_unchecked_into::<web_sys::HtmlInputElement>().value();
    ///     State {
    ///         count: value.parse().unwrap()
    ///     }
    ///     .into()
    /// });
    /// html! {
    ///     <input type="number" placeholder="Enter a number" {onchange}  />
    /// }
    /// # ;
    /// # }
    /// ```
    pub fn reduce_callback_with<F, E>(&self, f: F) -> Callback<E>
    where
        F: Fn(Rc<S>, E) -> Rc<S> + 'static,
        E: 'static,
    {
        let context = self.cx.clone();
        Callback::from(move |e: E| {
            context.reduce(|x| f(x, e));
        })
    }

    /// Mutate state with given function.
    ///
    /// ```
    /// # use yew::prelude::*;
    /// # use yewdux::prelude::*;
    /// # #[derive(Default, Clone, PartialEq, Eq, Store)]
    /// # struct State {
    /// #     count: u32,
    /// # }
    /// # fn main() {
    /// # // Context handling code is omitted for clarity
    /// # let cx = yewdux::Context::new();
    /// # let dispatch = Dispatch::<State>::new(&cx);
    /// // Mutate the state in-place
    /// dispatch.reduce_mut(|state| state.count += 1);
    /// # }
    /// ```
    pub fn reduce_mut<F, R>(&self, f: F) -> R
    where
        S: Clone,
        F: FnOnce(&mut S) -> R,
    {
        let mut result = None;

        self.cx.reduce_mut(|x| {
            result = Some(f(x));
        });

        result.expect("result not initialized")
    }

    /// Like [Self::reduce_mut] but from a callback.
    ///
    /// ```
    /// # use yew::prelude::*;
    /// # use yewdux::prelude::*;
    /// # #[derive(Default, Clone, PartialEq, Eq, Store)]
    /// # struct State {
    /// #     count: u32,
    /// # }
    /// # fn main() {
    /// # // Context handling code is omitted for clarity
    /// # let cx = yewdux::Context::new();
    /// # let dispatch = Dispatch::<State>::new(&cx);
    /// // Create a callback that will mutate the state in-place when triggered
    /// let onclick = dispatch.reduce_mut_callback(|s| s.count += 1);
    /// html! {
    ///     <button {onclick}>{"+1"}</button>
    /// }
    /// # ;
    /// # }
    /// ```
    pub fn reduce_mut_callback<F, R, E>(&self, f: F) -> Callback<E>
    where
        S: Clone,
        F: Fn(&mut S) -> R + 'static,
        E: 'static,
    {
        let context = self.cx.clone();
        Callback::from(move |_| {
            context.reduce_mut(|x| {
                f(x);
            });
        })
    }

    /// Similar to [Self::reduce_mut_callback] but also provides the fired event.
    ///
    /// ```
    /// # use yew::prelude::*;
    /// # use yewdux::prelude::*;
    /// # #[derive(Default, Clone, PartialEq, Eq, Store)]
    /// # struct State {
    /// #     count: u32,
    /// # }
    /// # fn main() {
    /// # // Context handling code is omitted for clarity
    /// # let cx = yewdux::Context::new();
    /// # let dispatch = Dispatch::<State>::new(&cx);
    /// // Create a callback that will mutate the state using event data
    /// let onchange = dispatch.reduce_mut_callback_with(|state, event: Event| {
    ///     let value = event.target_unchecked_into::<web_sys::HtmlInputElement>().value();
    ///     state.count = value.parse().unwrap();
    /// });
    /// html! {
    ///     <input type="number" placeholder="Enter a number" {onchange}  />
    /// }
    /// # ;
    /// # }
    /// ```
    pub fn reduce_mut_callback_with<F, R, E>(&self, f: F) -> Callback<E>
    where
        S: Clone,
        F: Fn(&mut S, E) -> R + 'static,
        E: 'static,
    {
        let context = self.cx.clone();
        Callback::from(move |e: E| {
            context.reduce_mut(|x| {
                f(x, e);
            });
        })
    }
}

impl<S: Store> Clone for Dispatch<S> {
    fn clone(&self) -> Self {
        Self {
            _subscriber_id: self._subscriber_id.clone(),
            cx: self.cx.clone(),
        }
    }
}

impl<S: Store> PartialEq for Dispatch<S> {
    fn eq(&self, other: &Self) -> bool {
        match (&self._subscriber_id, &other._subscriber_id) {
            (Some(a), Some(b)) => Rc::ptr_eq(a, b),
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::{mrc::Mrc, subscriber::Subscribers};

    use super::*;

    #[derive(Clone, PartialEq, Eq)]
    struct TestState(u32);
    impl Store for TestState {
        fn new(_cx: &Context) -> Self {
            Self(0)
        }

        fn should_notify(&self, other: &Self) -> bool {
            self != other
        }
    }
    #[derive(PartialEq, Eq)]
    struct TestStateNoClone(u32);
    impl Store for TestStateNoClone {
        fn new(_cx: &Context) -> Self {
            Self(0)
        }

        fn should_notify(&self, other: &Self) -> bool {
            self != other
        }
    }

    struct Msg;
    impl Reducer<TestState> for Msg {
        fn apply(self, state: Rc<TestState>) -> Rc<TestState> {
            TestState(state.0 + 1).into()
        }
    }

    #[test]
    fn apply_no_clone() {
        Dispatch::new(&Context::new()).reduce(|_| TestStateNoClone(1).into());
    }

    #[test]
    fn reduce_changes_value() {
        let dispatch = Dispatch::<TestState>::new(&Context::new());

        let old = dispatch.get();

        dispatch.reduce(|_| TestState(1).into());

        let new = dispatch.get();

        assert!(old != new);
    }

    #[test]
    fn reduce_mut_changes_value() {
        let dispatch = Dispatch::<TestState>::new(&Context::new());
        let old = dispatch.get();

        dispatch.reduce_mut(|state| *state = TestState(1));

        let new = dispatch.get();

        assert!(old != new);
    }

    #[test]
    fn reduce_does_not_require_static() {
        let val = "1".to_string();
        Dispatch::new(&Context::new()).reduce(|_| TestState(val.parse().unwrap()).into());
    }

    #[test]
    fn reduce_mut_does_not_require_static() {
        let val = "1".to_string();
        Dispatch::new(&Context::new())
            .reduce_mut(|state: &mut TestState| state.0 = val.parse().unwrap());
    }

    #[test]
    fn set_changes_value() {
        let dispatch = Dispatch::<TestState>::new(&Context::new());

        let old = dispatch.get();

        dispatch.set(TestState(1));

        let new = dispatch.get();

        assert!(old != new);
    }

    #[test]
    fn apply_changes_value() {
        let dispatch = Dispatch::<TestState>::new(&Context::new());
        let old = dispatch.get();

        dispatch.apply(Msg);

        let new = dispatch.get();

        assert!(old != new);
    }

    #[test]
    fn dispatch_set_works() {
        let dispatch = Dispatch::<TestState>::new(&Context::new());
        let old = dispatch.get();

        dispatch.set(TestState(1));

        assert!(dispatch.get() != old)
    }

    #[test]
    fn dispatch_set_callback_works() {
        let dispatch = Dispatch::<TestState>::new(&Context::new());
        let old = dispatch.get();

        let cb = dispatch.set_callback(|_| TestState(1));
        cb.emit(());

        assert!(dispatch.get() != old)
    }

    #[test]
    fn dispatch_reduce_mut_works() {
        let dispatch = Dispatch::<TestState>::new(&Context::new());
        let old = dispatch.get();

        dispatch.reduce_mut(|state| state.0 += 1);

        assert!(dispatch.get() != old)
    }

    #[test]
    fn dispatch_reduce_works() {
        let dispatch = Dispatch::<TestState>::new(&Context::new());
        let old = dispatch.get();

        dispatch.reduce(|_| TestState(1).into());

        assert!(dispatch.get() != old)
    }

    #[test]
    fn dispatch_reduce_callback_works() {
        let dispatch = Dispatch::<TestState>::new(&Context::new());
        let old = dispatch.get();

        let cb = dispatch.reduce_callback(|_| TestState(1).into());
        cb.emit(());

        assert!(dispatch.get() != old)
    }

    #[test]
    fn dispatch_reduce_mut_callback_works() {
        let dispatch = Dispatch::<TestState>::new(&Context::new());
        let old = dispatch.get();

        let cb = dispatch.reduce_mut_callback(|state| state.0 += 1);
        cb.emit(());

        assert!(dispatch.get() != old)
    }

    #[test]
    fn dispatch_reduce_callback_with_works() {
        let dispatch = Dispatch::<TestState>::new(&Context::new());
        let old = dispatch.get();

        let cb = dispatch.reduce_callback_with(|_, _| TestState(1).into());
        cb.emit(1);

        assert!(dispatch.get() != old)
    }

    #[test]
    fn dispatch_reduce_mut_callback_with_works() {
        let dispatch = Dispatch::<TestState>::new(&Context::new());
        let old = dispatch.get();

        let cb = dispatch.reduce_mut_callback_with(|state, val| state.0 += val);
        cb.emit(1);

        assert!(dispatch.get() != old)
    }

    #[test]
    fn dispatch_apply_works() {
        let dispatch = Dispatch::<TestState>::new(&Context::new());
        let old = dispatch.get();

        dispatch.apply(Msg);

        assert!(dispatch.get() != old)
    }

    #[test]
    fn dispatch_apply_callback_works() {
        let dispatch = Dispatch::<TestState>::new(&Context::new());
        let old = dispatch.get();

        let cb = dispatch.apply_callback(|_| Msg);
        cb.emit(());

        assert!(dispatch.get() != old)
    }

    #[test]
    fn subscriber_is_notified() {
        let cx = Context::new();
        let flag = Mrc::new(false);

        let _id = {
            let flag = flag.clone();
            Dispatch::<TestState>::new(&cx)
                .subscribe(move |_| flag.clone().with_mut(|flag| *flag = true))
        };

        *flag.borrow_mut() = false;

        Dispatch::<TestState>::new(&cx).reduce_mut(|state| state.0 += 1);

        assert!(*flag.borrow());
    }

    #[test]
    fn subscriber_is_not_notified_when_state_is_same() {
        let cx = Context::new();
        let flag = Mrc::new(false);
        let dispatch = Dispatch::<TestState>::new(&cx);

        // TestState(1)
        dispatch.reduce_mut(|_| {});

        let _id = {
            let flag = flag.clone();
            Dispatch::<TestState>::new(&cx)
                .subscribe(move |_| flag.clone().with_mut(|flag| *flag = true))
        };

        *flag.borrow_mut() = false;

        // TestState(1)
        dispatch.reduce_mut(|state| state.0 = 0);

        assert!(!*flag.borrow());
    }

    #[test]
    fn dispatch_unsubscribes_when_dropped() {
        let cx = Context::new();
        let entry = cx.get_or_init_default::<Mrc<Subscribers<TestState>>>();

        assert!(entry.store.borrow().borrow().0.is_empty());

        let dispatch = Dispatch::<TestState>::new(&cx).subscribe(|_| ());

        assert!(!entry.store.borrow().borrow().0.is_empty());

        drop(dispatch);

        assert!(entry.store.borrow().borrow().0.is_empty());
    }

    #[test]
    fn dispatch_clone_and_original_unsubscribe_when_both_dropped() {
        let cx = Context::new();
        let entry = cx.get_or_init_default::<Mrc<Subscribers<TestState>>>();

        assert!(entry.store.borrow().borrow().0.is_empty());

        let dispatch = Dispatch::<TestState>::new(&cx).subscribe(|_| ());
        let dispatch_clone = dispatch.clone();

        assert!(!entry.store.borrow().borrow().0.is_empty());

        drop(dispatch_clone);

        assert!(!entry.store.borrow().borrow().0.is_empty());

        drop(dispatch);

        assert!(entry.store.borrow().borrow().0.is_empty());
    }
}
