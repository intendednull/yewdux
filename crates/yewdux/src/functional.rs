//! The functional interface for Yewdux
use std::{ops::Deref, rc::Rc};

use yew::functional::*;

use crate::{
    dispatch::{self, Dispatch},
    store::Store,
};

/// This hook allows accessing the state of a store. When the store is modified, a re-render is
/// automatically triggered.
///
/// # Example
/// ```ignore
/// # use yew_functional::function_component;
/// # use yew::prelude::*;
/// use yewdux_functional::use_store;
///
/// #[derive(Default, Clone, Store)]
/// struct State {
///     count: u32,
/// }
///
/// #[function_component(App)]
/// fn app() -> Html {
///     let (state, dispatch) = use_store::<State>();
///     let onclick = dispatch.reduce_callback(|s| s.count += 1);
///     html! {
///         <>
///         <p>{ state.count }</p>
///         <button {onclick}>{"+1"}</button>
///         </>
///     }
/// }
/// ```
#[hook]
pub fn use_store<S: Store>() -> (RefHandle<Rc<S>>, RefHandle<Dispatch<S>>) {
    let state = use_state(|| dispatch::get::<S>());

    let dispatch = {
        let state = state.clone();
        use_state(move || Dispatch::<S>::subscribe(move |val| state.set(val)))
    };

    (RefHandle(state), RefHandle(dispatch))
}

/// This hook provides access to a portion of state. The equality function tests whether the next
/// selection is equal to previous, and re-renders when true.
///
/// # Example
/// ```ignore
/// #[derive(Default, Clone, PartialEq, Store)]
/// struct State {
///     count: u32,
/// }
///
/// #[function_component]
/// fn App() -> Html {
///     let count = use_selector_eq(|state: &State| state.count, |a, b| a == b);
///     let dispatch = Dispatch::<State>::new();
///     let onclick = dispatch.reduce_callback(|state| state.count += 1);
///
///     html! {
///         <>
///         <p>{ *count }</p>
///         <button {onclick}>{"+1"}</button>
///         </>
///     }
/// }
/// ```
#[hook]
pub fn use_selector_eq<S, F, R, E>(selector: F, eq: E) -> RcHandle<R>
where
    S: Store,
    R: 'static,
    F: Fn(&S) -> R + 'static,
    E: Fn(&R, &R) -> bool + 'static,
{
    // Given to user, this is what we update to force a re-render.
    let selected = {
        let state = dispatch::get::<S>();
        let value = selector(&state);

        use_state(|| Rc::new(value))
    };
    // This is required to track the current state. `selected` does not update in our subscription
    // function.
    let current = {
        let selected = Rc::clone(&selected);
        use_mut_ref(|| selected)
    };

    let _dispatch = {
        let selected = selected.clone();
        use_state(move || {
            Dispatch::subscribe(move |val: Rc<S>| {
                let value = selector(&val);

                if !eq(&current.borrow(), &value) {
                    let value = Rc::new(value);
                    // Update value for user.
                    selected.set(Rc::clone(&value));
                    // Make sure to update our tracking value too.
                    *current.borrow_mut() = Rc::clone(&value);
                }
            })
        })
    };

    RcHandle(selected)
}

/// This hook provides access to a portion of state. Similar to [`use_selector_eq`], with a default
/// equality function of `|a, b| a == b`.
#[hook]
pub fn use_selector<S, F, R>(selector: F) -> RcHandle<R>
where
    S: Store,
    R: PartialEq + 'static,
    F: Fn(&S) -> R + 'static,
{
    use_selector_eq(selector, |a, b| a == b)
}

/// Helper type for wrapping hook handles. Ensures the inner handle is only accessible to library
/// code.
#[derive(Debug, PartialEq, Clone)]
pub struct RefHandle<T>(UseStateHandle<T>);

impl<T: std::fmt::Display> std::fmt::Display for RefHandle<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl<T> AsRef<T> for RefHandle<T> {
    fn as_ref(&self) -> &T {
        self.0.deref()
    }
}

impl<T> Deref for RefHandle<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

/// Simlar to [`RefHandle`]. Used when the handle has in inner `Rc`, and we'd like to deref it
/// directly.
#[derive(Debug, PartialEq, Clone)]
pub struct RcHandle<T>(UseStateHandle<Rc<T>>);

impl<T: std::fmt::Display> std::fmt::Display for RcHandle<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl<T> AsRef<T> for RcHandle<T> {
    fn as_ref(&self) -> &T {
        self.deref()
    }
}

impl<T> Deref for RcHandle<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.0.deref().deref()
    }
}
