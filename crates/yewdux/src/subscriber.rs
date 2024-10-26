use std::rc::Rc;
use std::{any::Any, marker::PhantomData};

use slab::Slab;
use yew::Callback;

use crate::{mrc::Mrc, store::Store, Context};

pub(crate) struct Subscribers<S>(pub(crate) Slab<Box<dyn Callable<S>>>);

impl<S: 'static> Store for Subscribers<S> {
    fn new(_cx: &Context) -> Self {
        Self(Default::default())
    }

    fn should_notify(&self, other: &Self) -> bool {
        self != other
    }
}

impl<S: Store> Mrc<Subscribers<S>> {
    pub(crate) fn subscribe<C: Callable<S>>(&self, on_change: C) -> SubscriberId<S> {
        let key = self.borrow_mut().0.insert(Box::new(on_change));
        SubscriberId {
            subscribers_ref: self.clone(),
            key,
            _store_type: Default::default(),
        }
    }

    pub(crate) fn unsubscribe(&mut self, key: usize) {
        self.borrow_mut().0.remove(key);
    }

    pub(crate) fn notify(&self, state: Rc<S>) {
        for (_, subscriber) in &self.borrow().0 {
            subscriber.call(Rc::clone(&state));
        }
    }
}

impl<S> PartialEq for Subscribers<S> {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl<S> Default for Subscribers<S> {
    fn default() -> Self {
        Self(Default::default())
    }
}

/// Points to a subscriber in context. That subscriber is removed when this is dropped.
pub struct SubscriberId<S: Store> {
    subscribers_ref: Mrc<Subscribers<S>>,
    pub(crate) key: usize,
    pub(crate) _store_type: PhantomData<S>,
}

impl<S: Store> std::fmt::Debug for SubscriberId<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SubscriberId")
            .field("key", &self.key)
            .finish()
    }
}

impl<S: Store> SubscriberId<S> {
    /// Leak this subscription, so it is never dropped.
    pub fn leak(self) {
        thread_local! {
            static LEAKED: Mrc<Vec<Box<dyn Any>>> = Default::default();
        }

        LEAKED
            .try_with(|leaked| leaked.clone())
            .expect("LEAKED thread local key init failed")
            .with_mut(|leaked| leaked.push(Box::new(self)));
    }
}

impl<S: Store> Drop for SubscriberId<S> {
    fn drop(&mut self) {
        self.subscribers_ref.unsubscribe(self.key)
    }
}

pub trait Callable<S>: 'static {
    fn call(&self, value: Rc<S>);
}

impl<S, F: Fn(Rc<S>) + 'static> Callable<S> for F {
    fn call(&self, value: Rc<S>) {
        self(value)
    }
}

impl<S: 'static> Callable<S> for Callback<Rc<S>> {
    fn call(&self, value: Rc<S>) {
        self.emit(value)
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    use crate::context::Context;
    use crate::dispatch::Dispatch;
    use crate::mrc::Mrc;

    #[derive(Clone, PartialEq, Eq)]
    struct TestState(u32);
    impl Store for TestState {
        fn new(_cx: &Context) -> Self {
            Self(0)
        }

        fn should_notify(&self, other: &Self) -> bool {
            self != other
        }
    }

    #[test]
    fn subscribe_adds_to_list() {
        let cx = Context::new();
        let entry = cx.get_or_init_default::<Mrc<Subscribers<TestState>>>();

        assert!(entry.store.borrow().borrow().0.is_empty());

        let _id = Dispatch::new(&cx).subscribe(|_: Rc<TestState>| ());

        assert!(!entry.store.borrow().borrow().0.is_empty());
    }

    #[test]
    fn unsubscribe_removes_from_list() {
        let cx = Context::new();
        let entry = cx.get_or_init_default::<Mrc<Subscribers<TestState>>>();

        assert!(entry.store.borrow().borrow().0.is_empty());

        let id = Dispatch::new(&cx).subscribe(|_: Rc<TestState>| ());

        assert!(!entry.store.borrow().borrow().0.is_empty());

        drop(id);

        assert!(entry.store.borrow().borrow().0.is_empty());
    }

    #[test]
    fn subscriber_id_unsubscribes_when_dropped() {
        let cx = Context::new();
        let entry = cx.get_or_init_default::<Mrc<Subscribers<TestState>>>();

        assert!(entry.store.borrow().borrow().0.is_empty());

        let id = Dispatch::<TestState>::new(&cx).subscribe(|_| {});

        assert!(!entry.store.borrow().borrow().0.is_empty());

        drop(id);

        assert!(entry.store.borrow().borrow().0.is_empty());
    }

    #[test]
    fn subscriber_is_notified_on_subscribe() {
        let flag = Mrc::new(false);
        let cx = Context::new();

        let _id = {
            let flag = flag.clone();
            Dispatch::<TestState>::new(&cx)
                .subscribe(move |_| flag.clone().with_mut(|flag| *flag = true))
        };

        assert!(*flag.borrow());
    }

    #[test]
    fn subscriber_is_notified_after_leak() {
        let flag = Mrc::new(false);
        let cx = Context::new();

        let id = {
            let flag = flag.clone();
            cx.subscribe::<TestState, _>(move |_| flag.clone().with_mut(|flag| *flag = true))
        };

        *flag.borrow_mut() = false;

        id.leak();

        cx.reduce_mut(|state: &mut TestState| state.0 += 1);

        assert!(*flag.borrow());
    }

    #[test]
    fn can_modify_state_inside_on_changed() {
        let cx = Context::new();
        let cxo = cx.clone();
        let dispatch = Dispatch::<TestState>::new(&cx).subscribe(move |state: Rc<TestState>| {
            if state.0 == 0 {
                Dispatch::new(&cxo).reduce_mut(|state: &mut TestState| state.0 += 1);
            }
        });

        assert_eq!(dispatch.get().0, 1)
    }
}
