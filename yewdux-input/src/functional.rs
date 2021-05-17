use std::{cell::RefCell, ops::Deref, rc::Rc};

use yew::prelude::*;
use yew_functional::*;
use yewdux::{dispatch::Dispatch, store::Store};

use crate::InputDispatch;

/// Reference to a store's state and dispatch.
pub struct InputRef<T: Store> {
    state: UseStateHandle<Option<Rc<T::Model>>>,
    dispatch: Rc<RefCell<InputDispatch<T>>>,
}

impl<T: Store> InputRef<T> {
    pub fn dispatch<'a>(&'a self) -> impl Deref<Target = InputDispatch<T>> + 'a {
        self.dispatch.borrow()
    }

    pub fn state(&self) -> Option<&T::Model> {
        self.state.as_ref().map(Rc::as_ref)
    }
}

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
/// #[derive(Default, Clone)]
/// struct State {
///     count: u32,
/// }
///
/// #[function_component(App)]
/// fn app() -> Html {
///     let store = use_input::<BasicStore<State>>();
///     let count = store.state().map(|s| s.count).unwrap_or(0);
///     let onclick = store.dispatch().reduce_callback(|s| s.count += 1);
///     html! {
///         <>
///         <p>{ count }</p>
///         <button onclick=onclick>{"+1"}</button>
///         </>
///     }
/// }
/// ```
pub fn use_input<T: Store>() -> InputRef<T> {
    let state = use_state(|| None);

    let dispatch = {
        let state = state.clone();
        // persist the Dispatch across renders
        use_ref(move || {
            let on_state = Callback::from(move |new_state| {
                state.set(Some(new_state));
            });

            InputDispatch(Dispatch::<T>::bridge_state(on_state))
        })
    };

    InputRef { state, dispatch }
}
