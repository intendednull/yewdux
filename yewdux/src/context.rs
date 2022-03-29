use std::rc::Rc;

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

    pub(crate) fn subscribe(&mut self, on_change: impl Callable<S>) -> usize {
        self.subscribers.insert(Box::new(on_change))
    }

    pub(crate) fn unsubscribe(&mut self, key: usize) {
        self.subscribers.remove(key);
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

#[cfg(test)]
mod tests {
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
    fn store_changed_is_called() {
        let mut context = get_or_init::<TestState>();

        context.with_mut(|context| context.reduce(|_| {}));

        assert!(context.borrow().store.0 == 1);
    }

    #[test]
    fn subscribe_adds_to_list() {
        let mut context = get_or_init::<TestState>();

        assert!(context.borrow().subscribers.is_empty());

        context.with_mut(|x| x.subscribe(|_| {}));

        assert!(!context.borrow().subscribers.is_empty());
    }

    #[test]
    fn unsubscribe_removes_from_list() {
        let mut context = get_or_init::<TestState>();

        assert!(context.borrow().subscribers.is_empty());

        let key = context.with_mut(|x| x.subscribe(|_| {}));

        assert!(!context.borrow().subscribers.is_empty());

        context.with_mut(|x| x.unsubscribe(key));

        assert!(context.borrow().subscribers.is_empty());
    }
}
