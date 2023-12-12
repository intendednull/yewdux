//! Mutable reference counted wrapper type that works well with Yewdux.
//!
//! Useful when you don't want to implement `Clone` or `PartialEq` for a type.
//!
//! ```
//! # use yewdux::mrc::Mrc;
//! # fn main() {
//! let expensive_data = Mrc::new("Some long string that shouldn't be cloned.".to_string());
//! let old_ref = expensive_data.clone();
//!
//! // They are equal (for now).
//! assert!(expensive_data == old_ref);
//!
//! // Here we use interior mutability to change the inner value. Doing so will mark the
//! // container as changed.
//! *expensive_data.borrow_mut() += " Here we can modify our expensive data.";
//!
//! // Once marked as changed, it will cause any equality check to fail (forcing a re-render).
//! assert!(expensive_data != old_ref);
//! // The underlying state is the same though.
//! assert!(*expensive_data.borrow() == *old_ref.borrow());
//! # }
//! ```

use std::{
    cell::{Cell, RefCell},
    ops::{Deref, DerefMut},
    rc::Rc,
};

use serde::{Deserialize, Serialize};

use crate::{store::Store, Context};

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
#[derive(Debug, Serialize, Deserialize)]
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
    fn new(cx: &Context) -> Self {
        T::new(cx).into()
    }

    fn should_notify(&self, other: &Self) -> bool {
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

    use crate::{dispatch::Dispatch, store::Store, Context};

    use super::*;

    #[derive(Clone, PartialEq)]
    struct TestState(Mrc<u32>);
    impl Store for TestState {
        fn new(_cx: &Context) -> Self {
            Self(Mrc::new(0))
        }

        fn should_notify(&self, other: &Self) -> bool {
            self != other
        }
    }

    struct CanImplStoreForMrcDirectly;
    impl Store for Mrc<CanImplStoreForMrcDirectly> {
        fn new(_cx: &Context) -> Self {
            CanImplStoreForMrcDirectly.into()
        }

        fn should_notify(&self, other: &Self) -> bool {
            self != other
        }
    }

    #[test]
    fn subscriber_is_notified_on_borrow_mut() {
        let flag = Mrc::new(false);
        let cx = Context::new();

        let dispatch = {
            let flag = flag.clone();
            Dispatch::<TestState>::new(&cx)
                .subscribe(move |_| flag.clone().with_mut(|flag| *flag = true))
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
        let cx = Context::new();

        let dispatch = {
            let flag = flag.clone();
            Dispatch::<TestState>::new(&cx)
                .subscribe(move |_| flag.clone().with_mut(|flag| *flag = true))
        };

        *flag.borrow_mut() = false;

        dispatch.reduce_mut(|state| state.0.with_mut(|_| ()));

        assert!(*flag.borrow());
    }

    #[test]
    fn can_wrap_store_with_mrc() {
        let cx = Context::new();
        let dispatch = Dispatch::<Mrc<TestState>>::new(&cx);
        assert!(*dispatch.get().borrow().0.borrow() == 0)
    }
}
