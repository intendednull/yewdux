use std::rc::Rc;
use std::{any::Any, marker::PhantomData};

use slab::Slab;
use yew::Callback;

use crate::mrc::Mrc;
use crate::store::Store;

pub(crate) struct Subscribers<S>(pub(crate) Slab<Rc<dyn Callable<S>>>);

impl<S: 'static> Store for Subscribers<S> {
    fn new() -> Self {
        Self(Default::default())
    }

    fn should_notify(&self, other: &Self) -> bool {
        self != other
    }
}

impl<S: Store> Mrc<Subscribers<S>> {
    pub(crate) fn subscribe<C: Callable<S>>(&self, on_change: C) -> SubscriberId<S> {
        let key = self.borrow_mut().0.insert(Rc::new(on_change));
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
        let subscribers = self
            .borrow()
            .0
            .iter()
            .map(|(_, subscriber)| subscriber.clone())
            .collect::<Vec<_>>();

        for subscriber in subscribers {
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

    use crate::context;
    use crate::dispatch::{self, Dispatch};
    use crate::mrc::Mrc;

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

    #[test]
    fn subscribe_adds_to_list() {
        let context = context::get_or_init::<Mrc<Subscribers<TestState>>>();

        assert!(context.store.borrow().borrow().0.is_empty());

        let _id = dispatch::subscribe(|_: Rc<TestState>| ());

        assert!(!context.store.borrow().borrow().0.is_empty());
    }

    #[test]
    fn unsubscribe_removes_from_list() {
        let context = context::get_or_init::<Mrc<Subscribers<TestState>>>();

        assert!(context.store.borrow().borrow().0.is_empty());

        let id = dispatch::subscribe(|_: Rc<TestState>| ());

        assert!(!context.store.borrow().borrow().0.is_empty());

        drop(id);

        assert!(context.store.borrow().borrow().0.is_empty());
    }

    #[test]
    fn subscriber_id_unsubscribes_when_dropped() {
        let context = context::get_or_init::<Mrc<Subscribers<TestState>>>();

        assert!(context.store.borrow().borrow().0.is_empty());

        let id = dispatch::subscribe::<TestState, _>(|_| {});

        assert!(!context.store.borrow().borrow().0.is_empty());

        drop(id);

        assert!(context.store.borrow().borrow().0.is_empty());
    }

    #[test]
    fn subscriber_is_notified_on_subscribe() {
        let flag = Mrc::new(false);

        let _id = {
            let flag = flag.clone();
            dispatch::subscribe::<TestState, _>(move |_| flag.clone().with_mut(|flag| *flag = true))
        };

        assert!(*flag.borrow());
    }

    #[test]
    fn subscriber_is_notified_after_leak() {
        let flag = Mrc::new(false);

        let id = {
            let flag = flag.clone();
            dispatch::subscribe::<TestState, _>(move |_| flag.clone().with_mut(|flag| *flag = true))
        };

        *flag.borrow_mut() = false;

        id.leak();

        dispatch::reduce_mut(|state: &mut TestState| state.0 += 1);

        assert!(*flag.borrow());
    }

    #[test]
    fn can_modify_state_inside_on_changed() {
        let dispatch = Dispatch::<TestState>::subscribe(|state: Rc<TestState>| {
            if state.0 == 0 {
                dispatch::reduce_mut(|state: &mut TestState| state.0 += 1);
            }
        });

        assert_eq!(dispatch.get().0, 1)
    }
}
