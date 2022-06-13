//! Mutable reference counted wrapper type that works well with Yewdux.
//!
//! Useful when you don't want to implement `Clone` or `PartialEq` for a type.
//!
//! ```ignore
//! use yew::prelude::*;
//! use yewdux::{prelude::*, mrc::Mrc};
//!
//! // Notice we don't implement Clone or PartialEq.
//! #[derive(Default)]
//! struct MyLargeData(u32);
//!
//! #[derive(Default, Clone, PartialEq, Store)]
//! struct State {
//!     // Your expensive-clone field here.
//!     data: Mrc<MyLargeData>,
//! }
//! ```
//!
//! Mutating is done as expected:
//!
//! ```ignore
//! let onclick = dispatch.reduce_callback(|state| {
//!     let mut data = state.data.borrow_mut();
//!
//!     data.0 += 1;
//! });
//! ```
//!
use std::{
    cell::{Cell, RefCell},
    ops::{Deref, DerefMut},
    rc::Rc,
};

use crate::store::Store;

fn nonce() -> u32 {
    thread_local! {
        static NONCE: Cell<u32> = Default::default();
    }

    NONCE
        .try_with(|nonce| {
            nonce.set(nonce.get().wrapping_add(1));
            nonce.get()
        })
        .expect("NONCE thread local key init failed")
}

/// Mutable reference counted wrapper type that works well with Yewdux.
///
/// This is basically a wrapper over `Rc<RefCell<T>>`, with the notable difference of simple change
/// detection (so it works with Yewdux). Whenever this type borrows mutably, it is marked as
/// changed. Because there is no way to detect whether it has actually changed or not, it is up to
/// the user to prevent unecessary re-renders.
#[derive(Debug)]
pub struct Mrc<T> {
    inner: Rc<RefCell<T>>,
    nonce: Cell<u32>,
}

impl<T> Mrc<T> {
    pub fn new(value: T) -> Self {
        Self {
            inner: Rc::new(RefCell::new(value)),
            nonce: Cell::new(nonce()),
        }
    }

    pub fn with_mut<R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
        let mut this = self.borrow_mut();
        f(this.deref_mut())
    }

    pub fn borrow(&self) -> impl Deref<Target = T> + '_ {
        self.inner.borrow()
    }

    /// Provide a mutable reference to inner value.
    pub fn borrow_mut(&self) -> impl DerefMut<Target = T> + '_ {
        // Mark as changed.
        self.nonce.set(nonce());
        self.inner.borrow_mut()
    }
}

impl<T: Store> Store for Mrc<T> {
    fn new() -> Self {
        T::new().into()
    }

    fn changed(&self, other: &Self) -> bool {
        self != other
    }
}

impl<T: Default> Default for Mrc<T> {
    fn default() -> Self {
        Self::new(Default::default())
    }
}

impl<T> Clone for Mrc<T> {
    fn clone(&self) -> Self {
        Self {
            inner: Rc::clone(&self.inner),
            nonce: self.nonce.clone(),
        }
    }
}

impl<T> From<T> for Mrc<T> {
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

impl<T> PartialEq for Mrc<T> {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.inner, &other.inner) && self.nonce == other.nonce
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

        fn changed(&self, other: &Self) -> bool {
            self != other
        }
    }

    struct CanImplStoreForMrcDirectly;
    impl Store for Mrc<CanImplStoreForMrcDirectly> {
        fn new() -> Self {
            CanImplStoreForMrcDirectly.into()
        }

        fn changed(&self, other: &Self) -> bool {
            self != other
        }
    }

    #[test]
    fn subscriber_is_notified_on_borrow_mut() {
        let flag = Mrc::new(false);

        let dispatch = {
            let flag = flag.clone();
            Dispatch::<TestState>::subscribe(move |_| flag.clone().with_mut(|flag| *flag = true))
        };

        *flag.borrow_mut() = false;

        dispatch.reduce_mut(|state| {
            state.0.borrow_mut();
        });

        assert!(*flag.borrow());
    }

    #[test]
    fn subscriber_is_notified_on_with_mut() {
        let flag = Mrc::new(false);

        let dispatch = {
            let flag = flag.clone();
            Dispatch::<TestState>::subscribe(move |_| flag.clone().with_mut(|flag| *flag = true))
        };

        *flag.borrow_mut() = false;

        dispatch.reduce_mut(|state| state.0.with_mut(|_| ()));

        assert!(*flag.borrow());
    }

    #[test]
    fn can_wrap_store_with_mrc() {
        let dispatch = Dispatch::<Mrc<TestState>>::new();
        assert!(*dispatch.get().borrow().0.borrow() == 0)
    }
}
