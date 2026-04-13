//! The functional interface for Yewdux
use std::{ops::Deref, rc::Rc};

use yew::functional::*;

use crate::{dispatch::Dispatch, store::Store, Context};

#[hook]
fn use_cx() -> Context {
    #[cfg(target_arch = "wasm32")]
    {
        use_context::<crate::context::Context>().unwrap_or_else(crate::context::Context::global)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        use_context::<crate::context::Context>().expect("YewduxRoot not found")
    }
}

#[hook]
pub fn use_dispatch<S>() -> Dispatch<S>
where
    S: Store,
{
    Dispatch::new(&use_cx())
}

/// This hook allows accessing the state of a store. When the store is modified, a re-render is
/// automatically triggered.
///
/// # Example
/// ```
/// use yew::prelude::*;
/// use yewdux::prelude::*;
///
/// #[derive(Default, Clone, PartialEq, Store)]
/// struct State {
///     count: u32,
/// }
///
/// #[function_component]
/// fn App() -> Html {
///     let (state, dispatch) = use_store::<State>();
///     let onclick = dispatch.reduce_mut_callback(|s| s.count += 1);
///     html! {
///         <>
///         <p>{ state.count }</p>
///         <button {onclick}>{"+1"}</button>
///         </>
///     }
/// }
/// ```
#[hook]
pub fn use_store<S>() -> (Rc<S>, Dispatch<S>)
where
    S: Store,
{
    let dispatch = use_dispatch::<S>();
    let state: UseStateHandle<Rc<S>> = use_state(|| dispatch.get());
    let dispatch = {
        let state = state.clone();
        use_state(move || dispatch.subscribe_silent(move |val| state.set(val)))
    };

    (Rc::clone(&state), dispatch.deref().clone())
}

/// Simliar to ['use_store'], but only provides the state.
#[hook]
pub fn use_store_value<S>() -> Rc<S>
where
    S: Store,
{
    let (state, _dispatch) = use_store();

    state
}

/// Provides access to some derived portion of state. Useful when you only want to rerender
/// when that portion has changed.
///
/// # Example
/// ```
/// use yew::prelude::*;
/// use yewdux::prelude::*;
///
/// #[derive(Default, Clone, PartialEq, Store)]
/// struct State {
///     count: u32,
/// }
///
/// #[function_component]
/// fn App() -> Html {
///     let dispatch = use_dispatch::<State>();
///     let count = use_selector(|state: &State| state.count);
///     let onclick = dispatch.reduce_mut_callback(|state| state.count += 1);
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

/// Similar to [`use_selector`], with the additional flexibility of a custom equality check for
/// selected value.
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

/// Similar to [`use_selector`], but also allows for dependencies from environment. This is
/// necessary when the derived value uses some captured value.
///
/// # Example
/// ```
/// use std::collections::HashMap;
///
/// use yew::prelude::*;
/// use yewdux::prelude::*;
///
/// #[derive(Default, Clone, PartialEq, Store)]
/// struct State {
///     user_names: HashMap<u32, String>,
/// }
///
/// #[derive(Properties, PartialEq, Clone)]
/// struct AppProps {
///     user_id: u32,
/// }
///
/// #[function_component]
/// fn ViewName(&AppProps { user_id }: &AppProps) -> Html {
///     let user_name = use_selector_with_deps(
///        |state: &State, id| state.user_names.get(id).cloned().unwrap_or_default(),
///        user_id,
///     );
///
///     html! {
///         <p>
///             { user_name }
///         </p>
///     }
/// }
/// ```
#[hook]
pub fn use_selector_with_deps<S, F, R, D>(selector: F, deps: D) -> Rc<R>
where
    S: Store,
    R: PartialEq + 'static,
    D: PartialEq + 'static,
    F: Fn(&S, &D) -> R + 'static,
{
    use_selector_eq_with_deps(selector, |a, b| a == b, deps)
}

/// Similar to [`use_selector_with_deps`], but also allows an equality function, similar to
/// [`use_selector_eq`]
#[hook]
pub fn use_selector_eq_with_deps<S, F, R, D, E>(selector: F, is_eq: E, deps: D) -> Rc<R>
where
    S: Store,
    R: 'static,
    D: PartialEq + 'static,
    F: Fn(&S, &D) -> R + 'static,
    E: Fn(&R, &R) -> bool + 'static,
{
    let dispatch = use_dispatch::<S>();
    let store = dispatch.get();

    // Re-run this hook when the store changes.
    let _trigger = use_state(|| ());
    let _sub = use_memo((), move |_| {
        dispatch.subscribe_silent(move |_| {
            // Trigger a re-render when the store changes.
            _trigger.set(());
        })
    });

    // Track if the store has changed.
    let store_version = use_state(|| 0u32);
    let last_store = use_state(|| Rc::clone(&store));
    if store.should_notify(&last_store) {
        last_store.set(Rc::clone(&store));
        store_version.set(1u32.wrapping_add(*store_version));
    }

    // Track if the selected value has changed
    let last_selected = use_state(|| None::<Rc<R>>);
    let selected = use_memo((store_version, deps), move |(_version, deps)| {
        let value = Rc::new(selector(&store, &deps));
        match &*last_selected {
            Some(last) if is_eq(&value, last) => Rc::clone(last),
            _ => {
                last_selected.set(Some(Rc::clone(&value)));
                value
            }
        }
    });

    Rc::clone(&selected)
}
