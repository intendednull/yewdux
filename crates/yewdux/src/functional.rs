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
pub fn use_store<S: Store>() -> (Rc<S>, Dispatch<S>) {
    let state = use_state(|| dispatch::get::<S>());

    let dispatch = {
        let state = state.clone();
        use_state(move || Dispatch::<S>::subscribe(move |val| state.set(val)))
    };

    (Rc::clone(&state), dispatch.deref().clone())
}

/// Simliar to ['use_store'], but only provides the state.
#[hook]
pub fn use_store_value<S: Store>() -> Rc<S> {
    let state = use_state(|| dispatch::get::<S>());

    let _dispatch = {
        let state = state.clone();
        use_state(move || Dispatch::<S>::subscribe(move |val| state.set(val)))
    };

    Rc::clone(&state)
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
pub fn use_selector_eq<S, F, R, E>(selector: F, eq: E) -> Rc<R>
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

    let _dispatch = {
        let selected = selected.clone();
        // Local var for tracking value (`selected` is not updated in the scope below).
        let mut current = Rc::clone(&selected);
        use_state(move || {
            Dispatch::subscribe(move |val: Rc<S>| {
                let value = selector(&val);

                if !eq(&current, &value) {
                    let value = Rc::new(value);
                    // Update value for user.
                    selected.set(Rc::clone(&value));
                    // Make sure to update our tracking value too.
                    current = Rc::clone(&value);
                }
            })
        })
    };

    Rc::clone(&selected)
}

/// This hook provides access to a portion of state. Similar to [`use_selector_eq`], with a default
/// equality function of `|a, b| a == b`.
#[hook]
pub fn use_selector<S, F, R>(selector: F) -> Rc<R>
where
    S: Store,
    R: PartialEq + 'static,
    F: Fn(&S) -> R + 'static,
{
    use_selector_eq(selector, |a, b| a == b)
}
