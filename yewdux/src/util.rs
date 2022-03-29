use std::{
    cell::RefCell,
    ops::{Deref, DerefMut},
    rc::Rc,
};

use yew::Callback;

pub(crate) struct Shared<T>(Rc<RefCell<T>>);

impl<T: 'static> Shared<T> {
    pub(crate) fn new(value: T) -> Self {
        Self(Rc::new(RefCell::new(value)))
    }

    pub(crate) fn with_mut<R>(&mut self, f: impl FnOnce(&mut T) -> R) -> R {
        let mut this = self.0.as_ref().borrow_mut();
        f(this.deref_mut())
    }

    pub(crate) fn borrow<'a>(&'a self) -> impl Deref<Target = T> + 'a {
        self.0.borrow()
    }
}

impl<T> Clone for Shared<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

pub trait Callable<S>: 'static {
    fn call(&self, value: Rc<S>);
}

impl<S, F: Fn(Rc<S>) + 'static> Callable<S> for F {
    fn call(&self, value: Rc<S>) {
        self(value)
    }
}

impl<S: 'static> Callable<S> for Callback<Rc<S>> {
    fn call(&self, value: Rc<S>) {
        self.emit(value)
    }
}
