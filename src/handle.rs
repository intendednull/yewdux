//! Ergonomic interface with shared state.
use std::rc::Rc;

use yew::{Callback, Properties};

use crate::handler::{
    HandlerLink, Reduction, ReductionOnce, SharedHandler, StateHandler, StorageHandler,
};

/// Handle for basic shared state.
pub type SharedHandle<T> = StateHandle<SharedHandler<T>>;
/// Handle for shared state with persistent storage.

pub type StorageHandle<T> = StateHandle<StorageHandler<T>>;

type Model<T> = <T as StateHandler>::Model;

pub trait Handle {
    type Handler: StateHandler;
}

impl<HANDLER> Handle for StateHandle<HANDLER>
where
    HANDLER: StateHandler,
    <HANDLER as StateHandler>::Message: Clone,
    <HANDLER as StateHandler>::Output: Clone,
    <HANDLER as StateHandler>::Input: Clone,
    Model<HANDLER>: Clone + 'static,
{
    type Handler = HANDLER;
}

pub trait SharedState {
    type Handle: Handle;

    fn handle(&mut self) -> &mut Self::Handle;
}

/// Provides mutable access for wrapper component to update
pub trait WrapperHandle: Handle {
    fn set_state(&mut self, state: Rc<Model<Self::Handler>>);
    fn set_callbacks(
        &mut self,
        callback: Callback<Reduction<Model<Self::Handler>>>,
        callback_once: Callback<ReductionOnce<Model<Self::Handler>>>,
    );
    fn set_link(&mut self, _link: HandlerLink<Self::Handler>) {}
}

impl<HANDLER> WrapperHandle for StateHandle<HANDLER>
where
    HANDLER: StateHandler,
    <HANDLER as StateHandler>::Message: Clone,
    <HANDLER as StateHandler>::Output: Clone,
    <HANDLER as StateHandler>::Input: Clone,
    Model<HANDLER>: Clone + 'static,
{
    fn set_state(&mut self, state: Rc<Model<Self::Handler>>) {
        self.state = Some(state);
    }

    fn set_link(&mut self, link: HandlerLink<Self::Handler>) {
        self.link = Some(link);
    }

    fn set_callbacks(
        &mut self,
        callback: Callback<Reduction<Model<Self::Handler>>>,
        callback_once: Callback<ReductionOnce<Model<Self::Handler>>>,
    ) {
        self.callback = callback;
        self.callback_once = callback_once;
    }
}

/// Interface to shared state
#[derive(Properties)]
pub struct StateHandle<HANDLER>
where
    HANDLER: StateHandler,
    <HANDLER as StateHandler>::Message: Clone,
    <HANDLER as StateHandler>::Output: Clone,
    <HANDLER as StateHandler>::Input: Clone,
    Model<HANDLER>: Clone + 'static,
{
    #[prop_or_default]
    state: Option<Rc<Model<HANDLER>>>,
    #[prop_or_default]
    callback: Callback<Reduction<Model<HANDLER>>>,
    #[prop_or_default]
    callback_once: Callback<ReductionOnce<Model<HANDLER>>>,
    #[prop_or_default]
    link: Option<HandlerLink<HANDLER>>,
}

impl<HANDLER> StateHandle<HANDLER>
where
    HANDLER: StateHandler,
    <HANDLER as StateHandler>::Message: Clone,
    <HANDLER as StateHandler>::Output: Clone,
    <HANDLER as StateHandler>::Input: Clone,
    Model<HANDLER>: Clone + 'static,
{
    pub fn link(&self) -> &HandlerLink<HANDLER> {
        self.link.as_ref().expect(
            "Link accessed prematurely. Is your component wrapped in a SharedStateComponent?",
        )
    }

    pub fn state(&self) -> &Model<HANDLER> {
        self.state.as_ref().expect(
            "State accessed prematurely. Is your component wrapped in a SharedStateComponent?",
        )
    }

    /// Apply a function that may mutate shared state.
    /// Changes are not immediate, and must be handled in `Component::change`.
    pub fn reduce(&self, f: impl FnOnce(&mut Model<HANDLER>) + 'static) {
        self.callback_once.emit(Box::new(f))
    }

    /// Convenience method for modifying shared state directly from a `Callback`.
    /// The callback event is ignored here, see `reduce_callback_with` for the alternative.
    pub fn reduce_callback<E: 'static>(
        &self,
        f: impl Fn(&mut Model<HANDLER>) + 'static,
    ) -> Callback<E>
    where
        Model<HANDLER>: 'static,
    {
        let f = Rc::new(f);
        self.callback.reform(move |_| f.clone())
    }

    /// Convenience method for modifying shared state directly from a `CallbackOnce`.
    /// The callback event is ignored here, see `reduce_callback_once_with` for the alternative.
    pub fn reduce_callback_once<E: 'static>(
        &self,
        f: impl FnOnce(&mut Model<HANDLER>) + 'static,
    ) -> Callback<E>
    where
        Model<HANDLER>: 'static,
    {
        let f = Box::new(f);
        let cb = self.callback_once.clone();
        Callback::once(move |_| cb.emit(f))
    }

    /// Convenience method for modifying shared state directly from a `Callback`.
    /// Similar to `reduce_callback` but it also accepts the fired event.
    pub fn reduce_callback_with<E: 'static>(
        &self,
        f: impl Fn(&mut Model<HANDLER>, E) + 'static,
    ) -> Callback<E>
    where
        Model<HANDLER>: 'static,
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
        f: impl FnOnce(&mut Model<HANDLER>, E) + 'static,
    ) -> Callback<E>
    where
        Model<HANDLER>: 'static,
    {
        let cb = self.callback_once.clone();
        Callback::once(move |e| cb.emit(Box::new(move |state| f(state, e))))
    }
}

impl<HANDLER> SharedState for StateHandle<HANDLER>
where
    HANDLER: StateHandler,
    <HANDLER as StateHandler>::Message: Clone,
    <HANDLER as StateHandler>::Output: Clone,
    <HANDLER as StateHandler>::Input: Clone,
    Model<HANDLER>: Clone,
{
    type Handle = Self;

    fn handle(&mut self) -> &mut Self::Handle {
        self
    }
}

impl<HANDLER> Default for StateHandle<HANDLER>
where
    HANDLER: StateHandler,
    <HANDLER as StateHandler>::Message: Clone,
    <HANDLER as StateHandler>::Output: Clone,
    <HANDLER as StateHandler>::Input: Clone,
    Model<HANDLER>: Clone,
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

impl<HANDLER> Clone for StateHandle<HANDLER>
where
    HANDLER: StateHandler,
    HandlerLink<HANDLER>: Clone,
    <HANDLER as StateHandler>::Message: Clone,
    <HANDLER as StateHandler>::Output: Clone,
    <HANDLER as StateHandler>::Input: Clone,
    Model<HANDLER>: Clone,
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

impl<HANDLER> PartialEq for StateHandle<HANDLER>
where
    HANDLER: StateHandler,
    <HANDLER as StateHandler>::Message: Clone,
    <HANDLER as StateHandler>::Output: Clone,
    <HANDLER as StateHandler>::Input: Clone,
    Model<HANDLER>: Clone,
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
