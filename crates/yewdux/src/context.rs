use std::{marker::PhantomData, rc::Rc};

use anymap::AnyMap;
use slab::Slab;

use crate::{
    store::Store,
    util::{Callable, Shared},
};

thread_local! {
    /// Stores all shared state.
    static CONTEXTS: Shared<AnyMap> = Shared::new(AnyMap::new());
}

pub(crate) struct Context<S> {
    pub(crate) store: Rc<S>,
    pub(crate) subscribers: Slab<Box<dyn Callable<S>>>,
}

impl<S: Store> Context<S> {
    pub(crate) fn reduce(&mut self, f: impl FnOnce(&mut S)) {
        let store = Rc::make_mut(&mut self.store);

        f(store);

        store.changed();
    }

    pub(crate) fn subscribe(&mut self, on_change: impl Callable<S>) -> SubscriberId<S> {
        let key = self.subscribers.insert(Box::new(on_change));
        SubscriberId {
            key,
            _store_type: Default::default(),
        }
    }

    pub(crate) fn unsubscribe(&mut self, id: usize) {
        self.subscribers.remove(id);
    }

    pub(crate) fn notify_subscribers(&self) {
        for (_, subscriber) in &self.subscribers {
            subscriber.call(Rc::clone(&self.store));
        }
    }
}

pub(crate) fn get_or_init<S: Store>() -> Shared<Context<S>> {
    let mut contexts = CONTEXTS
        .try_with(|context| context.clone())
        .expect("Thread local key init failed");

    contexts.with_mut(|contexts| {
        contexts
            .entry::<Shared<Context<S>>>()
            .or_insert_with(|| {
                Shared::new(Context {
                    store: Rc::new(S::new()),
                    subscribers: Default::default(),
                })
            })
            .clone()
    })
}

pub(crate) fn subscribe<S: Store, N: Callable<S>>(subscriber: N) -> SubscriberId<S> {
    let mut context = get_or_init::<S>();
    context.with_mut(|context| context.subscribe(subscriber))
}

pub(crate) fn unsubscribe<S: Store>(id: usize) {
    let mut context = get_or_init::<S>();
    context.with_mut(|context| context.unsubscribe(id))
}

/// Points to a subscriber in context. That subscriber is removed when this is dropped.
#[derive(Debug)]
pub(crate) struct SubscriberId<S: Store> {
    key: usize,
    _store_type: PhantomData<S>,
}

impl<S: Store> Drop for SubscriberId<S> {
    fn drop(&mut self) {
        unsubscribe::<S>(self.key);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dispatch::*;

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

    #[test]
    fn store_changed_is_called() {
        let mut context = get_or_init::<TestState>();

        context.with_mut(|context| context.reduce(|_| {}));

        assert!(context.borrow().store.0 == 1);
    }

    #[test]
    fn subscribe_adds_to_list() {
        let mut context = get_or_init::<TestState>();

        assert!(context.borrow().subscribers.is_empty());

        let _id = context.with_mut(|x| x.subscribe(|_| {}));

        assert!(!context.borrow().subscribers.is_empty());
    }

    #[test]
    fn unsubscribe_removes_from_list() {
        let mut context = get_or_init::<TestState>();

        assert!(context.borrow().subscribers.is_empty());

        let id = context.with_mut(|x| x.subscribe(|_| {}));

        assert!(!context.borrow().subscribers.is_empty());

        drop(id);

        assert!(context.borrow().subscribers.is_empty());
    }

    #[test]
    fn subscriber_is_notified() {
        let flag = Shared::new(false);

        let _id = {
            let flag = flag.clone();
            subscribe::<TestState, _>(move |_| flag.clone().with_mut(|flag| *flag = true))
        };

        reduce::<TestState, _>(|_| {});

        assert!(*flag.borrow());
    }

    #[test]
    fn dispatch_unsubscribes_when_dropped() {
        let context = get_or_init::<TestState>();

        assert!(context.borrow().subscribers.is_empty());

        let dispatch = Dispatch::<TestState>::subscribe(|_| {});

        assert!(!context.borrow().subscribers.is_empty());

        drop(dispatch);

        assert!(context.borrow().subscribers.is_empty());
    }
}
