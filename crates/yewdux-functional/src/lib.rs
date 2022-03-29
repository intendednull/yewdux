use std::{ops::Deref, rc::Rc};

use yew::functional::*;
use yewdux::dispatch::{self, Dispatch};
use yewdux::store::Store;

/// This hook allows accessing the state of a store. When the store is modified, a re-render is automatically triggered.
/// This hook also accepts a callback that is triggered for state output. To only receive state, use [`use_store_state`] instead.
///
/// This function returns the state of the store.
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

pub struct RefHandle<T>(UseStateHandle<T>);

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
