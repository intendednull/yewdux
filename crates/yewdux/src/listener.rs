use std::rc::Rc;

use crate::store::Store;

pub trait Listener<S: Store> {
    fn changed(&self, state: Rc<S>);
}

struct ListenerType<T>(T);
