use std::rc::Rc;

use super::{Store, StoreLink};

/// Handler for basic shared state.
#[derive(Default, Clone)]
pub struct BasicStore<T> {
    state: Rc<T>,
}

impl<T> Store for BasicStore<T>
where
    T: Clone + Default + 'static,
{
    type Model = T;
    type Message = ();
    type Input = ();
    type Output = ();

    fn new(_link: StoreLink<Self>) -> Self {
        Default::default()
    }

    fn state_mut(&mut self) -> &mut Self::Model {
        Rc::make_mut(&mut self.state)
    }

    fn state(&self) -> Rc<Self::Model> {
        Rc::clone(&self.state)
    }
}
