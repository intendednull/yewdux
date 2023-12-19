//! Unique state shared application-wide
use std::rc::Rc;

pub use yewdux_macros::Store;

use crate::Context;

/// A type that holds application state.
pub trait Store: 'static {
    /// Create this store.
    fn new(cx: &Context) -> Self;

    /// Indicate whether or not subscribers should be notified about this change. Usually this
    /// should be set to `self != old`.
    fn should_notify(&self, old: &Self) -> bool;
}

/// A type that can change state.
///
/// ```
/// use std::rc::Rc;
///
/// use yew::prelude::*;
/// use yewdux::prelude::*;
///
/// #[derive(Default, Clone, PartialEq, Eq, Store)]
/// struct Counter {
///     count: u32,
/// }
///
/// enum Msg {
///     AddOne,
/// }
///
/// impl Reducer<Counter> for Msg {
///     fn apply(self, mut counter: Rc<Counter>) -> Rc<Counter> {
///         let state = Rc::make_mut(&mut counter);
///         match self {
///             Msg::AddOne => state.count += 1,
///         };
///
///         counter
///     }
/// }
///
/// #[function_component]
/// fn App() -> Html {
///     let (counter, dispatch) = use_store::<Counter>();
///     let onclick = dispatch.apply_callback(|_| Msg::AddOne);
///
///     html! {
///         <>
///         <p>{ counter.count }</p>
///         <button {onclick}>{"+1"}</button>
///         </>
///     }
/// }
/// ```
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
