use std::rc::Rc;

use yew::agent::HandlerId;

use super::{ShouldNotify, Store, StoreLink};

pub trait Reducer {
    type Action;

    fn reduce(&mut self, action: Self::Action) -> ShouldNotify;
    fn new() -> Self;
}

pub struct ReducerStore<T>
where
    T: Reducer + Clone,
{
    state: Rc<T>,
}

impl<T> Store for ReducerStore<T>
where
    T: Reducer + Clone + 'static,
{
    type Model = T;
    type Message = ();
    type Input = T::Action;
    type Output = ();

    fn new(_link: StoreLink<Self>) -> Self {
        Self {
            state: Rc::new(T::new()),
        }
    }

    fn state_mut(&mut self) -> &mut Self::Model {
        Rc::make_mut(&mut self.state)
    }

    fn state(&self) -> Rc<Self::Model> {
        Rc::clone(&self.state)
    }

    fn handle_input(&mut self, msg: Self::Input, _who: HandlerId) -> ShouldNotify {
        let state = Rc::make_mut(&mut self.state);
        state.reduce(msg)
    }
}
