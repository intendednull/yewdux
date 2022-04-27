use std::rc::Rc;

use anymap::AnyMap;

use crate::{mrc::Mrc, store::Store, subscriber::Subscribers};

pub(crate) struct Context<S> {
    pub(crate) state: Mrc<Rc<S>>,
    pub(crate) subscribers: Mrc<Subscribers<S>>,
}

impl<S> Clone for Context<S> {
    fn clone(&self) -> Self {
        Self {
            state: self.state.clone(),
            subscribers: self.subscribers.clone(),
        }
    }
}

impl<S: Store> Context<S> {
    /// Apply a function to state, returning if it has changed or not.
    pub(crate) fn reduce(&self, f: impl FnOnce(Rc<S>) -> Rc<S>) -> bool {
        let old = Rc::clone(&self.state.borrow());
        *self.state.borrow_mut() = f(Rc::clone(&old));

        old.as_ref() != self.state.borrow().as_ref()
    }
}

pub(crate) fn get_or_init<S: Store>() -> Context<S> {
    thread_local! {
        /// Stores all shared state.
        static CONTEXTS: Mrc<AnyMap> = Mrc::new(AnyMap::new());
    }

    // Init store outside of context borrow. This allows `Store::new` to access other stores if
    // needed.
    let state = Mrc::new(Rc::new(S::new()));
    CONTEXTS
        .try_with(|contexts| contexts.clone())
        .expect("CONTEXTS thread local key init failed")
        .with_mut(|contexts| {
            contexts
                .entry::<Context<S>>()
                .or_insert_with(|| {
                    Context {
                        state,
                        subscribers: Default::default(),
                    }
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
    }

    #[derive(Clone, PartialEq)]
    struct TestState2(u32);
    impl Store for TestState2 {
        fn new() -> Self {
            get_or_init::<TestState>();
            Self(0)
        }
    }

    #[test]
    fn can_access_other_store_for_new_of_current_store() {
        let _context = get_or_init::<TestState2>();
    }
}
