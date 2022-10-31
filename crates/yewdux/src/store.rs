//! Unique state shared application-wide
use std::{future::Future, rc::Rc};

use async_trait::async_trait;

pub use yewdux_macros::Store;

/// Globally shared state.
pub trait Store: 'static {
    /// Create this store.
    fn new() -> Self;

    /// Indicate whether or not subscribers should be notified about this change. Usually this
    /// should be set to `self != old`.
    fn should_notify(&self, old: &Self) -> bool;
}

/// A type that can change state.
pub trait Reducer<S> {
    /// Mutate state.
    fn apply(self, state: Rc<S>) -> Rc<S>;
}

impl<F, S> Reducer<S> for F
where
    F: FnOnce(Rc<S>) -> Rc<S>,
{
    fn apply(self, state: Rc<S>) -> Rc<S> {
        self(state)
    }
}

/// A type that can change state asynchronously.
#[async_trait(?Send)]
pub trait AsyncReducer<S> {
    /// Mutate state.
    async fn apply(self, state: Rc<S>) -> Rc<S>;
}

#[async_trait(?Send)]
impl<F, FU, S> AsyncReducer<S> for F
where
    S: 'static,
    F: FnOnce(Rc<S>) -> FU,
    FU: Future<Output = Rc<S>>,
{
    async fn apply(self, state: Rc<S>) -> Rc<S> {
        self(state).await
    }
}
