use std::marker::PhantomData;
use std::rc::Rc;

use yew::Callback;

use crate::{context, store::Store};

pub(crate) fn subscribe<S: Store, N: Callable<S>>(on_change: N) -> SubscriberId<S> {
    let mut context = context::get_or_init::<S>();
    context.with_mut(|context| context.subscribe(on_change))
}

pub(crate) fn unsubscribe<S: Store>(id: usize) {
    let mut context = context::get_or_init::<S>();
    context.with_mut(|context| context.unsubscribe(id))
}

/// Points to a subscriber in context. That subscriber is removed when this is dropped.
#[derive(Debug)]
pub(crate) struct SubscriberId<S: Store> {
    pub(crate) key: usize,
    pub(crate) _store_type: PhantomData<S>,
}

impl<S: Store> Drop for SubscriberId<S> {
    fn drop(&mut self) {
        unsubscribe::<S>(self.key);
    }
}

pub trait Callable<S>: 'static {
    fn call(&mut self, value: Rc<S>);
}

impl<S, F: FnMut(Rc<S>) + 'static> Callable<S> for F {
    fn call(&mut self, value: Rc<S>) {
        self(value)
    }
}

impl<S: 'static> Callable<S> for Callback<Rc<S>> {
    fn call(&mut self, value: Rc<S>) {
        self.emit(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::mrc::Mrc;

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
    fn subscribe_adds_to_list() {
        let mut context = context::get_or_init::<TestState>();

        assert!(context.borrow().subscribers.is_empty());

        let _id = context.with_mut(|x| x.subscribe(|_| {}));

        assert!(!context.borrow().subscribers.is_empty());
    }

    #[test]
    fn unsubscribe_removes_from_list() {
        let mut context = context::get_or_init::<TestState>();

        assert!(context.borrow().subscribers.is_empty());

        let id = context.with_mut(|x| x.subscribe(|_| {}));

        assert!(!context.borrow().subscribers.is_empty());

        drop(id);

        assert!(context.borrow().subscribers.is_empty());
    }

    #[test]
    fn subscriber_id_unsubscribes_when_dropped() {
        let context = context::get_or_init::<TestState>();

        assert!(context.borrow().subscribers.is_empty());

        let id = subscribe::<TestState, _>(|_| {});

        assert!(!context.borrow().subscribers.is_empty());

        drop(id);

        assert!(context.borrow().subscribers.is_empty());
    }

    #[test]
    fn subscriber_is_notified_on_subscribe() {
        let flag = Mrc::new(false);

        let _id = {
            let flag = flag.clone();
            subscribe::<TestState, _>(move |_| flag.clone().with_mut(|flag| *flag = true))
        };

        assert!(*flag.borrow());
    }
}
