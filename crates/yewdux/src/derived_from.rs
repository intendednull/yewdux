use std::rc::Rc;

use crate::Context;

pub trait DerivedFrom<Store: crate::Store>: crate::Store + 'static {
    fn on_change(&self, state: Rc<Store>) -> Self;
}

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

pub trait DerivedFromMut<Store: crate::Store>: crate::Store + Clone + 'static {
    fn on_change(&mut self, state: Rc<Store>);
}

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
