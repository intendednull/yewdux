//!  This module defines how you can interact with your [`Store`].
//!
//!  ```ignore
//!  use yewdux::prelude::*;
//!
//!  #[derive(Default, Clone, Store)]
//!  struct MyState {
//!     count: usize,
//!  }
//!
//!  # fn main() {
//!  let dispatch = Dispatch::<MyState>::new();
//!  dispatch.reduce(|state| state.count = 1);
//!
//!  let state = dispatch.get();
//!
//!  assert!(state.count == 1);
//!  # }
//!  ```
//!
use std::rc::Rc;

use yew::Callback;

use crate::{
    context::{self, SubscriberId},
    store::{Message, Store},
    util::Callable,
};

/// The primary interface to a [`Store`].
#[derive(Debug, Default)]
pub struct Dispatch<S: Store> {
    _subscriber_id: Option<SubscriberId<S>>,
}

impl<S: Store> Dispatch<S> {
    /// Create a new dispatch.
    pub fn new() -> Self {
        Self {
            _subscriber_id: Default::default(),
        }
    }

    /// Create a dispatch, and subscribe to state changes.
    pub fn subscribe<C: Callable<S>>(on_change: C) -> Self {
        let id = context::subscribe(on_change);

        Self {
            _subscriber_id: Some(id),
        }
    }

    /// Get the current state.
    pub fn get() -> Rc<S> {
        get::<S>()
    }

    /// Send a message to the store.
    pub fn apply<M: Message<S>>(&self, msg: M) {
        apply(msg);
    }

    /// Callback for sending a message to the store.
    ///
    /// ```ignore
    /// let onclick = dispatch.apply_callback(|_| StoreMsg::AddOne);
    /// ```
    pub fn apply_callback<E, M, F>(&self, f: F) -> Callback<E>
    where
        M: Message<S>,
        F: Fn(E) -> M + 'static,
    {
        Callback::from(move |e| {
            let msg = f(e);
            apply(msg);
        })
    }

    /// Set state to given value.
    pub fn set(val: S) {
        set(val);
    }

    /// Set state using value from callback.
    pub fn set_callback<E, F>(f: F) -> Callback<E>
    where
        F: Fn(E) -> S + 'static,
    {
        Callback::from(move |e| {
            let val = f(e);
            set(val);
        })
    }

    /// Mutate state with given function.
    ///
    /// ```ignore
    /// let onclick = dispatch.reduce(|state| state.count += 1);
    /// ```
    pub fn reduce<F, R>(&self, f: F)
    where
        F: FnOnce(&mut S) -> R + 'static,
    {
        reduce(|x| {
            f(x);
        });
    }

    /// Like [reduce](Self::reduce) but from a callback.
    ///
    /// ```ignore
    /// let onclick = dispatch.reduce_callback(|s| s.count += 1);
    /// ```
    pub fn reduce_callback<F, R, E>(&self, f: F) -> Callback<E>
    where
        F: Fn(&mut S) -> R + 'static,
        E: 'static,
    {
        Callback::from(move |_| {
            reduce(|x| {
                f(x);
            });
        })
    }

    /// Similar to [Self::reduce_callback] but also provides the fired event.
    ///
    /// ```ignore
    /// let oninput = dispatch.reduce_callback_with(|state, name: String| state.name = name);
    /// ```
    pub fn reduce_callback_with<F, R, E>(&self, f: F) -> Callback<E>
    where
        F: Fn(&mut S, E) -> R + 'static,
        E: 'static,
    {
        Callback::from(move |e: E| {
            reduce(|x| {
                f(x, e);
            });
        })
    }
}

/// Change state using given function.
pub fn reduce<S: Store, F: FnOnce(&mut S)>(f: F) {
    let mut context = context::get_or_init::<S>();
    let previous_store = Rc::clone(&context.borrow().store);

    context.with_mut(|context| {
        context.reduce(f);
    });

    // Only notify subscribers if state has changed.
    if previous_store.as_ref() != context.borrow().store.as_ref() {
        context.borrow().notify_subscribers();
    }
}

/// Set state to given value.
pub fn set<S: Store>(value: S) {
    reduce(move |store| *store = value);
}

/// Send a message to state.
pub fn apply<S: Store, M: Message<S>>(msg: M) {
    reduce(move |state: &mut S| msg.apply(state));
}

/// Get current state.
pub fn get<S: Store>() -> Rc<S> {
    Rc::clone(&context::get_or_init::<S>().borrow().store)
}

#[cfg(test)]
mod tests {

    use crate::util::Shared;

    use super::*;

    #[derive(Clone, PartialEq)]
    struct TestState(u32);
    impl Store for TestState {
        fn new() -> Self {
            Self(0)
        }

        fn changed(&mut self) {
            self.0 += 1;
        }
    }

    struct Msg;
    impl Message<TestState> for Msg {
        fn apply(&self, state: &mut TestState) {
            state.0 += 1;
        }
    }

    #[test]
    fn reduce_changes_value() {
        let old = get::<TestState>();

        reduce(|state| *state = TestState(1));

        let new = get::<TestState>();

        assert!(old != new);
    }

    #[test]
    fn set_changes_value() {
        let old = get::<TestState>();

        set(TestState(1));

        let new = get::<TestState>();

        assert!(old != new);
    }

    #[test]
    fn store_update_is_called_on_apply() {
        apply::<TestState, Msg>(Msg);

        assert!(get::<TestState>().0 == 2);
    }

    #[test]
    fn subscriber_is_not_notified_when_state_is_same() {
        let flag = Shared::new(false);

        // TestState(1)
        reduce::<TestState, _>(|_| {});

        let _id = {
            let flag = flag.clone();
            context::subscribe::<TestState, _>(move |_| flag.clone().with_mut(|flag| *flag = true))
        };

        // TestState(1)
        reduce::<TestState, _>(|state| state.0 = 0);

        assert!(!*flag.borrow());
    }
}
