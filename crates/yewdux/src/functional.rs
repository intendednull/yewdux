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
        use_state(move || Dispatch::<S>::subscribe_silent(move |val| state.set(val)))
    };

    (Rc::clone(&state), dispatch.deref().clone())
}

/// Simliar to ['use_store'], but only provides the state.
#[hook]
pub fn use_store_value<S: Store>() -> Rc<S> {
    let (state, _dispatch) = use_store();

    state
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
    use_selector_eq_with_deps(move |state, _| selector(state), eq, ())
}

/// This hook provides access to a portion of state. Similar to [`use_selector_eq`], with a default
/// equality function of `|a, b| a == b`.
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
///     let count = use_selector(|state: &State| state.count);
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
pub fn use_selector<S, F, R>(selector: F) -> Rc<R>
where
    S: Store,
    R: PartialEq + 'static,
    F: Fn(&S) -> R + 'static,
{
    use_selector_eq(selector, |a, b| a == b)
}

/// Hook for selecting a value with dependencies.
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
///     let multiplier = use_state(|| 1);
///     let count = use_selector_with_deps(
///        |state: &State, multiplier| *multiplier * state.count,
///        *multiplier,
///     );
///     let incr = Dispatch::<State>::new().reduce_callback(|state| state.count += 1);
///     let incr_multiplier = {
///         let multiplier = multiplier.clone();
///         Callback::from(|_| multiplier.set(*multiplier + 1))
///     );
///
///     html! {
///         <>
///         <p>{ *count }</p>
///         <button onclick={incr}>{"+1"}</button>
///         <button onclick={incr_multiplier}>{"+1 mult"}</button>
///         </>
///     }
/// }
/// ```
#[hook]
pub fn use_selector_with_deps<S, F, R, D>(selector: F, deps: D) -> Rc<R>
where
    S: Store,
    R: PartialEq + 'static,
    D: Clone + PartialEq + 'static,
    F: Fn(&S, &D) -> R + 'static,
{
    use_selector_eq_with_deps(selector, |a, b| a == b, deps)
}

/// Hook for selecting a value with dependencies.
#[hook]
pub fn use_selector_eq_with_deps<S, F, R, D, E>(selector: F, eq: E, deps: D) -> Rc<R>
where
    S: Store,
    R: 'static,
    D: Clone + PartialEq + 'static,
    F: Fn(&S, &D) -> R + 'static,
    E: Fn(&R, &R) -> bool + 'static,
{
    // Given to user, this is what we update to force a re-render.
    let selected = {
        let state = dispatch::get::<S>();
        let value = selector(&state, &deps);

        use_state(|| Rc::new(value))
    };
    // Local tracking value, because `selected` isn't updated in our subscriber scope.
    let current = {
        let value = Rc::clone(&selected);
        use_mut_ref(|| value)
    };

    let _dispatch = {
        let selected = selected.clone();
        use_memo(
            move |deps| {
                let deps = deps.clone();
                Dispatch::subscribe(move |val: Rc<S>| {
                    let value = selector(&val, &deps);

                    if !eq(&current.borrow(), &value) {
                        let value = Rc::new(value);
                        // Update value for user.
                        selected.set(Rc::clone(&value));
                        // Make sure to update our tracking value too.
                        *current.borrow_mut() = Rc::clone(&value);
                    }
                })
            },
            deps,
        )
    };

    Rc::clone(&selected)
}
