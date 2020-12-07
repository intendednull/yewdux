//! Ergonomic interface with shared state.
use std::rc::Rc;

use yew::{Callback, Properties};

use crate::handler::{
    HandlerLink, Reduction, ReductionOnce, SharedHandler, StateHandler, StorageHandler,
};

type Model<T> = <T as StateHandler>::Model;

/// Provides mutable access for wrapper component to update
pub trait WrapperHandle: StateHandle {
    fn set_state(&mut self, state: Rc<Model<Self::Handler>>);
    fn set_callbacks(
        &mut self,
        callback: Callback<Reduction<Model<Self::Handler>>>,
        callback_once: Callback<ReductionOnce<Model<Self::Handler>>>,
    );
    fn set_link(&mut self, _link: HandlerLink<Self::Handler>) {}
}

pub trait StateHandle {
    type Handler: StateHandler;

    /// Current state.
    fn state(&self) -> &Model<Self::Handler>;

    fn callback(&self) -> &Callback<Reduction<Model<Self::Handler>>>;
    fn callback_once(&self) -> &Callback<ReductionOnce<Model<Self::Handler>>>;

    /// Apply a function that may mutate shared state.
    /// Changes are not immediate, and must be handled in `Component::change`.
    fn reduce(&self, f: impl FnOnce(&mut Model<Self::Handler>) + 'static) {
        self.callback_once().emit(Box::new(f))
    }

    /// Convenience method for modifying shared state directly from a `Callback`.
    /// The callback event is ignored here, see `reduce_callback_with` for the alternative.
    fn reduce_callback<E: 'static>(
        &self,
        f: impl Fn(&mut Model<Self::Handler>) + 'static,
    ) -> Callback<E>
    where
        Model<Self::Handler>: 'static,
    {
        let f = Rc::new(f);
        self.callback().reform(move |_| f.clone())
    }

    /// Convenience method for modifying shared state directly from a `CallbackOnce`.
    /// The callback event is ignored here, see `reduce_callback_once_with` for the alternative.
    fn reduce_callback_once<E: 'static>(
        &self,
        f: impl FnOnce(&mut Model<Self::Handler>) + 'static,
    ) -> Callback<E>
    where
        Model<Self::Handler>: 'static,
    {
        let f = Box::new(f);
        let cb = self.callback_once().clone();
        Callback::once(move |_| cb.emit(f))
    }

    /// Convenience method for modifying shared state directly from a `Callback`.
    /// Similar to `reduce_callback` but it also accepts the fired event.
    fn reduce_callback_with<E: 'static>(
        &self,
        f: impl Fn(&mut Model<Self::Handler>, E) + 'static,
    ) -> Callback<E>
    where
        Model<Self::Handler>: 'static,
        E: Clone,
    {
        let f = Rc::new(f);
        self.callback().reform(move |e: E| {
            let f = f.clone();
            Rc::new(move |state| f.clone()(state, e.clone()))
        })
    }

    /// Convenience method for modifying shared state directly from a `CallbackOnce`.
    /// Similar to `reduce_callback` but it also accepts the fired event.
    fn reduce_callback_once_with<E: 'static>(
        &self,
        f: impl FnOnce(&mut Model<Self::Handler>, E) + 'static,
    ) -> Callback<E>
    where
        Model<Self::Handler>: 'static,
    {
        let cb = self.callback_once().clone();
        Callback::once(move |e| cb.emit(Box::new(move |state| f(state, e))))
    }
}

/// Provides mutable access for wrapper.
pub trait SharedState {
    type Handle: StateHandle;
    fn handle(&mut self) -> &mut Self::Handle;
}

/// Interface to shared state
#[derive(Default, Properties)]
pub struct StateHandleFoo<HANDLER>
where
    HANDLER: StateHandler,
    Model<HANDLER>: Default + Clone + 'static,
{
    #[prop_or_default]
    state: Rc<Model<HANDLER>>,
    #[prop_or_default]
    callback: Callback<Reduction<Model<HANDLER>>>,
    #[prop_or_default]
    callback_once: Callback<ReductionOnce<Model<HANDLER>>>,
    #[prop_or_default]
    _mark_handler: std::marker::PhantomData<HANDLER>,
}

impl<HANDLER> StateHandle for StateHandleFoo<HANDLER>
where
    HANDLER: StateHandler,
    Model<HANDLER>: Default + Clone,
{
    type Handler = HANDLER;

    fn state(&self) -> &Model<Self::Handler> {
        &self.state
    }
    fn callback(&self) -> &Callback<Reduction<Model<Self::Handler>>> {
        &self.callback
    }

    fn callback_once(&self) -> &Callback<ReductionOnce<Model<Self::Handler>>> {
        &self.callback_once
    }
}

impl<HANDLER> WrapperHandle for StateHandleFoo<HANDLER>
where
    HANDLER: StateHandler,
    Model<HANDLER>: Default + Clone,
{
    fn set_state(&mut self, state: Rc<Model<Self::Handler>>) {
        self.state = state;
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

impl<HANDLER> SharedState for StateHandleFoo<HANDLER>
where
    HANDLER: StateHandler,
    Model<HANDLER>: Default + Clone,
{
    type Handle = Self;

    fn handle(&mut self) -> &mut Self::Handle {
        self
    }
}

impl<HANDLER> Clone for StateHandleFoo<HANDLER>
where
    HANDLER: StateHandler,
    Model<HANDLER>: Default + Clone,
{
    fn clone(&self) -> Self {
        Self {
            state: self.state.clone(),
            callback: self.callback.clone(),
            callback_once: self.callback_once.clone(),
            _mark_handler: Default::default(),
        }
    }
}

impl<HANDLER> PartialEq for StateHandleFoo<HANDLER>
where
    HANDLER: StateHandler,
    Model<HANDLER>: PartialEq + Default + Clone,
{
    fn eq(&self, other: &Self) -> bool {
        self.state == other.state
            && self.callback == other.callback
            && self.callback_once == other.callback_once
    }
}

/// Handle for basic shared state.
pub type SharedHandle<T> = StateHandleFoo<SharedHandler<T>>;
/// Handle for shared state with persistent storage.
pub type StorageHandle<T> = StateHandleFoo<StorageHandler<T>>;

/// Interface to shared state
#[derive(Properties)]
pub struct LinkHandle<HANDLER>
where
    HANDLER: StateHandler + Clone + 'static,
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

impl<HANDLER> LinkHandle<HANDLER>
where
    HANDLER: StateHandler + Clone,
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
}

impl<HANDLER> StateHandle for LinkHandle<HANDLER>
where
    HANDLER: Clone + StateHandler,
    <HANDLER as StateHandler>::Message: Clone,
    <HANDLER as StateHandler>::Output: Clone,
    <HANDLER as StateHandler>::Input: Clone,
    Model<HANDLER>: Clone,
{
    type Handler = HANDLER;

    fn state(&self) -> &Model<HANDLER> {
        self.state.as_ref().expect(
            "State accessed prematurely. Is your component wrapped in a SharedStateComponent?",
        )
    }

    fn callback(&self) -> &Callback<Reduction<Model<Self::Handler>>> {
        &self.callback
    }

    fn callback_once(&self) -> &Callback<ReductionOnce<Model<Self::Handler>>> {
        &self.callback_once
    }
}

impl<HANDLER> WrapperHandle for LinkHandle<HANDLER>
where
    HANDLER: StateHandler + Clone,
    <HANDLER as StateHandler>::Message: Clone,
    <HANDLER as StateHandler>::Output: Clone,
    <HANDLER as StateHandler>::Input: Clone,
    Model<HANDLER>: Clone,
{
    fn set_state(&mut self, state: Rc<Model<Self::Handler>>) {
        self.state = Some(state);
    }

    fn set_callbacks(
        &mut self,
        callback: Callback<Reduction<Model<Self::Handler>>>,
        callback_once: Callback<ReductionOnce<Model<Self::Handler>>>,
    ) {
        self.callback = callback;
        self.callback_once = callback_once;
    }

    fn set_link(&mut self, link: HandlerLink<HANDLER>) {
        self.link = Some(link);
    }
}

impl<HANDLER> SharedState for LinkHandle<HANDLER>
where
    HANDLER: StateHandler + Clone,
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

impl<HANDLER> Default for LinkHandle<HANDLER>
where
    HANDLER: StateHandler + Clone,
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

impl<HANDLER> Clone for LinkHandle<HANDLER>
where
    HANDLER: StateHandler + Clone,
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

impl<HANDLER> PartialEq for LinkHandle<HANDLER>
where
    HANDLER: StateHandler + Clone,
    <HANDLER as StateHandler>::Message: Clone,
    <HANDLER as StateHandler>::Output: Clone,
    <HANDLER as StateHandler>::Input: Clone,
    Model<HANDLER>: PartialEq + Clone,
{
    fn eq(&self, other: &Self) -> bool {
        self.state == other.state
            && self.callback == other.callback
            && self.callback_once == other.callback_once
    }
}
