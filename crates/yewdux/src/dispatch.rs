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
    context,
    store::{Reducer, Store},
    subscriber::{subscribe, Callable, SubscriberId},
};

/// The primary interface to a [`Store`].
#[derive(Debug, Default)]
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

    /// Create a dispatch, and subscribe to state changes. Will automatically unsubscribe when this
    /// dispatch is dropped.
    pub fn subscribe<C: Callable<S>>(on_change: C) -> Self {
        let id = subscribe(on_change);

        Self {
            _subscriber_id: Some(Rc::new(id)),
        }
    }

    /// Get the current state.
    pub fn get(&self) -> Rc<S> {
        get::<S>()
    }

    /// Send a message to the store.
    pub fn apply<M: Reducer<S>>(&self, msg: M) {
        apply(msg);
    }

    /// Callback for sending a message to the store.
    ///
    /// ```ignore
    /// let onclick = dispatch.apply_callback(|_| StoreMsg::AddOne);
    /// ```
    pub fn apply_callback<E, M, F>(&self, f: F) -> Callback<E>
    where
        M: Reducer<S>,
        F: Fn(E) -> M + 'static,
    {
        Callback::from(move |e| {
            let msg = f(e);
            apply(msg);
        })
    }

    /// Set state to given value.
    pub fn set(&self, val: S) {
        set(val);
    }

    /// Set state using value from callback.
    pub fn set_callback<E, F>(&self, f: F) -> Callback<E>
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
        R: Into<Rc<S>>,
        F: FnOnce(Rc<S>) -> R + 'static,
    {
        reduce(f);
    }

    /// Like [reduce](Self::reduce) but from a callback.
    ///
    /// ```ignore
    /// let onclick = dispatch.reduce_callback(|s| s.count += 1);
    /// ```
    pub fn reduce_callback<F, R, E>(&self, f: F) -> Callback<E>
    where
        R: Into<Rc<S>>,
        F: Fn(Rc<S>) -> R + 'static,
        E: 'static,
    {
        Callback::from(move |_| {
            reduce(&f);
        })
    }

    /// Similar to [Self::reduce_callback] but also provides the fired event.
    ///
    /// ```ignore
    /// let oninput = dispatch.reduce_callback_with(|state, name: String| state.name = name);
    /// ```
    pub fn reduce_callback_with<F, R, E>(&self, f: F) -> Callback<E>
    where
        R: Into<Rc<S>>,
        F: Fn(Rc<S>, E) -> R + 'static,
        E: 'static,
    {
        Callback::from(move |e: E| {
            reduce(|x| f(x, e));
        })
    }

    /// Mutate state with given function.
    ///
    /// ```ignore
    /// let onclick = dispatch.reduce(|state| state.count += 1);
    /// ```
    pub fn reduce_mut<F, R>(&self, f: F)
    where
        S: Clone,
        F: FnOnce(&mut S) -> R + 'static,
    {
        reduce_mut(|x| {
            f(x);
        });
    }

    /// Like [reduce](Self::reduce) but from a callback.
    ///
    /// ```ignore
    /// let onclick = dispatch.reduce_callback(|s| s.count += 1);
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

    /// Similar to [Self::reduce_callback] but also provides the fired event.
    ///
    /// ```ignore
    /// let oninput = dispatch.reduce_callback_with(|state, name: String| state.name = name);
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
pub fn reduce<S: Store, R: Into<Rc<S>>, F: FnOnce(Rc<S>) -> R>(f: F) {
    let mut context = context::get_or_init::<S>();
    context.with_mut(|context| context.reduce(|s| f(s).into()));
}

/// Change state using a mutable reference from a function.
pub fn reduce_mut<S: Store + Clone, F: FnOnce(&mut S)>(f: F) {
    reduce(|mut state| {
        f(Rc::make_mut(&mut state));
        state
    });
}

/// Set state to given value.
pub fn set<S: Store>(value: S) {
    reduce(move |_| value);
}

/// Send a message to state.
pub fn apply<S: Store, M: Reducer<S>>(msg: M) {
    reduce(move |state| msg.apply(state));
}

/// Get current state.
pub fn get<S: Store>() -> Rc<S> {
    Rc::clone(&context::get_or_init::<S>().borrow().store)
}

#[cfg(test)]
mod tests {

    use crate::mrc::Mrc;

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
    impl Reducer<TestState> for Msg {
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
    fn apply_changes_value() {
        let old = get::<TestState>();

        apply::<TestState, Msg>(Msg);

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
        dispatch.set(TestState(1));

        assert!(dispatch.get().0 == 2)
    }

    #[test]
    fn dispatch_set_callback_works() {
        let dispatch = Dispatch::<TestState>::new();
        let cb = dispatch.set_callback(|_| TestState(1));
        cb.emit(());

        assert!(dispatch.get().0 == 2)
    }

    #[test]
    fn dispatch_reduce_works() {
        let dispatch = Dispatch::<TestState>::new();
        dispatch.reduce_mut(|state| state.0 += 1);

        assert!(dispatch.get().0 == 2)
    }

    #[test]
    fn dispatch_reduce_callback_works() {
        let dispatch = Dispatch::<TestState>::new();
        let cb = dispatch.reduce_mut_callback(|state| state.0 += 1);
        cb.emit(());

        assert!(dispatch.get().0 == 2)
    }

    #[test]
    fn dispatch_reduce_callback_with_works() {
        let dispatch = Dispatch::<TestState>::new();
        let cb = dispatch.reduce_callback_with(|state, val| state.0 += val);
        cb.emit(1);

        assert!(dispatch.get().0 == 2)
    }

    #[test]
    fn dispatch_apply_works() {
        let dispatch = Dispatch::<TestState>::new();
        dispatch.apply(Msg);

        assert!(dispatch.get().0 == 2)
    }

    #[test]
    fn dispatch_apply_callback_works() {
        let dispatch = Dispatch::<TestState>::new();
        let cb = dispatch.apply_callback(|_| Msg);
        cb.emit(());

        assert!(dispatch.get().0 == 2)
    }

    #[test]
    fn subscriber_is_notified() {
        let mut flag = Mrc::new(false);

        let _id = {
            let flag = flag.clone();
            subscribe::<TestState, _>(move |_| flag.clone().with_mut(|flag| *flag = true))
        };

        *flag.borrow_mut() = false;

        reduce::<TestState, _>(|state| state.0 += 1);

        assert!(*flag.borrow());
    }

    #[test]
    fn subscriber_is_not_notified_when_state_is_same() {
        let mut flag = Mrc::new(false);

        // TestState(1)
        reduce::<TestState, _>(|_| {});

        let _id = {
            let flag = flag.clone();
            subscribe::<TestState, _>(move |_| flag.clone().with_mut(|flag| *flag = true))
        };

        *flag.borrow_mut() = false;

        // TestState(1)
        reduce::<TestState, _>(|state| state.0 = 0);

        assert!(!*flag.borrow());
    }

    #[test]
    fn dispatch_unsubscribes_when_dropped() {
        let context = context::get_or_init::<TestState>();

        assert!(context.borrow().subscribers.is_empty());

        let dispatch = Dispatch::<TestState>::subscribe(|_| ());

        assert!(!context.borrow().subscribers.is_empty());

        drop(dispatch);

        assert!(context.borrow().subscribers.is_empty());
    }

    #[test]
    fn dispatch_clone_and_original_unsubscribe_when_both_dropped() {
        let context = context::get_or_init::<TestState>();

        assert!(context.borrow().subscribers.is_empty());

        let dispatch = Dispatch::<TestState>::subscribe(|_| ());
        let dispatch_clone = dispatch.clone();

        assert!(!context.borrow().subscribers.is_empty());

        drop(dispatch_clone);

        assert!(!context.borrow().subscribers.is_empty());

        drop(dispatch);

        assert!(context.borrow().subscribers.is_empty());
    }
}
