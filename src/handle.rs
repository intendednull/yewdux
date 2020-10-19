//! Ergonomic interface with shared state.
use std::rc::Rc;

use yew::{agent::AgentLink, Callback, Properties};

use super::handler::{Reduction, ReductionOnce, SharedHandler, StateHandler, StorageHandler};
use crate::component::wrapper::SharedStateService;

type Model<T> = <T as StateHandler>::Model;

pub trait WrapperHandle: StateHandle {
    fn set_state(&mut self, state: Rc<Model<Self::Handler>>);
    fn set_callbacks(
        &mut self,
        callback: Callback<Reduction<Model<Self::Handler>>>,
        callback_once: Callback<ReductionOnce<Model<Self::Handler>>>,
    );
    fn set_link(&mut self, _link: AgentLink<SharedStateService<Self::Handler, Self::Scope>>) {}
}

/// Provides mutable access for wrapper component to update
pub trait StateHandle {
    type Handler: StateHandler;
    type Scope;

    /// Current state.
    fn state(&self) -> &Rc<Model<Self::Handler>>;

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
pub struct StateHandleFoo<HANDLER, SCOPE>
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
    #[prop_or_default]
    _mark_scope: std::marker::PhantomData<SCOPE>,
}

impl<HANDLER, SCOPE> StateHandle for StateHandleFoo<HANDLER, SCOPE>
where
    HANDLER: StateHandler,
    Model<HANDLER>: Default + Clone,
{
    type Handler = HANDLER;
    type Scope = SCOPE;

    fn state(&self) -> &Rc<Model<Self::Handler>> {
        &self.state
    }
    fn callback(&self) -> &Callback<Reduction<Model<Self::Handler>>> {
        &self.callback
    }

    fn callback_once(&self) -> &Callback<ReductionOnce<Model<Self::Handler>>> {
        &self.callback_once
    }
}

impl<HANDLER, SCOPE> WrapperHandle for StateHandleFoo<HANDLER, SCOPE>
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

impl<HANDLER, SCOPE> SharedState for StateHandleFoo<HANDLER, SCOPE>
where
    HANDLER: StateHandler,
    Model<HANDLER>: Default + Clone,
{
    type Handle = Self;

    fn handle(&mut self) -> &mut Self::Handle {
        self
    }
}

impl<HANDLER, SCOPE> Clone for StateHandleFoo<HANDLER, SCOPE>
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
            _mark_scope: Default::default(),
        }
    }
}

impl<HANDLER, SCOPE> PartialEq for StateHandleFoo<HANDLER, SCOPE>
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
pub type SharedHandle<T, SCOPE = SharedHandler<T>> = StateHandleFoo<SharedHandler<T>, SCOPE>;
/// Handle for shared state with persistent storage.
pub type StorageHandle<T, SCOPE = StorageHandler<T>> = StateHandleFoo<StorageHandler<T>, SCOPE>;

/// Interface to shared state
#[derive(Properties)]
pub struct AgentHandle<HANDLER, SCOPE>
where
    HANDLER: StateHandler + 'static,
    Model<HANDLER>: Clone + 'static,
    SCOPE: 'static,
{
    #[prop_or_default]
    state: Option<Rc<Model<HANDLER>>>,
    #[prop_or_default]
    callback: Callback<Reduction<Model<HANDLER>>>,
    #[prop_or_default]
    callback_once: Callback<ReductionOnce<Model<HANDLER>>>,
    link: Option<AgentLink<SharedStateService<HANDLER, SCOPE>>>,
}

impl<HANDLER, SCOPE> AgentHandle<HANDLER, SCOPE>
where
    HANDLER: StateHandler,
    Model<HANDLER>: Clone + 'static,
{
    pub fn link(&self) -> &AgentLink<SharedStateService<HANDLER, SCOPE>> {
        self.link.as_ref().expect(
            "Link accessed prematurely. Is your component wrapped in a SharedStateComponent?",
        )
    }
}

impl<HANDLER, SCOPE> StateHandle for AgentHandle<HANDLER, SCOPE>
where
    HANDLER: StateHandler,
    Model<HANDLER>: Clone,
{
    type Handler = HANDLER;
    type Scope = SCOPE;

    fn state(&self) -> &Rc<Model<HANDLER>> {
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

impl<HANDLER, SCOPE> WrapperHandle for AgentHandle<HANDLER, SCOPE>
where
    HANDLER: StateHandler,
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

    fn set_link(&mut self, link: AgentLink<SharedStateService<Self::Handler, Self::Scope>>) {
        self.link = Some(link);
    }
}

impl<HANDLER, SCOPE> SharedState for AgentHandle<HANDLER, SCOPE>
where
    HANDLER: StateHandler,
    Model<HANDLER>: Clone,
{
    type Handle = Self;

    fn handle(&mut self) -> &mut Self::Handle {
        self
    }
}

impl<HANDLER, SCOPE> Default for AgentHandle<HANDLER, SCOPE>
where
    HANDLER: StateHandler,
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

impl<HANDLER, SCOPE> Clone for AgentHandle<HANDLER, SCOPE>
where
    HANDLER: StateHandler,
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

impl<HANDLER, SCOPE> PartialEq for AgentHandle<HANDLER, SCOPE>
where
    HANDLER: StateHandler,
    Model<HANDLER>: PartialEq + Clone,
{
    fn eq(&self, other: &Self) -> bool {
        self.state == other.state
            && self.callback == other.callback
            && self.callback_once == other.callback_once
    }
}

pub type LinkHandle<HANDLER, SCOPE = HANDLER> = AgentHandle<HANDLER, SCOPE>;
