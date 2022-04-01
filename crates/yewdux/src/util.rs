use std::{
    cell::RefCell,
    ops::{Deref, DerefMut},
    rc::Rc,
};

use yew::Callback;

/// A cloneable wrapper type that provides interior mutability.
#[derive(Debug, Default)]
pub struct Mrc<T>(Rc<RefCell<T>>);

impl<T: 'static> Mrc<T> {
    pub(crate) fn new(value: T) -> Self {
        Self(Rc::new(RefCell::new(value)))
    }

    pub fn with_mut<R>(&mut self, f: impl FnOnce(&mut T) -> R) -> R {
        let mut this = self.0.as_ref().borrow_mut();
        f(this.deref_mut())
    }

    pub fn borrow(&self) -> impl Deref<Target = T> + '_ {
        self.0.borrow()
    }

    pub fn borrow_mut(&mut self) -> impl DerefMut<Target = T> + '_ {
        self.0.borrow_mut()
    }
}

impl<T> Clone for Mrc<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T> PartialEq for Mrc<T> {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
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
