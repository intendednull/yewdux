use std::rc::Rc;

use super::{Store, StoreLink};

/// Handler for basic shared state.
#[derive(Default, Clone)]
pub struct DefaultStore<T> {
    state: Rc<T>,
}

impl<T> Store for DefaultStore<T>
where
    T: Clone + Default,
{
    type Model = T;
    type Message = ();
    type Input = ();
    type Output = ();

    fn new(_link: StoreLink<Self>) -> Self {
        Default::default()
    }

    fn state(&mut self) -> &mut Rc<Self::Model> {
        &mut self.state
    }
}
