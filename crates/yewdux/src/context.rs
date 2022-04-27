use std::rc::Rc;

use anymap::AnyMap;
use slab::Slab;

use crate::{
    mrc::Mrc,
    store::Store,
    subscriber::{Callable, SubscriberId},
};

pub(crate) struct Context<S> {
    pub(crate) store: Rc<S>,
    pub(crate) subscribers: Slab<Box<dyn Callable<S>>>,
}

impl<S: Store> Context<S> {
    /// Apply a function to state, returning if it has changed or not.
    pub(crate) fn reduce(&mut self, f: impl FnOnce(Rc<S>) -> Rc<S>) -> bool {
        let old = Rc::clone(&self.store);
        self.store = f(Rc::clone(&old));

        let changed = old != self.store;
        if changed {
            self.notify_subscribers();
        }

        changed
    }

    pub(crate) fn subscribe(&mut self, mut on_change: impl Callable<S>) -> SubscriberId<S> {
        // Notify subscriber with inital state.
        on_change.call(Rc::clone(&self.store));

        let key = self.subscribers.insert(Box::new(on_change));
        SubscriberId {
            key,
            _store_type: Default::default(),
        }
    }

    pub(crate) fn unsubscribe(&mut self, id: usize) {
        self.subscribers.remove(id);
    }

    pub(crate) fn notify_subscribers(&mut self) {
        for (_, subscriber) in &mut self.subscribers {
            subscriber.call(Rc::clone(&self.store));
        }
    }
}

pub(crate) fn get_or_init<S: Store>() -> Mrc<Context<S>> {
    thread_local! {
        /// Stores all shared state.
        static CONTEXTS: Mrc<AnyMap> = Mrc::new(AnyMap::new());
    }

    // Init store outside of context borrow. This allows `Store::new` to access other stores if
    // needed.
    let store = Rc::new(S::new());
    CONTEXTS
        .try_with(|contexts| contexts.clone())
        .expect("CONTEXTS thread local key init failed")
        .with_mut(|contexts| {
            contexts
                .entry::<Mrc<Context<S>>>()
                .or_insert_with(|| {
                    Mrc::new(Context {
                        store,
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
        fn new() -> Self {
            Self(0)
        }

        fn changed(&mut self) {
            self.0 += 1;
        }
    }

    #[derive(Clone, PartialEq)]
    struct TestState2(u32);
    impl Store for TestState2 {
        fn new() -> Self {
            get_or_init::<TestState>();
            Self(0)
        }

        fn changed(&mut self) {
            self.0 += 1;
        }
    }

    #[test]
    fn store_changed_is_called() {
        let mut context = get_or_init::<TestState>();

        context.with_mut(|context| context.reduce(|state| TestState(state.0 + 1).into()));

        assert!(context.borrow().store.0 == 2);
    }

    #[test]
    fn store_changed_is_not_called_when_state_is_same() {
        let mut context = get_or_init::<TestState>();

        context.with_mut(|context| context.reduce(|x| x));

        assert!(context.borrow().store.0 == 0);
    }

    #[test]
    fn can_access_other_store_for_new_of_current_store() {
        let _context = get_or_init::<TestState2>();
    }
}
