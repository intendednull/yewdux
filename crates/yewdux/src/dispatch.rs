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
#[derive(Debug, Default, Clone)]
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
        let id = context::subscribe(on_change);

        Self {
            _subscriber_id: Some(Rc::new(id)),
        }
    }

    /// Get the current state.
    pub fn get(&self) -> Rc<S> {
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

    let changed = context.with_mut(|context| context.reduce(f));

    // Only notify subscribers if state has changed.
    if changed {
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

    use crate::util::Mrc;

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
        dispatch.reduce(|state| state.0 += 1);

        assert!(dispatch.get().0 == 2)
    }

    #[test]
    fn dispatch_reduce_callback_works() {
        let dispatch = Dispatch::<TestState>::new();
        let cb = dispatch.reduce_callback(|state| state.0 += 1);
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
            context::subscribe::<TestState, _>(move |_| flag.clone().with_mut(|flag| *flag = true))
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
            context::subscribe::<TestState, _>(move |_| flag.clone().with_mut(|flag| *flag = true))
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
    fn dispatch_clone_does_not_unsubscribes_when_dropped() {
        let context = context::get_or_init::<TestState>();

        assert!(context.borrow().subscribers.is_empty());

        let dispatch = Dispatch::<TestState>::subscribe(|_| ());
        let dispatch_clone = dispatch.clone();

        assert!(!context.borrow().subscribers.is_empty());

        drop(dispatch_clone);

        assert!(!context.borrow().subscribers.is_empty());
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
