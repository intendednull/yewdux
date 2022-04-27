use std::rc::Rc;

use crate::{dispatch, mrc::Mrc, store::Store, subscriber::SubscriberId};

pub trait Listener: 'static {
    type Store: Store;

    fn on_change(&mut self, state: Rc<Self::Store>);
}

struct ListenerStore<S: Store>(Option<SubscriberId<S>>);
impl<S: Store> Store for Mrc<ListenerStore<S>> {
    fn new() -> Self {
        ListenerStore(None).into()
    }
}

pub fn init_listener<L: Listener>(listener: L) {
    let id = {
        let listener = Mrc::new(listener);
        dispatch::subscribe(move |state| listener.borrow_mut().on_change(state))
    };

    dispatch::reduce_mut(|state: &mut Mrc<ListenerStore<L::Store>>| {
        state.borrow_mut().0 = Some(id)
    });
}

#[cfg(test)]
mod tests {

    use std::cell::Cell;

    use super::*;

    #[derive(Clone, PartialEq)]
    struct TestState(u32);
    impl Store for TestState {
        fn new() -> Self {
            Self(0)
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
}
