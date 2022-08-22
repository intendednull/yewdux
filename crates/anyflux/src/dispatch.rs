//!  This module defines how you can interact with your [`Store`].

use std::rc::Rc;
#[cfg(feature = "future")]
use std::{future::Future, pin::Pin};

use crate::{
    context,
    mrc::Mrc,
    store::{Reducer, Store},
    subscriber::{Notify, SubscriberId, Subscribers},
};

/// The primary interface to a [`Store`].
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

    /// Create a dispatch that subscribes to changes in state. Latest state is sent immediately,
    /// and on every subsequent change. Automatically unsubscribes when this dispatch is dropped.
    pub fn subscribe<C: Notify<S>>(on_change: C) -> Self {
        let id = subscribe(on_change);

        Self {
            _subscriber_id: Some(Rc::new(id)),
        }
    }

    /// Create a dispatch that subscribes to changes in state. Similar to [Self::subscribe],
    /// however state is **not** sent immediately. Automatically unsubscribes when this dispatch is
    /// dropped.
    pub fn subscribe_silent<C: Notify<S>>(on_change: C) -> Self {
        let id = subscribe_silent(on_change);

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

    /// Set state to given value.
    pub fn set(&self, val: S) {
        set(val);
    }

    /// Mutate state with given function.
    ///
    /// ```ignore
    /// let onclick = dispatch.reduce(|state| State { count: state.count + 1 });
    /// ```
    pub fn reduce<F, R>(&self, f: F)
    where
        R: Into<Rc<S>>,
        F: FnOnce(Rc<S>) -> R,
    {
        reduce(f);
    }

    /// Mutate state with a given function, asynchronously.
    ///
    /// ```ignore
    /// dispatch.reduce_future(|state| async move { ... }).await;
    /// ```
    #[cfg(feature = "future")]
    pub async fn reduce_future<R, FUT, FUN>(&self, f: FUN)
    where
        R: Into<Rc<S>>,
        FUT: Future<Output = R>,
        FUN: FnOnce(Rc<S>) -> FUT,
    {
        reduce_future(f).await;
    }

    /// Mutate state with given function.
    ///
    /// ```ignore
    /// let onclick = dispatch.reduce_mut(|state| state.count += 1);
    /// ```
    pub fn reduce_mut<F, R>(&self, f: F)
    where
        S: Clone,
        F: FnOnce(&mut S) -> R,
    {
        reduce_mut(|x| {
            f(x);
        });
    }

    /// Mutate state with given function, in an async context.
    ///
    /// ```ignore
    /// dispatch.reduce_mut_future(|state| Box::pin(async move { *state = ... })).await;
    /// ```
    #[cfg(feature = "future")]
    pub async fn reduce_mut_future<R, F>(&self, f: F)
    where
        S: Clone,
        F: FnOnce(&mut S) -> Pin<Box<dyn Future<Output = R> + '_>>,
    {
        reduce_mut_future(f).await;
    }
}

impl<S: Store> Default for Dispatch<S> {
    fn default() -> Self {
        Self::new()
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
    let context = context::get_or_init::<S>();
    let should_notify = context.reduce(|s| f(s).into());

    if should_notify {
        let state = Rc::clone(&context.store.borrow());
        notify_subscribers(state)
    }
}

#[cfg(feature = "future")]
pub async fn reduce_future<S, R, FUT, FUN>(f: FUN)
where
    S: Store,
    R: Into<Rc<S>>,
    FUT: Future<Output = R>,
    FUN: FnOnce(Rc<S>) -> FUT,
{
    let context = context::get_or_init::<S>();
    let should_notify = context
        .reduce_future(|s| async move { f(s).await.into() })
        .await;

    if should_notify {
        let state = Rc::clone(&context.store.borrow());
        notify_subscribers(state)
    }
}

/// Change state using a mutable reference from a function.
pub fn reduce_mut<S: Store + Clone, F: FnOnce(&mut S)>(f: F) {
    reduce(|mut state| {
        f(Rc::make_mut(&mut state));
        state
    });
}

#[cfg(feature = "future")]
pub async fn reduce_mut_future<S, R, F>(f: F)
where
    S: Store + Clone,
    F: FnOnce(&mut S) -> Pin<Box<dyn Future<Output = R> + '_>>,
{
    reduce_future(|mut state| async move {
        f(Rc::make_mut(&mut state)).await;
        state
    })
    .await;
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
    Rc::clone(&context::get_or_init::<S>().store.borrow())
}

/// Send state to all subscribers.
pub fn notify_subscribers<S: Store>(state: Rc<S>) {
    let context = context::get_or_init::<Mrc<Subscribers<S>>>();
    context.store.borrow().notify(state);
}

/// Subscribe to a store. `on_change` is called immediately, then every  time state changes.
pub fn subscribe<S: Store, N: Notify<S>>(on_change: N) -> SubscriberId<S> {
    // Notify subscriber with inital state.
    on_change.call(get::<S>());

    context::get_or_init::<Mrc<Subscribers<S>>>()
        .store
        .borrow()
        .subscribe(on_change)
}

/// Similar to [subscribe], however state is not called immediately.
pub fn subscribe_silent<S: Store, N: Notify<S>>(on_change: N) -> SubscriberId<S> {
    context::get_or_init::<Mrc<Subscribers<S>>>()
        .store
        .borrow()
        .subscribe(on_change)
}

#[cfg(test)]
mod tests {

    use crate::mrc::Mrc;

    use super::*;

    #[derive(Clone, PartialEq, Eq)]
    struct TestState(u32);
    impl Store for TestState {
        fn new() -> Self {
            Self(0)
        }

        fn should_notify(&self, other: &Self) -> bool {
            self != other
        }
    }
    #[derive(PartialEq, Eq)]
    struct TestStateNoClone(u32);
    impl Store for TestStateNoClone {
        fn new() -> Self {
            Self(0)
        }

        fn should_notify(&self, other: &Self) -> bool {
            self != other
        }
    }

    struct Msg;
    impl Reducer<TestState> for Msg {
        fn apply(&self, state: Rc<TestState>) -> Rc<TestState> {
            TestState(state.0 + 1).into()
        }
    }

    #[test]
    fn apply_no_clone() {
        reduce(|_| TestStateNoClone(1));
    }

    #[test]
    fn reduce_changes_value() {
        let old = get::<TestState>();

        reduce(|_| TestState(1));

        let new = get::<TestState>();

        assert!(old != new);
    }

    #[cfg(feature = "future")]
    #[async_std::test]
    async fn reduce_future_changes_value() {
        let old = get::<TestState>();

        reduce_future(|state: Rc<TestState>| async move { TestState(state.0 + 1) }).await;

        let new = get::<TestState>();

        assert!(old != new);
    }

    #[test]
    fn reduce_mut_changes_value() {
        let old = get::<TestState>();

        reduce_mut(|state| *state = TestState(1));

        let new = get::<TestState>();

        assert!(old != new);
    }

    #[cfg(feature = "future")]
    #[async_std::test]
    async fn reduce_mut_future_changes_value() {
        let old = get::<TestState>();

        reduce_mut_future(|state| Box::pin(async move { *state = TestState(1) })).await;

        let new = get::<TestState>();

        assert!(old != new);
    }

    #[test]
    fn reduce_does_not_require_static() {
        let val = "1".to_string();
        reduce(|_| TestState(val.parse().unwrap()));
    }

    #[test]
    fn reduce_mut_does_not_require_static() {
        let val = "1".to_string();
        reduce_mut(|state: &mut TestState| state.0 = val.parse().unwrap());
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
        let old = dispatch.get();

        dispatch.set(TestState(1));

        assert!(dispatch.get() != old)
    }

    #[test]
    fn dispatch_reduce_mut_works() {
        let dispatch = Dispatch::<TestState>::new();
        let old = dispatch.get();

        dispatch.reduce_mut(|state| state.0 += 1);

        assert!(dispatch.get() != old)
    }

    #[cfg(feature = "future")]
    #[async_std::test]
    async fn dispatch_reduce_mut_future_works() {
        let dispatch = Dispatch::<TestState>::new();
        let old = dispatch.get();

        dispatch
            .reduce_mut_future(|state| Box::pin(async move { state.0 += 1 }))
            .await;

        assert!(dispatch.get() != old)
    }

    #[test]
    fn dispatch_reduce_works() {
        let dispatch = Dispatch::<TestState>::new();
        let old = dispatch.get();

        dispatch.reduce(|_| TestState(1));

        assert!(dispatch.get() != old)
    }

    #[cfg(feature = "future")]
    #[async_std::test]
    async fn dispatch_reduce_future_works() {
        let dispatch = Dispatch::<TestState>::new();
        let old = dispatch.get();

        dispatch
            .reduce_future(|state| async move { TestState(state.0 + 1) })
            .await;

        assert!(dispatch.get() != old)
    }

    #[test]
    fn dispatch_apply_works() {
        let dispatch = Dispatch::<TestState>::new();
        let old = dispatch.get();

        dispatch.apply(Msg);

        assert!(dispatch.get() != old)
    }

    #[test]
    fn subscriber_is_notified() {
        let flag = Mrc::new(false);

        let _id = {
            let flag = flag.clone();
            subscribe::<TestState, _>(move |_| flag.clone().with_mut(|flag| *flag = true))
        };

        *flag.borrow_mut() = false;

        reduce_mut::<TestState, _>(|state| state.0 += 1);

        assert!(*flag.borrow());
    }

    #[test]
    fn subscriber_is_not_notified_when_state_is_same() {
        let flag = Mrc::new(false);

        // TestState(1)
        reduce_mut::<TestState, _>(|_| {});

        let _id = {
            let flag = flag.clone();
            subscribe::<TestState, _>(move |_| flag.clone().with_mut(|flag| *flag = true))
        };

        *flag.borrow_mut() = false;

        // TestState(1)
        reduce_mut::<TestState, _>(|state| state.0 = 0);

        assert!(!*flag.borrow());
    }

    #[test]
    fn dispatch_unsubscribes_when_dropped() {
        let context = context::get_or_init::<Mrc<Subscribers<TestState>>>();

        assert!(context.store.borrow().borrow().0.is_empty());

        let dispatch = Dispatch::<TestState>::subscribe(|_| ());

        assert!(!context.store.borrow().borrow().0.is_empty());

        drop(dispatch);

        assert!(context.store.borrow().borrow().0.is_empty());
    }

    #[test]
    fn dispatch_clone_and_original_unsubscribe_when_both_dropped() {
        let context = context::get_or_init::<Mrc<Subscribers<TestState>>>();

        assert!(context.store.borrow().borrow().0.is_empty());

        let dispatch = Dispatch::<TestState>::subscribe(|_| ());
        let dispatch_clone = dispatch.clone();

        assert!(!context.store.borrow().borrow().0.is_empty());

        drop(dispatch_clone);

        assert!(!context.store.borrow().borrow().0.is_empty());

        drop(dispatch);

        assert!(context.store.borrow().borrow().0.is_empty());
    }
}
