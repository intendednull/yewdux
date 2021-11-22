use std::{cell::RefCell, ops::Deref, rc::Rc};

use yew::{functional::*, prelude::*};
use yewdux::dispatch::Dispatch;
use yewdux::store::Store;

/// Reference to a store's state and dispatch.
pub struct StoreRef<T: Store> {
    state: UseStateHandle<Option<Rc<T::Model>>>,
    dispatch: Rc<RefCell<Dispatch<T>>>,
    #[allow(dead_code)]
    output: Option<Rc<RefCell<Dispatch<T>>>>,
}

impl<T: Store> StoreRef<T> {
    pub fn dispatch<'a>(&'a self) -> impl Deref<Target = Dispatch<T>> + 'a {
        self.dispatch.borrow()
    }

    pub fn state(&self) -> Option<&Rc<T::Model>> {
        self.state.as_ref()
    }

    pub fn on_output(mut self, on_output: impl Fn(T::Output) + 'static) -> Self {
        self.output = Some(use_mut_ref(move || {
            Dispatch::bridge(Default::default(), on_output.into())
        }));
        self
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
///     let store = use_store::<BasicStore<State>>();
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
pub fn use_store<T: Store>() -> StoreRef<T> {
    let state = use_state(|| None);

    let dispatch = {
        let state = state.clone();
        // persist the Dispatch across renders
        use_mut_ref(move || {
            let on_state = Callback::from(move |new_state| {
                state.set(Some(new_state));
            });

            Dispatch::<T>::bridge_state(on_state)
        })
    };

    StoreRef {
        state,
        dispatch,
        output: None,
    }
}

/// This hook allows getting a [`Dispatch`] to the store.
///
/// Do not use the `state` method on the [`Dispatch`]. The dispatch should only be used to create callbacks.
/// The proper way to access the state is via the [`use_store`] hook.
///
/// # Example
/// ```ignore
/// # use yew_functional::function_component;
/// # use yew::prelude::*;
/// use yewdux::use_dispatch;
///
/// #[function_component(UseDispatch)]
/// fn dispatch() -> Html {
///     let dispatch = use_dispatch::<CounterStore>();
///     
///     html! {
///         <button onclick=dispatch.callback(|_| Input::Increment)>{ "Increment" }</button>
///     }
/// }
/// ```
pub fn use_dispatch<T: Store>() -> impl Deref<Target = Dispatch<T>> {
    // persist the Dispatch across renders
    let dispatch = use_state(Dispatch::<T>::new);

    dispatch
}
