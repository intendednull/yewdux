//! Ergonomic interface with shared state.
use std::rc::Rc;

use yew::{Callback, Properties};

use crate::store::{Reduction, ReductionOnce, Store, StoreLink};

type Model<T> = <T as Store>::Model;

pub trait DispatcherProp {
    type Store: Store;

    fn dispatcher(&mut self) -> &mut Dispatcher<Self::Store>;
}

/// Interface to shared state
#[derive(Properties)]
pub struct Dispatcher<STORE>
where
    STORE: Store,
{
    #[prop_or_default]
    pub(crate) state: Option<Rc<Model<STORE>>>,
    #[prop_or_default]
    pub(crate) callback: Callback<Reduction<Model<STORE>>>,
    #[prop_or_default]
    pub(crate) callback_once: Callback<ReductionOnce<Model<STORE>>>,
    #[prop_or_default]
    pub(crate) link: Option<StoreLink<STORE>>,
}

impl<STORE> Dispatcher<STORE>
where
    STORE: Store,
{
    pub fn link(&self) -> &StoreLink<STORE> {
        self.link.as_ref().expect(
            "Link accessed prematurely. Is your component wrapped in a SharedStateComponent?",
        )
    }

    pub fn state(&self) -> &Model<STORE> {
        self.state.as_ref().expect(
            "State accessed prematurely. Is your component wrapped in a SharedStateComponent?",
        )
    }

    /// Apply a function that may mutate shared state.
    /// Changes are not immediate, and must be handled in `Component::change`.
    pub fn reduce(&self, f: impl FnOnce(&mut Model<STORE>) + 'static) {
        self.callback_once.emit(Box::new(f))
    }

    /// Convenience method for modifying shared state directly from a `Callback`.
    /// The callback event is ignored here, see `reduce_callback_with` for the alternative.
    pub fn reduce_callback<E: 'static>(
        &self,
        f: impl Fn(&mut Model<STORE>) + 'static,
    ) -> Callback<E>
    where
        Model<STORE>: 'static,
    {
        let f = Rc::new(f);
        self.callback.reform(move |_| f.clone())
    }

    /// Convenience method for modifying shared state directly from a `CallbackOnce`.
    /// The callback event is ignored here, see `reduce_callback_once_with` for the alternative.
    pub fn reduce_callback_once<E: 'static>(
        &self,
        f: impl FnOnce(&mut Model<STORE>) + 'static,
    ) -> Callback<E>
    where
        Model<STORE>: 'static,
    {
        let f = Box::new(f);
        let cb = self.callback_once.clone();
        Callback::once(move |_| cb.emit(f))
    }

    /// Convenience method for modifying shared state directly from a `Callback`.
    /// Similar to `reduce_callback` but it also accepts the fired event.
    pub fn reduce_callback_with<E: 'static>(
        &self,
        f: impl Fn(&mut Model<STORE>, E) + 'static,
    ) -> Callback<E>
    where
        Model<STORE>: 'static,
        E: Clone,
    {
        let f = Rc::new(f);
        self.callback.reform(move |e: E| {
            let f = f.clone();
            Rc::new(move |state| f.clone()(state, e.clone()))
        })
    }

    /// Convenience method for modifying shared state directly from a `CallbackOnce`.
    /// Similar to `reduce_callback` but it also accepts the fired event.
    pub fn reduce_callback_once_with<E: 'static>(
        &self,
        f: impl FnOnce(&mut Model<STORE>, E) + 'static,
    ) -> Callback<E>
    where
        Model<STORE>: 'static,
    {
        let cb = self.callback_once.clone();
        Callback::once(move |e| cb.emit(Box::new(move |state| f(state, e))))
    }
}

impl<STORE> DispatcherProp for Dispatcher<STORE>
where
    STORE: Store,
{
    type Store = STORE;

    fn dispatcher(&mut self) -> &mut Dispatcher<Self::Store> {
        self
    }
}

impl<STORE> Default for Dispatcher<STORE>
where
    STORE: Store,
{
    fn default() -> Self {
        Self {
            state: Default::default(),
            callback: Default::default(),
            callback_once: Default::default(),
            link: Default::default(),
        }
    }
}

impl<STORE> Clone for Dispatcher<STORE>
where
    STORE: Store,
    StoreLink<STORE>: Clone,
{
    fn clone(&self) -> Self {
        Self {
            state: self.state.clone(),
            callback: self.callback.clone(),
            callback_once: self.callback_once.clone(),
            link: self.link.clone(),
        }
    }
}

impl<STORE> PartialEq for Dispatcher<STORE>
where
    STORE: Store,
{
    fn eq(&self, other: &Self) -> bool {
        self.state
            .as_ref()
            .zip(other.state.as_ref())
            .map(|(a, b)| Rc::ptr_eq(a, b))
            .unwrap_or(false)
            && self.callback == other.callback
            && self.callback_once == other.callback_once
    }
}
