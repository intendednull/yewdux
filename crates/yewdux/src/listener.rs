use std::rc::Rc;

use crate::{context::Context, dispatch::Dispatch, store::Store};

/// Listens to [Store](crate::store::Store) changes.
pub trait Listener: 'static {
    type Store: Store;

    fn on_change(&self, cx: &Context, state: Rc<Self::Store>);
}

#[allow(unused)]
struct ListenerStore<L: Listener>(Dispatch<L::Store>);
impl<L: Listener> Store for ListenerStore<L> {
    fn new(_cx: &Context) -> Self {
        // This is a private type, and only ever constructed by `init_listener` with a manual
        // constructor, so this should never run.
        unreachable!()
    }

    fn should_notify(&self, _other: &Self) -> bool {
        false
    }
}

/// Initiate a [Listener]. Does nothing if listener is already initiated.
pub fn init_listener<L: Listener, F: FnOnce() -> L>(new_listener: F, cx: &Context) {
    cx.get_or_init(|cx| {
        let dispatch = {
            let listener = new_listener();
            let cx = cx.clone();
            Dispatch::new(&cx).subscribe_silent(move |state| listener.on_change(&cx, state))
        };

        ListenerStore::<L>(dispatch)
    });
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

        fn on_change(&self, _cx: &Context, state: Rc<Self::Store>) {
            self.0.set(state.0);
        }
    }

    #[derive(Clone)]
    struct AnotherTestListener(Rc<Cell<u32>>);
    impl Listener for AnotherTestListener {
        type Store = TestState;

        fn on_change(&self, _cx: &Context, state: Rc<Self::Store>) {
            self.0.set(state.0);
        }
    }

    #[derive(Clone, PartialEq, Eq)]
    struct TestState2;
    impl Store for TestState2 {
        fn new(cx: &Context) -> Self {
            init_listener(|| TestListener2, cx);
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

        fn on_change(&self, _cx: &Context, _state: Rc<Self::Store>) {}
    }

    #[derive(Clone, PartialEq, Eq)]
    struct TestStateRecursive(u32);
    impl Store for TestStateRecursive {
        fn new(_cx: &Context) -> Self {
            Self(0)
        }

        fn should_notify(&self, other: &Self) -> bool {
            self != other
        }
    }

    #[derive(Clone)]
    struct TestListenerRecursive;
    impl Listener for TestListenerRecursive {
        type Store = TestStateRecursive;

        fn on_change(&self, cx: &Context, state: Rc<Self::Store>) {
            let dispatch = Dispatch::<TestStateRecursive>::new(cx);
            if state.0 < 10 {
                dispatch.reduce_mut(|state| state.0 += 1);
            }
        }
    }

    #[test]
    fn recursion() {
        let cx = Context::new();
        init_listener(|| TestListenerRecursive, &cx);
        let dispatch = Dispatch::<TestStateRecursive>::new(&cx);
        dispatch.reduce_mut(|state| state.0 = 1);
        assert_eq!(dispatch.get().0, 10);
    }

    #[test]
    fn listener_is_called() {
        let cx = Context::new();
        let listener = TestListener(Default::default());

        init_listener(|| listener.clone(), &cx);

        Dispatch::new(&cx).reduce_mut(|state: &mut TestState| state.0 = 1);

        assert_eq!(listener.0.get(), 1)
    }

    #[test]
    fn listener_is_not_replaced() {
        let cx = Context::new();
        let listener1 = TestListener(Default::default());
        let listener2 = TestListener(Default::default());

        init_listener(|| listener1.clone(), &cx);

        Dispatch::new(&cx).reduce_mut(|state: &mut TestState| state.0 = 1);

        assert_eq!(listener1.0.get(), 1);

        init_listener(|| listener2.clone(), &cx);

        Dispatch::new(&cx).reduce_mut(|state: &mut TestState| state.0 = 2);

        assert_eq!(listener1.0.get(), 2);
        assert_eq!(listener2.0.get(), 0);
    }

    #[test]
    fn listener_of_different_type_is_not_replaced() {
        let cx = Context::new();
        let listener1 = TestListener(Default::default());
        let listener2 = AnotherTestListener(Default::default());

        init_listener(|| listener1.clone(), &cx);

        cx.reduce_mut(|state: &mut TestState| state.0 = 1);

        assert_eq!(listener1.0.get(), 1);

        init_listener(|| listener2.clone(), &cx);

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
