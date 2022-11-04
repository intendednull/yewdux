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
//!  # fn main() {
//!  let dispatch = Dispatch::<State>::new();
//!  dispatch.reduce_mut(|state| state.count = 1);
//!
//!  let state = dispatch.get();
//!
//!  assert!(state.count == 1);
//!  # }
//!  ```
//!

use std::rc::Rc;
#[cfg(feature = "future")]
use std::{future::Future, pin::Pin};

use yew::Callback;

use crate::{
    context,
    mrc::Mrc,
    store::{AsyncReducer, Reducer, Store},
    subscriber::{Callable, SubscriberId, Subscribers},
};

/// The primary interface to a [`Store`].
pub struct Dispatch<S: Store> {
    _subscriber_id: Option<Rc<SubscriberId<S>>>,
}

impl<S: Store> Dispatch<S> {
    /// Create a new dispatch.
    pub fn new() -> Self {
        Self {
            _subscriber_id: Default::default(),
        }
    }

    /// Create a dispatch that subscribes to changes in state. Latest state is sent immediately,
    /// and on every subsequent change. Automatically unsubscribes when this dispatch is dropped.
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
    /// struct App {
    ///     /// Our local version of state.
    ///     state: Rc<State>,
    ///     /// Our dispatch. Make sure to keep this, or the subscription will be dropped.
    ///     dispatch: Dispatch<State>,
    /// }
    ///
    /// enum Msg {
    ///     /// Message to receive new state.
    ///     State(Rc<State>),
    /// }
    ///
    /// impl Component for App {
    ///     type Message = Msg;
    /// #    type Properties = ();
    ///
    ///     fn create(ctx: &Context<Self>) -> Self {
    ///         let on_change = ctx.link().callback(Msg::State);
    ///         let dispatch = Dispatch::subscribe(on_change);
    ///         Self {
    ///             state: dispatch.get(),
    ///             dispatch,
    ///         }
    ///     }
    ///
    ///     fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
    ///         match msg {
    ///             Msg::State(state) => {
    ///                 self.state = state;
    ///                 true
    ///             }
    ///         }
    ///     }
    ///
    ///     /// ...
    /// #    fn view(&self, _ctx: &Context<Self>) -> Html {
    /// #        html! {}
    /// #    }
    /// }
    /// ```
    pub fn subscribe<C: Callable<S>>(on_change: C) -> Self {
        let id = subscribe(on_change);

        Self {
            _subscriber_id: Some(Rc::new(id)),
        }
    }

    /// Create a dispatch that subscribes to changes in state. Similar to [Self::subscribe],
    /// however state is **not** sent immediately. Automatically unsubscribes when this dispatch is
    /// dropped.
    pub fn subscribe_silent<C: Callable<S>>(on_change: C) -> Self {
        let id = subscribe_silent(on_change);

        Self {
            _subscriber_id: Some(Rc::new(id)),
        }
    }

    /// Get the current state.
    pub fn get(&self) -> Rc<S> {
        get::<S>()
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
    /// # let dispatch = Dispatch::<State>::new();
    /// dispatch.apply(AddOne);
    /// # ;
    /// # }
    /// ```
    pub fn apply<R: Reducer<S>>(&self, reducer: R) {
        reduce(reducer);
    }

    /// Apply an [`AsyncReducer`](crate::store::AsyncReducer) immediately.
    ///
    /// ```
    /// # use std::rc::Rc;
    /// # use yew::prelude::*;
    /// # use yewdux::prelude::*;
    /// # async fn get_incr() -> u32 {
    /// #     1
    /// # }
    /// #[derive(Default, Clone, PartialEq, Eq, Store)]
    /// struct State {
    ///     count: u32,
    /// }
    ///
    /// struct AddOne;
    /// #[async_reducer]
    /// impl AsyncReducer<State> for AddOne {
    ///     async fn apply(self, state: Rc<State>) -> Rc<State> {
    ///         // you can do async things here!
    ///         let incr = get_incr().await;
    ///         State {
    ///             count: state.count + incr,
    ///         }
    ///         .into()
    ///     }
    /// }
    ///
    /// # async fn do_thing() {
    /// # let dispatch = Dispatch::<State>::new();
    /// dispatch.apply_future(AddOne).await;
    /// # ;
    /// # }
    /// ```
    #[cfg(feature = "future")]
    pub async fn apply_future<R: AsyncReducer<S>>(&self, reducer: R) {
        reduce_future(reducer).await;
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
    /// # let dispatch = Dispatch::<State>::new();
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
        Callback::from(move |e| {
            let msg = f(e);
            reduce(msg);
        })
    }

    /// Create a callback for applying an [`AsyncReducer`](crate::store::AsyncReducer).
    ///
    /// ```
    /// # use std::rc::Rc;
    /// # use yew::prelude::*;
    /// # use yewdux::prelude::*;
    /// # async fn get_incr() -> u32 {
    /// #     1
    /// # }
    /// #[derive(Default, Clone, PartialEq, Eq, Store)]
    /// struct State {
    ///     count: u32,
    /// }
    ///
    /// struct AddOne;
    /// #[async_reducer]
    /// impl AsyncReducer<State> for AddOne {
    ///     async fn apply(self, state: Rc<State>) -> Rc<State> {
    ///         // you can do async things here!
    ///         let incr = get_incr().await;
    ///         State {
    ///             count: state.count + incr,
    ///         }
    ///         .into()
    ///     }
    /// }
    ///
    /// # fn main() {
    /// # let dispatch = Dispatch::<State>::new();
    /// let onclick = dispatch.apply_future_callback(|_| AddOne);
    /// html! {
    ///     <button {onclick}>{"+1"}</button>
    /// }
    /// # ;
    /// # }
    /// ```
    #[cfg(feature = "future")]
    pub fn apply_future_callback<E, M, F>(&self, f: F) -> Callback<E>
    where
        M: AsyncReducer<S> + 'static,
        F: Fn(E) -> M + 'static,
    {
        Callback::from(move |e| {
            let msg = f(e);
            wasm_bindgen_futures::spawn_local(async move {
                reduce_future(msg).await;
            })
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
    /// # let dispatch = Dispatch::<State>::new();
    /// dispatch.set(State { count: 0 });
    /// # }
    /// ```
    pub fn set(&self, val: S) {
        set(val);
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
    /// # fn main() {
    /// # let dispatch = Dispatch::<State>::new();
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
        Callback::from(move |e| {
            let val = f(e);
            set(val);
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
    /// # let dispatch = Dispatch::<State>::new();
    /// dispatch.reduce(|state| State { count: state.count + 1 }.into());
    /// # }
    /// ```
    pub fn reduce<F>(&self, f: F)
    where
        F: FnOnce(Rc<S>) -> Rc<S>,
    {
        reduce(f);
    }

    /// Change state immediately, in an async context.
    ///
    /// ```
    /// # use yew::prelude::*;
    /// # use yewdux::prelude::*;
    /// # #[derive(Default, Clone, PartialEq, Eq, Store)]
    /// # struct State {
    /// #     count: u32,
    /// # }
    /// # async fn get_incr() -> u32 {
    /// #   1
    /// # }
    /// # async fn do_thing() {
    /// # let dispatch = Dispatch::<State>::new();
    /// dispatch
    ///     .reduce_future(|state| async move {
    ///         let incr = get_incr().await;
    ///         State {
    ///             count: state.count + incr,
    ///         }
    ///         .into()
    ///     })
    ///     .await;
    ///
    /// # }
    /// ```
    #[cfg(feature = "future")]
    pub async fn reduce_future<FUT, FUN>(&self, f: FUN)
    where
        FUT: Future<Output = Rc<S>>,
        FUN: FnOnce(Rc<S>) -> FUT,
    {
        reduce_future(f).await;
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
    /// # let dispatch = Dispatch::<State>::new();
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
        Callback::from(move |_| {
            reduce(&f);
        })
    }

    /// Create a callback to reduce state asynchronously.
    ///
    /// ```
    /// # use yew::prelude::*;
    /// # use yewdux::prelude::*;
    /// # #[derive(Default, Clone, PartialEq, Eq, Store)]
    /// # struct State {
    /// #     count: u32,
    /// # }
    /// # async fn get_incr() -> u32 {
    /// #   1
    /// # }
    /// # fn main() {
    /// # let dispatch = Dispatch::<State>::new();
    /// let onclick = dispatch.reduce_future_callback(|state| async move {
    ///     let incr = get_incr().await;
    ///     State {
    ///         count: state.count + incr,
    ///     }
    ///     .into()
    /// });
    /// html! {
    ///     <button {onclick}>{"+1"}</button>
    /// }
    /// # ;
    /// # }
    /// ```
    #[cfg(feature = "future")]
    pub fn reduce_future_callback<FUT, FUN, E>(&self, f: FUN) -> Callback<E>
    where
        FUT: Future<Output = Rc<S>>,
        FUN: Fn(Rc<S>) -> FUT + 'static,
        E: 'static,
    {
        let f = Rc::new(f);
        Callback::from(move |_| {
            let f = f.clone();
            wasm_bindgen_futures::spawn_local(async move {
                reduce_future(f.as_ref()).await;
            })
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
    /// # let dispatch = Dispatch::<State>::new();
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
        Callback::from(move |e: E| {
            reduce(|x| f(x, e));
        })
    }

    /// Create a callback to reduce state asynchronously, with the fired event.
    ///
    /// ```
    /// # use yew::prelude::*;
    /// # use yewdux::prelude::*;
    /// # #[derive(Default, Clone, PartialEq, Eq, Store)]
    /// # struct State {
    /// #     count: u32,
    /// # }
    /// # async fn get_incr() -> u32 {
    /// #     1
    /// # }
    /// # fn main() {
    /// # let dispatch = Dispatch::<State>::new();
    /// let onchange = dispatch.reduce_future_callback_with(|state, event: Event| async move {
    ///     let value = event.target_unchecked_into::<web_sys::HtmlInputElement>().value();
    ///     let incr = get_incr().await;
    ///     let count = value.parse::<u32>().unwrap() * incr;
    ///     State {
    ///         count: state.count + count,
    ///     }
    ///     .into()
    /// });
    /// html! {
    ///     <input type="number" placeholder="Enter a number" {onchange}  />
    /// }
    /// # ;
    /// # }
    /// ```
    #[cfg(feature = "future")]
    pub fn reduce_future_callback_with<FUT, FUN, E>(&self, f: FUN) -> Callback<E>
    where
        FUT: Future<Output = Rc<S>>,
        FUN: Fn(Rc<S>, E) -> FUT + 'static,
        E: 'static,
    {
        let f = Rc::new(f);
        Callback::from(move |e: E| {
            let f = f.clone();
            wasm_bindgen_futures::spawn_local(async move {
                reduce_future(move |s| f(s, e)).await;
            })
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
    /// # let dispatch = Dispatch::<State>::new();
    /// dispatch.reduce_mut(|state| state.count += 1);
    /// # }
    /// ```
    pub fn reduce_mut<F, R>(&self, f: F)
    where
        S: Clone,
        F: FnOnce(&mut S) -> R,
    {
        reduce_mut(|x| {
            f(x);
        });
    }

    /// Mutate state with given function, in an async context.
    ///
    /// ```
    /// # use yew::prelude::*;
    /// # use yewdux::prelude::*;
    /// # #[derive(Default, Clone, PartialEq, Eq, Store)]
    /// # struct State {
    /// #     count: u32,
    /// # }
    /// # async fn get_incr() -> u32 {
    /// #   1
    /// # }
    /// # async fn do_thing() {
    /// # let dispatch = Dispatch::<State>::new();
    /// dispatch
    ///     .reduce_mut_future(|state| {
    ///         Box::pin(async move {
    ///             let incr = get_incr().await;
    ///             state.count += incr;
    ///         })
    ///     })
    ///     .await;
    /// # }
    /// ```
    #[cfg(feature = "future")]
    pub async fn reduce_mut_future<R, F>(&self, f: F)
    where
        S: Clone,
        F: FnOnce(&mut S) -> Pin<Box<dyn Future<Output = R> + '_>>,
    {
        reduce_mut_future(f).await;
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
    /// # let dispatch = Dispatch::<State>::new();
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
        Callback::from(move |_| {
            reduce_mut(|x| {
                f(x);
            });
        })
    }

    /// Create a callback to asynchronously mutate state.
    ///
    /// ```
    /// # use yew::prelude::*;
    /// # use yewdux::prelude::*;
    /// # #[derive(Default, Clone, PartialEq, Eq, Store)]
    /// # struct State {
    /// #     count: u32,
    /// # }
    /// # async fn get_incr() -> u32 {
    /// #   1
    /// # }
    /// # fn main() {
    /// # let dispatch = Dispatch::<State>::new();
    /// let onclick = dispatch.reduce_mut_future_callback(|state| Box::pin(async move {
    ///     let incr = get_incr().await;
    ///     state.count += incr;
    /// }));
    /// html! {
    ///     <button {onclick}>{"+1"}</button>
    /// }
    /// # ;
    /// # }
    /// ```
    ///
    #[cfg(feature = "future")]
    pub fn reduce_mut_future_callback<R, F, E>(&self, f: F) -> Callback<E>
    where
        S: Clone,
        F: Fn(&mut S) -> Pin<Box<dyn Future<Output = R> + '_>> + 'static,
        E: 'static,
    {
        let f = Rc::new(f);
        Callback::from(move |_| {
            let f = f.clone();
            wasm_bindgen_futures::spawn_local(async move {
                reduce_mut_future(f.as_ref()).await;
            })
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
    /// # let dispatch = Dispatch::<State>::new();
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
        Callback::from(move |e: E| {
            reduce_mut(|x| {
                f(x, e);
            });
        })
    }

    /// Create a callback to asynchronously mutate state with given function, provided the fired
    /// event.
    ///
    /// ```
    /// # use yew::prelude::*;
    /// # use yewdux::prelude::*;
    /// # #[derive(Default, Clone, PartialEq, Eq, Store)]
    /// # struct State {
    /// #     count: u32,
    /// # }
    /// # async fn get_incr() -> u32 {
    /// #     1
    /// # }
    /// # fn main() {
    /// # let dispatch = Dispatch::<State>::new();
    /// let onchange = dispatch.reduce_mut_future_callback_with(|state, event: Event| {
    ///     Box::pin(async move {
    ///         let value = event
    ///             .target_unchecked_into::<web_sys::HtmlInputElement>()
    ///             .value();
    ///         let incr = get_incr().await;
    ///         state.count = value.parse::<u32>().unwrap() * incr;
    ///     })
    /// });
    /// html! {
    ///     <input type="number" placeholder="Enter a number" {onchange}  />
    /// }
    /// # ;
    /// # }
    /// ```
    #[cfg(feature = "future")]
    pub fn reduce_mut_future_callback_with<R, F, E>(&self, f: F) -> Callback<E>
    where
        S: Clone,
        F: Fn(&mut S, E) -> Pin<Box<dyn Future<Output = R> + '_>> + 'static,
        E: 'static,
    {
        let f = Rc::new(f);
        Callback::from(move |e: E| {
            let f = f.clone();
            wasm_bindgen_futures::spawn_local(async move {
                reduce_mut_future(move |s| f(s, e)).await;
            })
        })
    }
}

impl<S: Store> Default for Dispatch<S> {
    fn default() -> Self {
        Self::new()
    }
}

impl<S: Store> Clone for Dispatch<S> {
    fn clone(&self) -> Self {
        Self {
            _subscriber_id: self._subscriber_id.clone(),
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

/// Change state from a function.
pub fn reduce<S: Store, R: Reducer<S>>(r: R) {
    let context = context::get_or_init::<S>();
    let should_notify = context.reduce(r);

    if should_notify {
        let state = Rc::clone(&context.store.borrow());
        notify_subscribers(state)
    }
}

#[cfg(feature = "future")]
pub async fn reduce_future<S, R>(r: R)
where
    S: Store,
    R: AsyncReducer<S>,
{
    let context = context::get_or_init::<S>();
    let should_notify = context.reduce_future(r).await;

    if should_notify {
        let state = Rc::clone(&context.store.borrow());
        notify_subscribers(state)
    }
}

/// Change state using a mutable reference from a function.
pub fn reduce_mut<S: Store + Clone, F: FnOnce(&mut S)>(f: F) {
    reduce(|mut state| {
        f(Rc::make_mut(&mut state));
        state
    });
}

#[cfg(feature = "future")]
pub async fn reduce_mut_future<S, R, F>(f: F)
where
    S: Store + Clone,
    F: FnOnce(&mut S) -> Pin<Box<dyn Future<Output = R> + '_>>,
{
    reduce_future(|mut state| async move {
        f(Rc::make_mut(&mut state)).await;
        state
    })
    .await;
}

/// Set state to given value.
pub fn set<S: Store>(value: S) {
    reduce(move |_| value.into());
}

/// Get current state.
pub fn get<S: Store>() -> Rc<S> {
    Rc::clone(&context::get_or_init::<S>().store.borrow())
}

/// Send state to all subscribers.
pub fn notify_subscribers<S: Store>(state: Rc<S>) {
    let context = context::get_or_init::<Mrc<Subscribers<S>>>();
    context.store.borrow().notify(state);
}

/// Subscribe to a store. `on_change` is called immediately, then every  time state changes.
pub fn subscribe<S: Store, N: Callable<S>>(on_change: N) -> SubscriberId<S> {
    // Notify subscriber with inital state.
    on_change.call(get::<S>());

    context::get_or_init::<Mrc<Subscribers<S>>>()
        .store
        .borrow()
        .subscribe(on_change)
}

/// Similar to [subscribe], however state is not called immediately.
pub fn subscribe_silent<S: Store, N: Callable<S>>(on_change: N) -> SubscriberId<S> {
    context::get_or_init::<Mrc<Subscribers<S>>>()
        .store
        .borrow()
        .subscribe(on_change)
}

#[cfg(test)]
mod tests {

    use crate::mrc::Mrc;

    use super::*;

    #[derive(Clone, PartialEq, Eq)]
    struct TestState(u32);
    impl Store for TestState {
        fn new() -> Self {
            Self(0)
        }

        fn should_notify(&self, other: &Self) -> bool {
            self != other
        }
    }
    #[derive(PartialEq, Eq)]
    struct TestStateNoClone(u32);
    impl Store for TestStateNoClone {
        fn new() -> Self {
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

    #[async_trait::async_trait(?Send)]
    impl AsyncReducer<TestState> for Msg {
        async fn apply(self, state: Rc<TestState>) -> Rc<TestState> {
            TestState(state.0 + 1).into()
        }
    }

    #[test]
    fn apply_no_clone() {
        reduce(|_| TestStateNoClone(1).into());
    }

    #[test]
    fn reduce_changes_value() {
        let old = get::<TestState>();

        reduce(|_| TestState(1).into());

        let new = get::<TestState>();

        assert!(old != new);
    }

    #[cfg(feature = "future")]
    #[async_std::test]
    async fn reduce_future_changes_value() {
        let old = get::<TestState>();

        reduce_future(|state: Rc<TestState>| async move { TestState(state.0 + 1).into() }).await;

        let new = get::<TestState>();

        assert!(old != new);
    }

    #[cfg(feature = "future")]
    #[async_std::test]
    async fn reduce_future_does_not_clash() {
        use std::time::Duration;

        let dispatch = Dispatch::<TestState>::new();

        dispatch
            .reduce_future(|state| async move {
                async_std::task::sleep(Duration::from_millis(100)).await;
                state
            })
            .await;

        dispatch.reduce(|s| s);
    }

    #[test]
    fn reduce_mut_changes_value() {
        let old = get::<TestState>();

        reduce_mut(|state| *state = TestState(1));

        let new = get::<TestState>();

        assert!(old != new);
    }

    #[cfg(feature = "future")]
    #[async_std::test]
    async fn reduce_mut_future_changes_value() {
        let old = get::<TestState>();

        reduce_mut_future(|state| Box::pin(async move { *state = TestState(1) })).await;

        let new = get::<TestState>();

        assert!(old != new);
    }

    #[test]
    fn reduce_does_not_require_static() {
        let val = "1".to_string();
        reduce(|_| TestState(val.parse().unwrap()).into());
    }

    #[test]
    fn reduce_mut_does_not_require_static() {
        let val = "1".to_string();
        reduce_mut(|state: &mut TestState| state.0 = val.parse().unwrap());
    }

    #[test]
    fn set_changes_value() {
        let old = get::<TestState>();

        set(TestState(1));

        let new = get::<TestState>();

        assert!(old != new);
    }

    #[test]
    fn apply_changes_value() {
        let old = get::<TestState>();

        reduce::<TestState, Msg>(Msg);

        let new = get::<TestState>();

        assert!(old != new);
    }

    #[test]
    fn dispatch_new_works() {
        let _dispatch = Dispatch::<TestState>::new();
    }

    #[test]
    fn dispatch_set_works() {
        let dispatch = Dispatch::<TestState>::new();
        let old = dispatch.get();

        dispatch.set(TestState(1));

        assert!(dispatch.get() != old)
    }

    #[test]
    fn dispatch_set_callback_works() {
        let dispatch = Dispatch::<TestState>::new();
        let old = dispatch.get();

        let cb = dispatch.set_callback(|_| TestState(1));
        cb.emit(());

        assert!(dispatch.get() != old)
    }

    #[test]
    fn dispatch_reduce_mut_works() {
        let dispatch = Dispatch::<TestState>::new();
        let old = dispatch.get();

        dispatch.reduce_mut(|state| state.0 += 1);

        assert!(dispatch.get() != old)
    }

    #[cfg(feature = "future")]
    #[async_std::test]
    async fn dispatch_reduce_mut_future_works() {
        let dispatch = Dispatch::<TestState>::new();
        let old = dispatch.get();

        dispatch
            .reduce_mut_future(|state| Box::pin(async move { state.0 += 1 }))
            .await;

        assert!(dispatch.get() != old)
    }

    #[test]
    fn dispatch_reduce_works() {
        let dispatch = Dispatch::<TestState>::new();
        let old = dispatch.get();

        dispatch.reduce(|_| TestState(1).into());

        assert!(dispatch.get() != old)
    }

    #[cfg(feature = "future")]
    #[async_std::test]
    async fn dispatch_reduce_future_works() {
        let dispatch = Dispatch::<TestState>::new();
        let old = dispatch.get();

        dispatch
            .reduce_future(|state| async move { TestState(state.0 + 1).into() })
            .await;

        assert!(dispatch.get() != old)
    }

    #[test]
    fn dispatch_reduce_callback_works() {
        let dispatch = Dispatch::<TestState>::new();
        let old = dispatch.get();

        let cb = dispatch.reduce_callback(|_| TestState(1).into());
        cb.emit(());

        assert!(dispatch.get() != old)
    }

    #[cfg(feature = "future")]
    #[async_std::test]
    async fn dispatch_reduce_future_callback_compiles() {
        let dispatch = Dispatch::<TestState>::new();

        let _ = dispatch.reduce_future_callback::<_, _, ()>(|state| async move {
            TestState(state.0 + 1).into()
        });
    }

    #[test]
    fn dispatch_reduce_mut_callback_works() {
        let dispatch = Dispatch::<TestState>::new();
        let old = dispatch.get();

        let cb = dispatch.reduce_mut_callback(|state| state.0 += 1);
        cb.emit(());

        assert!(dispatch.get() != old)
    }

    #[cfg(feature = "future")]
    #[async_std::test]
    async fn dispatch_reduce_mut_future_callback_compiles() {
        let dispatch = Dispatch::<TestState>::new();

        let _ = dispatch.reduce_mut_future_callback::<_, _, ()>(|state| {
            Box::pin(async move {
                state.0 += 1;
            })
        });
    }

    #[cfg(feature = "future")]
    #[async_std::test]
    async fn dispatch_reduce_future_callback_with_compiles() {
        let dispatch = Dispatch::<TestState>::new();

        let _ = dispatch.reduce_future_callback_with(|state, e: u32| async move {
            TestState(state.0 + e).into()
        });
    }

    #[test]
    fn dispatch_reduce_callback_with_works() {
        let dispatch = Dispatch::<TestState>::new();
        let old = dispatch.get();

        let cb = dispatch.reduce_callback_with(|_, _| TestState(1).into());
        cb.emit(1);

        assert!(dispatch.get() != old)
    }

    #[test]
    fn dispatch_reduce_mut_callback_with_works() {
        let dispatch = Dispatch::<TestState>::new();
        let old = dispatch.get();

        let cb = dispatch.reduce_mut_callback_with(|state, val| state.0 += val);
        cb.emit(1);

        assert!(dispatch.get() != old)
    }

    #[cfg(feature = "future")]
    #[async_std::test]
    async fn dispatch_reduce_mut_future_callback_with_compiles() {
        let dispatch = Dispatch::<TestState>::new();

        let _ = dispatch.reduce_mut_future_callback_with::<_, _, u32>(|state, e| {
            Box::pin(async move {
                state.0 += e;
            })
        });
    }

    #[test]
    fn dispatch_apply_works() {
        let dispatch = Dispatch::<TestState>::new();
        let old = dispatch.get();

        dispatch.apply(Msg);

        assert!(dispatch.get() != old)
    }

    #[cfg(feature = "future")]
    #[async_std::test]
    async fn apply_future_changes_value() {
        let old = get::<TestState>();
        let dispatch = Dispatch::<TestState>::new();

        dispatch.apply_future(Msg).await;

        let new = get::<TestState>();

        assert!(old != new);
    }

    #[test]
    fn dispatch_apply_callback_works() {
        let dispatch = Dispatch::<TestState>::new();
        let old = dispatch.get();

        let cb = dispatch.apply_callback(|_| Msg);
        cb.emit(());

        assert!(dispatch.get() != old)
    }

    #[cfg(feature = "future")]
    #[async_std::test]
    async fn apply_future_callback_compiles() {
        let dispatch = Dispatch::<TestState>::new();

        dispatch.apply_future_callback(|_: ()| Msg);
    }

    #[test]
    fn subscriber_is_notified() {
        let flag = Mrc::new(false);

        let _id = {
            let flag = flag.clone();
            subscribe::<TestState, _>(move |_| flag.clone().with_mut(|flag| *flag = true))
        };

        *flag.borrow_mut() = false;

        reduce_mut::<TestState, _>(|state| state.0 += 1);

        assert!(*flag.borrow());
    }

    #[test]
    fn subscriber_is_not_notified_when_state_is_same() {
        let flag = Mrc::new(false);

        // TestState(1)
        reduce_mut::<TestState, _>(|_| {});

        let _id = {
            let flag = flag.clone();
            subscribe::<TestState, _>(move |_| flag.clone().with_mut(|flag| *flag = true))
        };

        *flag.borrow_mut() = false;

        // TestState(1)
        reduce_mut::<TestState, _>(|state| state.0 = 0);

        assert!(!*flag.borrow());
    }

    #[test]
    fn dispatch_unsubscribes_when_dropped() {
        let context = context::get_or_init::<Mrc<Subscribers<TestState>>>();

        assert!(context.store.borrow().borrow().0.is_empty());

        let dispatch = Dispatch::<TestState>::subscribe(|_| ());

        assert!(!context.store.borrow().borrow().0.is_empty());

        drop(dispatch);

        assert!(context.store.borrow().borrow().0.is_empty());
    }

    #[test]
    fn dispatch_clone_and_original_unsubscribe_when_both_dropped() {
        let context = context::get_or_init::<Mrc<Subscribers<TestState>>>();

        assert!(context.store.borrow().borrow().0.is_empty());

        let dispatch = Dispatch::<TestState>::subscribe(|_| ());
        let dispatch_clone = dispatch.clone();

        assert!(!context.store.borrow().borrow().0.is_empty());

        drop(dispatch_clone);

        assert!(!context.store.borrow().borrow().0.is_empty());

        drop(dispatch);

        assert!(context.store.borrow().borrow().0.is_empty());
    }
}
