//! Ergonomic interface with shared state.
use std::rc::Rc;

use yew::{Callback, Properties};

use super::handler::{GlobalHandler, Handler, Reduction, StorageHandler};

type Model<T> = <T as Handler>::Model;

/// Allows any `Properties` to have shared state.
pub trait SharedState {
    type Handle: Handle;
    fn handle(&mut self) -> &mut Self::Handle;
}

/// Primary shared state interface
pub trait Handle {
    type Handler: Handler;

    /// Current state.
    fn state(&self) -> &Model<Self::Handler>;
    /// Callback to use for updating state.
    fn callback(&self) -> &Callback<Reduction<Model<Self::Handler>>>;
    #[doc(hidden)]
    fn __set_local(
        &mut self,
        state: &Rc<Model<Self::Handler>>,
        callback: &Callback<Reduction<Model<Self::Handler>>>,
    );

    /// Apply a function that may mutate shared state.
    /// Changes are not immediate, and must be handled in `Component::change`.
    fn reduce(&self, f: impl FnOnce(&mut Model<Self::Handler>) + 'static) {
        self.callback().emit(Box::new(f))
    }

    /// Convenience method for modifying shared state directly from a callback.
    /// The callback event is ignored here, see `reduce_callback_with` for the alternative.
    fn reduce_callback<T: 'static>(
        &self,
        f: impl FnOnce(&mut Model<Self::Handler>) + Copy + 'static,
    ) -> Callback<T>
    where
        Model<Self::Handler>: 'static,
    {
        self.callback()
            .reform(move |_| Box::new(move |state| f(state)))
    }

    /// Convenience method for modifying shared state directly from a callback.
    /// Similar to `reduce_callback` but it also accepts the fired event.
    fn reduce_callback_with<T: 'static>(
        &self,
        f: impl FnOnce(T, &mut Model<Self::Handler>) + Copy + 'static,
    ) -> Callback<T>
    where
        Model<Self::Handler>: 'static,
    {
        self.callback()
            .reform(move |e| Box::new(move |state| f(e, state)))
    }
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
    _mark: std::marker::PhantomData<H>,
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

    fn callback(&self) -> &Callback<Reduction<T>> {
        &self.callback
    }

    fn state(&self) -> &T {
        &self.state
    }

    fn __set_local(&mut self, state: &Rc<T>, callback: &Callback<Reduction<T>>) {
        self.state = state.clone();
        self.callback = callback.clone();
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
