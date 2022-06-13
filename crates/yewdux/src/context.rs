#[cfg(feature = "future")]
use std::future::Future;
use std::rc::Rc;

use anymap::AnyMap;

use crate::{mrc::Mrc, store::Store};

pub(crate) struct Context<S> {
    pub(crate) state: Mrc<Rc<S>>,
}

impl<S> Clone for Context<S> {
    fn clone(&self) -> Self {
        Self {
            state: self.state.clone(),
        }
    }
}

impl<S: Store> Context<S> {
    /// Apply a function to state, returning if it has changed or not.
    pub(crate) fn reduce(&self, f: impl FnOnce(Rc<S>) -> Rc<S>) -> bool {
        let old = Rc::clone(&self.state.borrow());
        *self.state.borrow_mut() = f(Rc::clone(&old));

        self.state.borrow().changed(&old)
    }

    /// Apply a future reduction to state, returning if it has changed or not.
    #[cfg(feature = "future")]
    pub(crate) async fn reduce_future<FUN, FUT>(&self, f: FUN) -> bool
    where
        FUN: FnOnce(Rc<S>) -> FUT,
        FUT: Future<Output = Rc<S>>,
    {
        let old = Rc::clone(&self.state.borrow());

        *self.state.borrow_mut() = f(Rc::clone(&old)).await;

        self.state.borrow().changed(&old)
    }
}

pub(crate) fn get_or_init<S: Store>() -> Context<S> {
    thread_local! {
        /// Stores all shared state.
        static CONTEXTS: Mrc<AnyMap> = Mrc::new(AnyMap::new());
    }

    let contexts = CONTEXTS
        .try_with(|contexts| contexts.clone())
        .expect("CONTEXTS thread local key init failed");

    // Init store outside of context borrow. This allows `Store::new` to access other stores if
    // needed.
    let context_exists = contexts.borrow().contains::<Context<S>>();
    let state = (!context_exists).then(|| Mrc::new(Rc::new(S::new())));

    contexts.with_mut(|x| {
        x.entry::<Context<S>>()
            .or_insert_with(|| Context {
                state: state.expect("Store not initialized"),
            })
            .clone()
    })
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use super::*;

    #[derive(Clone, PartialEq, Eq)]
    struct TestState(u32);
    impl Store for TestState {
        fn new() -> Self {
            Self(0)
        }

        fn changed(&self, other: &Self) -> bool {
            self != other
        }
    }

    #[derive(Clone, PartialEq, Eq)]
    struct TestState2(u32);
    impl Store for TestState2 {
        fn new() -> Self {
            get_or_init::<TestState>();
            Self(0)
        }

        fn changed(&self, other: &Self) -> bool {
            self != other
        }
    }

    #[test]
    fn can_access_other_store_for_new_of_current_store() {
        let _context = get_or_init::<TestState2>();
    }

    #[derive(Clone, PartialEq, Eq)]
    struct StoreNewIsOnlyCalledOnce(Rc<Cell<u32>>);
    impl Store for StoreNewIsOnlyCalledOnce {
        fn new() -> Self {
            thread_local! {
                /// Stores all shared state.
                static COUNT: Rc<Cell<u32>> = Default::default();
            }

            let count = COUNT.try_with(|x| x.clone()).unwrap();

            count.set(count.get() + 1);

            Self(count)
        }

        fn changed(&self, other: &Self) -> bool {
            self != other
        }
    }

    #[test]
    fn store_new_is_only_called_once() {
        get_or_init::<StoreNewIsOnlyCalledOnce>();
        let context = get_or_init::<StoreNewIsOnlyCalledOnce>();

        assert!(context.state.borrow().0.get() == 1)
    }
}
