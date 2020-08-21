//! Ergonomic interface with shared state.
use std::rc::Rc;

use yew::{Callback, Properties};

use super::handler::{GlobalHandler, Handler, Reduction, ReductionOnce, StorageHandler};

type Model<T> = <T as Handler>::Model;

/// Provides mutable access for wrapper component to update
pub trait Handle {
    type Handler: Handler;

    fn set_local_state(&mut self, state: Rc<Model<Self::Handler>>);
    fn set_local_callback(
        &mut self,
        callback: Callback<Reduction<Model<Self::Handler>>>,
        callback_once: Callback<ReductionOnce<Model<Self::Handler>>>,
    );
    fn set_local(&mut self, other: &Self);
}

/// Allows any `Properties` to have shared state.
pub trait SharedState {
    type Handle: Handle;
    fn handle(&mut self) -> &mut Self::Handle;
}

/// A handle for io with state handlers
#[derive(Default, Properties)]
pub struct StateHandle<T, H>
where
    T: Default + Clone + 'static,
    H: Handler,
{
    #[prop_or_default]
    state: Rc<T>,
    #[prop_or_default]
    callback: Callback<Reduction<T>>,
    #[prop_or_default]
    callback_once: Callback<ReductionOnce<T>>,
    #[prop_or_default]
    _mark: std::marker::PhantomData<H>,
}

impl<T, H> StateHandle<T, H>
where
    T: Default + Clone + 'static,
    H: Handler<Model = T>,
{
    pub fn state(&self) -> &T {
        &self.state
    }

    /// Apply a function that may mutate shared state.
    /// Changes are not immediate, and must be handled in `Component::change`.
    pub fn reduce(&self, f: impl FnOnce(&mut T) + 'static) {
        self.callback_once.emit(Box::new(f))
    }

    /// Convenience method for modifying shared state directly from a `Callback`.
    /// The callback event is ignored here, see `reduce_callback_with` for the alternative.
    pub fn reduce_callback<E: 'static>(&self, f: impl Fn(&mut T) + 'static) -> Callback<E>
    where
        T: 'static,
    {
        let f = Rc::new(f);
        self.callback.reform(move |_| f.clone())
    }

    /// Convenience method for modifying shared state directly from a `CallbackOnce`.
    /// The callback event is ignored here, see `reduce_callback_once_with` for the alternative.
    pub fn reduce_callback_once<E: 'static>(&self, f: impl FnOnce(&mut T) + 'static) -> Callback<E>
    where
        T: 'static,
    {
        let f = Box::new(f);
        let cb = self.callback_once.clone();
        Callback::once(move |_| cb.emit(f))
    }

    /// Convenience method for modifying shared state directly from a `Callback`.
    /// Similar to `reduce_callback` but it also accepts the fired event.
    pub fn reduce_callback_with<E: 'static>(&self, f: impl Fn(&mut T, E) + 'static) -> Callback<E>
    where
        T: 'static,
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
        f: impl FnOnce(&mut T, E) + 'static,
    ) -> Callback<E>
    where
        T: 'static,
    {
        let cb = self.callback_once.clone();
        Callback::once(move |e| cb.emit(Box::new(move |state| f(state, e))))
    }
}

impl<T, H> Clone for StateHandle<T, H>
where
    T: Default + Clone + 'static,
    H: Handler,
{
    fn clone(&self) -> Self {
        Self {
            state: self.state.clone(),
            callback: self.callback.clone(),
            callback_once: self.callback_once.clone(),
            _mark: Default::default(),
        }
    }
}

impl<T, H> PartialEq for StateHandle<T, H>
where
    T: Default + PartialEq + Clone + 'static,
    H: Handler,
{
    fn eq(&self, other: &Self) -> bool {
        self.state == other.state && self.callback == other.callback
    }
}

impl<T, H> Handle for StateHandle<T, H>
where
    T: Default + Clone,
    H: Handler<Model = T>,
{
    type Handler = H;

    fn set_local_state(&mut self, state: Rc<Model<Self::Handler>>) {
        self.state = state;
    }

    fn set_local_callback(
        &mut self,
        callback: Callback<Reduction<Model<Self::Handler>>>,
        callback_once: Callback<ReductionOnce<Model<Self::Handler>>>,
    ) {
        self.callback = callback;
        self.callback_once = callback_once;
    }

    fn set_local(&mut self, other: &Self) {
        *self = other.clone();
    }
}

impl<T, H> SharedState for StateHandle<T, H>
where
    T: Default + Clone + 'static,
    H: Handler<Model = T>,
{
    type Handle = Self;

    fn handle(&mut self) -> &mut Self::Handle {
        self
    }
}

/// Handle for basic shared state.
pub type GlobalHandle<T> = StateHandle<T, GlobalHandler<T>>;
/// Handle for shared state with persistent storage.
pub type StorageHandle<T> = StateHandle<T, StorageHandler<T>>;
