use std::rc::Rc;

use crate::{context::Context, dispatch::Dispatch, mrc::Mrc, store::Store};

/// Listens to [Store](crate::store::Store) changes.
pub trait Listener: 'static {
    type Store: Store;

    fn on_change(&mut self, cx: &Context, state: Rc<Self::Store>);
}

struct ListenerStore<L: Listener>(Option<Dispatch<L::Store>>);
impl<L: Listener> Store for Mrc<ListenerStore<L>> {
    fn new(_cx: &Context) -> Self {
        ListenerStore(None).into()
    }

    fn should_notify(&self, other: &Self) -> bool {
        self != other
    }
}

/// Initiate a [Listener]. If this listener has already been initiated, it is dropped and replaced
/// with the new one.
pub fn init_listener<L: Listener>(listener: L, cx: &Context) {
    let dispatch = {
        let listener = Mrc::new(listener);
        let cxo = cx.clone();
        Dispatch::with_cx(cx)
            .subscribe_silent(move |state| listener.borrow_mut().on_change(&cxo, state))
    };

    cx.reduce_mut(|state: &mut Mrc<ListenerStore<L>>| state.borrow_mut().0 = Some(dispatch));
}

#[cfg(test)]
mod tests {

    use std::cell::Cell;

    use super::*;

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

    #[derive(Clone)]
    struct TestListener(Rc<Cell<u32>>);
    impl Listener for TestListener {
        type Store = TestState;

        fn on_change(&mut self, _cx: &Context, state: Rc<Self::Store>) {
            self.0.set(state.0);
        }
    }

    #[derive(Clone)]
    struct AnotherTestListener(Rc<Cell<u32>>);
    impl Listener for AnotherTestListener {
        type Store = TestState;

        fn on_change(&mut self, _cx: &Context, state: Rc<Self::Store>) {
            self.0.set(state.0);
        }
    }

    #[derive(Clone, PartialEq, Eq)]
    struct TestState2;
    impl Store for TestState2 {
        fn new(cx: &Context) -> Self {
            init_listener(TestListener2, cx);
            Self
        }

        fn should_notify(&self, other: &Self) -> bool {
            self != other
        }
    }

    #[derive(Clone)]
    struct TestListener2;
    impl Listener for TestListener2 {
        type Store = TestState2;

        fn on_change(&mut self, _cx: &Context, _state: Rc<Self::Store>) {}
    }

    #[test]
    fn listener_is_called() {
        let cx = Context::new();
        let listener = TestListener(Default::default());

        init_listener(listener.clone(), &cx);

        Dispatch::with_cx(&cx).reduce_mut(|state: &mut TestState| state.0 = 1);

        assert_eq!(listener.0.get(), 1)
    }

    #[test]
    fn listener_is_replaced() {
        let cx = Context::new();
        let listener1 = TestListener(Default::default());
        let listener2 = TestListener(Default::default());

        init_listener(listener1.clone(), &cx);

        Dispatch::with_cx(&cx).reduce_mut(|state: &mut TestState| state.0 = 1);

        assert_eq!(listener1.0.get(), 1);

        init_listener(listener2.clone(), &cx);

        Dispatch::with_cx(&cx).reduce_mut(|state: &mut TestState| state.0 = 2);

        assert_eq!(listener1.0.get(), 1);
        assert_eq!(listener2.0.get(), 2);
    }

    #[test]
    fn listener_of_different_type_is_not_replaced() {
        let cx = Context::new();
        let listener1 = TestListener(Default::default());
        let listener2 = AnotherTestListener(Default::default());

        init_listener(listener1.clone(), &cx);

        cx.reduce_mut(|state: &mut TestState| state.0 = 1);

        assert_eq!(listener1.0.get(), 1);

        init_listener(listener2.clone(), &cx);

        cx.reduce_mut(|state: &mut TestState| state.0 = 2);

        assert_eq!(listener1.0.get(), 2);
        assert_eq!(listener2.0.get(), 2);
    }

    #[test]
    fn can_init_listener_from_store() {
        let cx = Context::new();
        cx.get::<TestState2>();
    }
}
