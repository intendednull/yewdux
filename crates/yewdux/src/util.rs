use std::{
    cell::{Cell, RefCell},
    ops::{Deref, DerefMut},
    rc::Rc,
};

use yew::Callback;

fn nonce() -> u32 {
    thread_local! {
        static NONCE: Cell<u32> = Default::default();
    }

    NONCE
        .try_with(|nonce| {
            nonce.set(nonce.get().wrapping_add(1));
            nonce.get()
        })
        .expect("Thread local key init failed")
}

/// A cloneable wrapper type that provides interior mutability.
#[derive(Debug, Default)]
pub struct Mrc<T> {
    inner: Rc<RefCell<T>>,
    nonce: u32,
}

impl<T: 'static> Mrc<T> {
    pub fn new(value: T) -> Self {
        Self {
            inner: Rc::new(RefCell::new(value)),
            nonce: nonce(),
        }
    }

    pub fn with_mut<R>(&mut self, f: impl FnOnce(&mut T) -> R) -> R {
        let mut this = self.borrow_mut();
        f(this.deref_mut())
    }

    pub fn borrow(&self) -> impl Deref<Target = T> + '_ {
        self.inner.borrow()
    }

    /// Provide a mutable reference to inner value.
    pub fn borrow_mut(&mut self) -> impl DerefMut<Target = T> + '_ {
        // Mark as changed.
        self.nonce = nonce();
        self.inner.borrow_mut()
    }
}

impl<T> Clone for Mrc<T> {
    fn clone(&self) -> Self {
        Self {
            inner: Rc::clone(&self.inner),
            nonce: self.nonce,
        }
    }
}

impl<T> PartialEq for Mrc<T> {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.inner, &other.inner) && self.nonce == other.nonce
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

#[cfg(test)]
mod tests {

    use crate::{dispatch::Dispatch, store::Store};

    use super::*;

    #[derive(Clone, PartialEq)]
    struct TestState(Mrc<u32>);
    impl Store for TestState {
        fn new() -> Self {
            Self(Mrc::new(0))
        }
    }

    #[test]
    fn subscriber_is_notified_with_mrc() {
        let mut flag = Mrc::new(false);

        let dispatch = {
            let flag = flag.clone();
            Dispatch::<TestState>::subscribe(move |_| flag.clone().with_mut(|flag| *flag = true))
        };

        *flag.borrow_mut() = false;

        dispatch.reduce(|state| *state.0.borrow_mut() += 1);

        assert!(*flag.borrow());
    }
}
