use std::rc::Rc;

use crate::{
    anymap::AnyMap,
    mrc::Mrc,
    store::{Reducer, Store},
    subscriber::{Callable, SubscriberId, Subscribers},
};

pub(crate) struct Entry<S> {
    pub(crate) store: Mrc<Rc<S>>,
}

impl<S> Clone for Entry<S> {
    fn clone(&self) -> Self {
        Self {
            store: Mrc::clone(&self.store),
        }
    }
}

impl<S: Store> Entry<S> {
    /// Apply a function to state, returning if it should notify subscribers or not.
    pub(crate) fn reduce<R: Reducer<S>>(&self, reducer: R) -> bool {
        let old = Rc::clone(&self.store.borrow());
        // Apply the reducer.
        let new = reducer.apply(Rc::clone(&old));
        // Update to new state.
        *self.store.borrow_mut() = new;
        // Return whether or not subscribers should be notified.
        self.store.borrow().should_notify(&old)
    }
}

/// Execution context for a dispatch
///
/// # Example
///
/// ```
/// use yewdux::prelude::*;
///
/// #[derive(Clone, PartialEq, Default, Store)]
/// struct Counter(usize);
///
/// // In a real application, you'd typically get the context from a parent component
/// let cx = yewdux::Context::new();
/// let dispatch = Dispatch::<Counter>::new(&cx);
/// ```
#[derive(Clone, Default, PartialEq)]
pub struct Context {
    inner: Mrc<AnyMap>,
}

impl Context {
    pub fn new() -> Self {
        Default::default()
    }

    #[cfg(any(doc, feature = "doctests", target_arch = "wasm32"))]
    pub fn global() -> Self {
        thread_local! {
            static CONTEXT: Context = Default::default();
        }

        CONTEXT
            .try_with(|cx| cx.clone())
            .expect("CONTEXTS thread local key init failed")
    }

    /// Initialize a store using a custom constructor. `Store::new` will not be called in this
    /// case. If already initialized, the custom constructor will not be called.
    pub fn init<S: Store, F: FnOnce(&Self) -> S>(&self, new_store: F) {
        self.get_or_init(new_store);
    }

    /// Get or initialize a store using a custom constructor. `Store::new` will not be called in
    /// this case. If already initialized, the custom constructor will not be called.
    pub(crate) fn get_or_init<S: Store, F: FnOnce(&Self) -> S>(&self, new_store: F) -> Entry<S> {
        // Get context, or None if it doesn't exist.
        //
        // We use an option here because a new Store should not be created during this borrow. We
        // want to allow this store access to other stores during creation, so cannot be borrowing
        // the global resource while initializing. Instead we create a temporary placeholder, which
        // indicates the store needs to be created. Without this indicator we would have needed to
        // check if the map contains the entry beforehand, which would have meant two map lookups
        // per call instead of just one.
        let maybe_entry = self.inner.with_mut(|x| {
            x.entry::<Mrc<Option<Entry<S>>>>()
                .or_insert_with(|| None.into())
                .clone()
        });

        // If it doesn't exist, create and save the new store.
        let exists = maybe_entry.borrow().is_some();
        if !exists {
            // Init store outside of borrow. This allows the store to access other stores when it
            // is being created.
            let entry = Entry {
                store: Mrc::new(Rc::new(new_store(self))),
            };

            *maybe_entry.borrow_mut() = Some(entry);
        }

        // Now we get the context, which must be initialized because we already checked above.
        let entry = maybe_entry
            .borrow()
            .clone()
            .expect("Context not initialized");

        entry
    }

    /// Get or initialize a store with a default Store::new implementation.
    pub(crate) fn get_or_init_default<S: Store>(&self) -> Entry<S> {
        self.get_or_init(S::new)
    }

    pub fn reduce<S: Store, R: Reducer<S>>(&self, r: R) {
        let entry = self.get_or_init_default::<S>();
        let should_notify = entry.reduce(r);

        if should_notify {
            let state = Rc::clone(&entry.store.borrow());
            self.notify_subscribers(state)
        }
    }

    pub fn reduce_mut<S: Store + Clone, F: FnOnce(&mut S)>(&self, f: F) {
        self.reduce(|mut state| {
            f(Rc::make_mut(&mut state));
            state
        });
    }

    /// Set state to given value.
    pub fn set<S: Store>(&self, value: S) {
        self.reduce(move |_| value.into());
    }

    /// Get current state.
    pub fn get<S: Store>(&self) -> Rc<S> {
        Rc::clone(&self.get_or_init_default::<S>().store.borrow())
    }

    /// Send state to all subscribers.
    pub fn notify_subscribers<S: Store>(&self, state: Rc<S>) {
        let entry = self.get_or_init_default::<Mrc<Subscribers<S>>>();
        entry.store.borrow().notify(state);
    }

    /// Subscribe to a store. `on_change` is called immediately, then every  time state changes.
    pub fn subscribe<S: Store, N: Callable<S>>(&self, on_change: N) -> SubscriberId<S> {
        // Notify subscriber with inital state.
        on_change.call(self.get::<S>());

        self.get_or_init_default::<Mrc<Subscribers<S>>>()
            .store
            .borrow()
            .subscribe(on_change)
    }

    /// Similar to [Self::subscribe], however state is not called immediately.
    pub fn subscribe_silent<S: Store, N: Callable<S>>(&self, on_change: N) -> SubscriberId<S> {
        self.get_or_init_default::<Mrc<Subscribers<S>>>()
            .store
            .borrow()
            .subscribe(on_change)
    }

    /// Initialize a listener
    pub fn init_listener<L: crate::Listener, F: FnOnce() -> L>(&self, new_listener: F) {
        crate::init_listener(new_listener, self);
    }

    pub fn derived_from<Store, Derived>(&self)
    where
        Store: crate::Store,
        Derived: crate::derived_from::DerivedFrom<Store>,
    {
        crate::derived_from::derive_from::<Store, Derived>(self);
    }

    pub fn derived_from_mut<Store, Derived>(&self)
    where
        Store: crate::Store,
        Derived: crate::derived_from::DerivedFromMut<Store>,
    {
        crate::derived_from::derive_from_mut::<Store, Derived>(self);
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use super::*;

    #[derive(Clone, PartialEq, Eq)]
    struct TestState(u32);
    impl Store for TestState {
        fn new(_cx: &Context) -> Self {
            Self(0)
        }

        fn should_notify(&self, other: &Self) -> bool {
            self != other
        }
    }

    #[derive(Clone, PartialEq, Eq)]
    struct TestState2(u32);
    impl Store for TestState2 {
        fn new(cx: &Context) -> Self {
            cx.get_or_init_default::<TestState>();
            Self(0)
        }

        fn should_notify(&self, other: &Self) -> bool {
            self != other
        }
    }

    #[test]
    fn can_access_other_store_for_new_of_current_store() {
        let _context = Context::new().get_or_init_default::<TestState2>();
    }

    #[derive(Clone, PartialEq, Eq)]
    struct StoreNewIsOnlyCalledOnce(Rc<Cell<u32>>);
    impl Store for StoreNewIsOnlyCalledOnce {
        fn new(_cx: &Context) -> Self {
            thread_local! {
                /// Stores all shared state.
                static COUNT: Rc<Cell<u32>> = Default::default();
            }

            let count = COUNT.try_with(|x| x.clone()).unwrap();

            count.set(count.get() + 1);

            Self(count)
        }

        fn should_notify(&self, other: &Self) -> bool {
            self != other
        }
    }

    #[test]
    fn store_new_is_only_called_once() {
        let cx = Context::new();
        cx.get_or_init_default::<StoreNewIsOnlyCalledOnce>();
        let entry = cx.get_or_init_default::<StoreNewIsOnlyCalledOnce>();

        assert!(entry.store.borrow().0.get() == 1)
    }

    #[test]
    fn recursive_reduce() {
        let cx = Context::new();
        let cx2 = cx.clone();
        cx.reduce::<TestState, _>(|_s: Rc<TestState>| {
            cx2.reduce::<TestState, _>(|s: Rc<TestState>| TestState(s.0 + 1).into());
            TestState(cx2.get::<TestState>().0 + 1).into()
        });

        assert_eq!(cx.get::<TestState>().0, 2);
    }
}
