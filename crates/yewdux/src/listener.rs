use std::rc::Rc;

use crate::{dispatch, mrc::Mrc, store::Store, subscriber::SubscriberId};

/// Listens to [Store](crate::store::Store) changes.
pub trait Listener: 'static {
    type Store: Store;

    fn on_change(&mut self, state: Rc<Self::Store>);
}

struct ListenerStore<S: Store>(Option<SubscriberId<S>>);
impl<S: Store> Store for Mrc<ListenerStore<S>> {
    fn new() -> Self {
        ListenerStore(None).into()
    }

    fn changed(&self, other: &Self) -> bool {
        self != other
    }
}

/// Initiate a [Listener]. If this listener has already been initiated, it is dropped and replaced
/// with the new one.
pub fn init_listener<L: Listener>(listener: L) {
    let id = {
        let listener = Mrc::new(listener);
        dispatch::subscribe_silent(move |state| listener.borrow_mut().on_change(state))
    };

    dispatch::reduce_mut(|state: &mut Mrc<ListenerStore<L::Store>>| {
        state.borrow_mut().0 = Some(id)
    });
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

    #[derive(Clone)]
    struct TestListener(Rc<Cell<u32>>);
    impl Listener for TestListener {
        type Store = TestState;

        fn on_change(&mut self, state: Rc<Self::Store>) {
            self.0.set(state.0);
        }
    }

    #[derive(Clone, PartialEq, Eq)]
    struct TestState2;
    impl Store for TestState2 {
        fn new() -> Self {
            init_listener(TestListener2);
            Self
        }

        fn changed(&self, other: &Self) -> bool {
            self != other
        }
    }

    #[derive(Clone)]
    struct TestListener2;
    impl Listener for TestListener2 {
        type Store = TestState2;

        fn on_change(&mut self, _state: Rc<Self::Store>) {}
    }

    #[test]
    fn listener_is_called() {
        let listener = TestListener(Default::default());

        init_listener(listener.clone());

        dispatch::reduce_mut(|state: &mut TestState| state.0 = 1);

        assert_eq!(listener.0.get(), 1)
    }

    #[test]
    fn listener_is_replaced() {
        let listener1 = TestListener(Default::default());
        let listener2 = TestListener(Default::default());

        init_listener(listener1.clone());

        dispatch::reduce_mut(|state: &mut TestState| state.0 = 1);

        assert_eq!(listener1.0.get(), 1);

        init_listener(listener2.clone());

        dispatch::reduce_mut(|state: &mut TestState| state.0 = 2);

        assert_eq!(listener1.0.get(), 1);
        assert_eq!(listener2.0.get(), 2);
    }

    #[test]
    fn can_init_listener_from_store() {
        dispatch::get::<TestState2>();
    }
}
