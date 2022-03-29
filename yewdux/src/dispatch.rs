//!  This module defines how you can interact with your [`Store`].
//!
//!  ```
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
use std::{marker::PhantomData, rc::Rc};

use yew::Callback;

use crate::{context, store::Store, util::Callable};

/// The primary interface to a [`Store`].
#[derive(Debug, Default)]
pub struct Dispatch<S: Store> {
    subscriber_key: Option<usize>,
    store_type: PhantomData<S>,
}

impl<S: Store> Dispatch<S> {
    /// Create a new dispatch.
    pub fn new() -> Self {
        Self {
            subscriber_key: Default::default(),
            store_type: Default::default(),
        }
    }

    /// Create a dispatch, and subscribe to state changes.
    pub fn subscribe<C: Callable<S>>(on_change: C) -> Self {
        let key = subscribe(on_change);

        Self {
            subscriber_key: Some(key),
            store_type: Default::default(),
        }
    }

    /// Get the current state.
    pub fn get() -> Rc<S> {
        get::<S>()
    }

    /// Send a message to the store.
    pub fn send(&self, msg: impl Into<S::Message>) {
        send::<S>(msg.into());
    }

    /// Callback for sending a message to the store.
    ///
    /// ```ignore
    /// let onclick = dispatch.callback(|_| StoreMsg::AddOne);
    /// ```
    pub fn callback<E, M>(&self, f: impl Fn(E) -> M + 'static) -> Callback<E>
    where
        M: Into<S::Message>,
    {
        Callback::from(move |e| {
            let msg = f(e);
            send::<S>(msg.into());
        })
    }

    /// Set state to given value.
    pub fn set(val: S) {
        set(val);
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

impl<S: Store> Drop for Dispatch<S> {
    fn drop(&mut self) {
        if let Some(key) = self.subscriber_key {
            unsubscribe::<S>(key);
        }
    }
}

/// Change state using given function.
pub fn reduce<S: Store, F: FnOnce(&mut S)>(f: F) {
    let mut context = context::get_or_init::<S>();

    context.with_mut(|context| {
        context.reduce(f);
    });

    context.borrow().notify_subscribers();
}

/// Set state to given value.
pub fn set<S: Store>(value: S) {
    reduce(move |store| *store = value);
}

/// Send a message to state.
pub fn send<S: Store>(msg: S::Message) {
    reduce(move |store: &mut S| store.update(msg));
}

/// Get current state.
pub fn get<S: Store>() -> Rc<S> {
    Rc::clone(&context::get_or_init::<S>().borrow().store)
}

/// Subscribe to context. This should never be accessible to user code. See [`unsubscribe`].
fn subscribe<S: Store, N: Callable<S>>(subscriber: N) -> usize {
    let mut context = context::get_or_init::<S>();
    context.with_mut(|context| context.subscribe(subscriber))
}

/// Unsubscribe from context. This should never be accessible to user code. Calling unsubscribe
/// twice, in the best case scenario, will cause a panic. Worst case it incorrectly unsubscribes
/// some other subscriber, causing all sorts of problems. It's very important we tightly control
/// when exactly this is called.
fn unsubscribe<S: Store>(key: usize) {
    let mut context = context::get_or_init::<S>();
    context.with_mut(|context| context.unsubscribe(key))
}

#[cfg(test)]
mod tests {
    use crate::util::Shared;

    use super::*;

    #[derive(Clone, PartialEq)]
    struct TestState(u32);
    impl Store for TestState {
        type Message = ();

        fn new() -> Self {
            Self(0)
        }

        fn update(&mut self, _msg: Self::Message) {
            self.0 += 1;
        }

        fn changed(&mut self) {
            self.0 += 1;
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
    fn subscriber_is_notified() {
        let flag = Shared::new(false);

        {
            let flag = flag.clone();
            subscribe::<TestState, _>(move |_| flag.clone().with_mut(|flag| *flag = true));
        }

        reduce::<TestState, _>(|_| {});

        assert!(*flag.borrow());
    }

    #[test]
    fn store_update_is_called_on_send() {
        send::<TestState>(());

        assert!(get::<TestState>().0 == 2);
    }

    #[test]
    fn dispatch_unsubscribes_when_dropped() {
        let context = context::get_or_init::<TestState>();

        assert!(context.borrow().subscribers.is_empty());

        let dispatch = Dispatch::<TestState>::subscribe(|_| {});

        assert!(!context.borrow().subscribers.is_empty());

        drop(dispatch);

        assert!(context.borrow().subscribers.is_empty());
    }
}
