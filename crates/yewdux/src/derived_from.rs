//! Provides functionality for creating derived stores that automatically update
//! based on changes to other stores.
//!
//! This module enables the creation of stores that are computed from other stores,
//! allowing for automatic synchronization when the source stores change.
//!
//! There are two approaches available:
//! - `DerivedFrom`: For immutable transformations where a new derived store is created on each update
//! - `DerivedFromMut`: For mutable transformations where the derived store is updated in-place

use std::rc::Rc;

use crate::Context;

/// Trait for creating a derived store that transforms from another store immutably.
///
/// Implementors of this trait represent a store that derives its state from another store.
/// When the source store changes, `on_change` is called to create a new instance of the derived store.
///
/// # Type Parameters
///
/// * `Store`: The source store type this store derives from
pub trait DerivedFrom<Store: crate::Store>: crate::Store + 'static {
    /// Creates a new instance of the derived store based on the current state of the source store.
    ///
    /// # Parameters
    ///
    /// * `state`: The current state of the source store
    ///
    /// # Returns
    ///
    /// A new instance of the derived store
    fn on_change(&self, state: Rc<Store>) -> Self;
}

/// Internal listener that updates the derived store when the source store changes.
///
/// This struct implements the `Listener` trait for the source store and manages
/// updating the derived store through its `Dispatch`.
struct Listener<Store, Derived>
where
    Store: crate::Store,
    Derived: DerivedFrom<Store>,
{
    derived: crate::Dispatch<Derived>,
    _marker: std::marker::PhantomData<Store>,
}

impl<Store, Derived> crate::Listener for Listener<Store, Derived>
where
    Store: crate::Store,
    Derived: DerivedFrom<Store>,
{
    type Store = Store;

    fn on_change(&self, _cx: &Context, state: Rc<Self::Store>) {
        self.derived
            .reduce(|derived| derived.on_change(Rc::clone(&state)).into());
    }
}

/// Initializes a derived store that automatically updates when the source store changes.
///
/// This function sets up a listener on the source store that will update the derived store
/// whenever the source store changes, using the `DerivedFrom` implementation to transform the state.
///
/// # Type Parameters
///
/// * `Store`: The source store type to derive from
/// * `Derived`: The derived store type that implements `DerivedFrom<Store>`
///
/// # Parameters
///
/// * `cx`: The Yewdux context
///
/// # Example
///
/// ```rust
/// use std::rc::Rc;
/// use yewdux::{Context, Store, Dispatch};
/// use yewdux::derived_from::{DerivedFrom, derive_from};
///
/// #[derive(Clone, PartialEq)]
/// struct SourceStore { value: i32 }
/// impl Store for SourceStore {
///     fn new(_: &Context) -> Self { Self { value: 0 } }
///     fn should_notify(&self, old: &Self) -> bool { self != old }
/// }
///
/// #[derive(Clone, PartialEq)]
/// struct DerivedStore { doubled_value: i32 }
/// impl Store for DerivedStore {
///     fn new(_: &Context) -> Self { Self { doubled_value: 0 } }
///     fn should_notify(&self, old: &Self) -> bool { self != old }
/// }
///
/// impl DerivedFrom<SourceStore> for DerivedStore {
///     fn on_change(&self, source: Rc<SourceStore>) -> Self {
///         Self { doubled_value: source.value * 2 }
///     }
/// }
///
/// // Create a context - in a real application, you'd typically get this from a parent component
/// let cx = Context::new();
/// 
/// // Set up the derived relationship
/// derive_from::<SourceStore, DerivedStore>(&cx);
///
/// // Get dispatches for both stores
/// let source_dispatch = Dispatch::<SourceStore>::new(&cx);
/// let derived_dispatch = Dispatch::<DerivedStore>::new(&cx);
///
/// source_dispatch.reduce_mut(|state| state.value = 5);
/// assert_eq!(derived_dispatch.get().doubled_value, 10);
/// ```
pub fn derive_from<Store, Derived>(cx: &Context)
where
    Store: crate::Store,
    Derived: DerivedFrom<Store>,
{
    crate::init_listener(
        || Listener {
            derived: crate::Dispatch::<Derived>::new(cx),
            _marker: std::marker::PhantomData,
        },
        cx,
    );
}

/// Trait for creating a derived store that is mutably updated from another store.
///
/// Implementors of this trait represent a store that derives its state from another store.
/// When the source store changes, `on_change` is called to mutably update the derived store.
///
/// # Type Parameters
///
/// * `Store`: The source store type this store derives from
pub trait DerivedFromMut<Store: crate::Store>: crate::Store + Clone + 'static {
    /// Updates the derived store based on the current state of the source store.
    ///
    /// # Parameters
    ///
    /// * `state`: The current state of the source store
    fn on_change(&mut self, state: Rc<Store>);
}

/// Internal listener that mutably updates the derived store when the source store changes.
///
/// This struct implements the `Listener` trait for the source store and manages
/// updating the derived store through its `Dispatch` using mutable references.
struct ListenerMut<Store, Derived>
where
    Store: crate::Store,
    Derived: DerivedFromMut<Store>,
{
    derived: crate::Dispatch<Derived>,
    _marker: std::marker::PhantomData<Store>,
}

impl<Store, Derived> crate::Listener for ListenerMut<Store, Derived>
where
    Store: crate::Store,
    Derived: DerivedFromMut<Store>,
{
    type Store = Store;

    fn on_change(&self, _cx: &Context, state: Rc<Self::Store>) {
        self.derived
            .reduce_mut(|derived| derived.on_change(Rc::clone(&state)));
    }
}

/// Initializes a derived store that is mutably updated when the source store changes.
///
/// This function sets up a listener on the source store that will update the derived store
/// whenever the source store changes, using the `DerivedFromMut` implementation to transform the state.
///
/// # Type Parameters
///
/// * `Store`: The source store type to derive from
/// * `Derived`: The derived store type that implements `DerivedFromMut<Store>`
///
/// # Parameters
///
/// * `cx`: The Yewdux context
///
/// # Example
///
/// ```rust
/// use std::rc::Rc;
/// use yewdux::{Context, Store, Dispatch};
/// use yewdux::derived_from::{DerivedFromMut, derive_from_mut};
///
/// #[derive(Clone, PartialEq)]
/// struct SourceStore { value: i32 }
/// impl Store for SourceStore {
///     fn new(_: &Context) -> Self { Self { value: 0 } }
///     fn should_notify(&self, old: &Self) -> bool { self != old }
/// }
///
/// #[derive(Clone, PartialEq)]
/// struct DerivedStore { doubled_value: i32 }
/// impl Store for DerivedStore {
///     fn new(_: &Context) -> Self { Self { doubled_value: 0 } }
///     fn should_notify(&self, old: &Self) -> bool { self != old }
/// }
///
/// impl DerivedFromMut<SourceStore> for DerivedStore {
///     fn on_change(&mut self, source: Rc<SourceStore>) {
///         self.doubled_value = source.value * 2;
///     }
/// }
///
/// // Create a context - in a real application, you'd typically get this from a parent component
/// let cx = Context::new();
/// 
/// // Set up the derived relationship with mutable updates
/// derive_from_mut::<SourceStore, DerivedStore>(&cx);
///
/// // Get dispatches for both stores
/// let source_dispatch = Dispatch::<SourceStore>::new(&cx);
/// let derived_dispatch = Dispatch::<DerivedStore>::new(&cx);
///
/// source_dispatch.reduce_mut(|state| state.value = 5);
/// assert_eq!(derived_dispatch.get().doubled_value, 10);
/// ```
pub fn derive_from_mut<Store, Derived>(cx: &Context)
where
    Store: crate::Store,
    Derived: DerivedFromMut<Store>,
{
    crate::init_listener(
        || ListenerMut {
            derived: crate::Dispatch::<Derived>::new(cx),
            _marker: std::marker::PhantomData,
        },
        cx,
    );
}

#[cfg(test)]
mod tests {
    use crate::Dispatch;

    use super::*;

    #[test]
    fn can_derive_from() {
        #[derive(Clone, PartialEq, Eq)]
        struct TestState(u32);
        impl crate::Store for TestState {
            fn new(_cx: &crate::Context) -> Self {
                Self(0)
            }

            fn should_notify(&self, other: &Self) -> bool {
                self != other
            }
        }

        #[derive(Clone, PartialEq, Eq)]
        struct TestDerived(u32);
        impl crate::Store for TestDerived {
            fn new(_cx: &crate::Context) -> Self {
                Self(0)
            }

            fn should_notify(&self, other: &Self) -> bool {
                self != other
            }
        }

        impl DerivedFrom<TestState> for TestDerived {
            fn on_change(&self, state: Rc<TestState>) -> Self {
                Self(state.0)
            }
        }

        let cx = crate::Context::new();
        derive_from::<TestState, TestDerived>(&cx);

        let dispatch_derived = Dispatch::<TestDerived>::new(&cx);
        let dispatch_state = Dispatch::<TestState>::new(&cx);

        dispatch_state.reduce_mut(|state| state.0 += 1);
        assert_eq!(dispatch_derived.get().0, 1);
    }

    #[test]
    fn can_derive_from_mut() {
        #[derive(Clone, PartialEq, Eq)]
        struct TestState(u32);
        impl crate::Store for TestState {
            fn new(_cx: &crate::Context) -> Self {
                Self(0)
            }

            fn should_notify(&self, other: &Self) -> bool {
                self != other
            }
        }

        #[derive(Clone, PartialEq, Eq)]
        struct TestDerived(u32);
        impl crate::Store for TestDerived {
            fn new(_cx: &crate::Context) -> Self {
                Self(0)
            }

            fn should_notify(&self, other: &Self) -> bool {
                self != other
            }
        }

        impl DerivedFromMut<TestState> for TestDerived {
            fn on_change(&mut self, state: Rc<TestState>) {
                self.0 = state.0;
            }
        }

        let cx = crate::Context::new();
        derive_from_mut::<TestState, TestDerived>(&cx);

        let dispatch_derived = Dispatch::<TestDerived>::new(&cx);
        let dispatch_state = Dispatch::<TestState>::new(&cx);

        dispatch_state.reduce_mut(|state| state.0 += 1);
        assert_eq!(dispatch_derived.get().0, 1);
    }
}
