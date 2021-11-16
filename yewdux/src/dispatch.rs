//! Primary interface to a [Store](crate::store::Store)
use std::rc::Rc;
use std::{cell::RefCell, future::Future};

use yew::{Callback, Properties};

use crate::{
    service::{ServiceBridge, ServiceOutput, ServiceRequest, ServiceResponse},
    store::Store,
};

type Model<T> = <T as Store>::Model;

/// Primary interface to a [Store](crate::store::Store)
pub trait Dispatcher {
    type Store: Store;

    #[doc(hidden)]
    fn bridge(&self) -> Rc<RefCell<ServiceBridge<Self::Store>>>;

    /// Send an input message.
    ///
    /// ```ignore
    /// dispatch.send(StoreMsg::AddOne)
    /// ```
    fn send(&self, msg: impl Into<<Self::Store as Store>::Input>) {
        self.bridge().borrow_mut().send_store(msg.into())
    }

    /// Callback for sending input messages.
    ///
    /// ```ignore
    /// let onclick = dispatch.callback(|_| StoreMsg::AddOne);
    /// html! { <button onclick=onclick>{"+1"}</button> }
    /// ```
    fn callback<E, M>(&self, f: impl Fn(E) -> M + 'static) -> Callback<E>
    where
        M: Into<<Self::Store as Store>::Input>,
    {
        let bridge = self.bridge();
        let f = Rc::new(f);
        Callback::from(move |e| {
            let msg = f(e);
            bridge.borrow_mut().send_store(msg.into())
        })
    }

    /// Once variation of [Self::callback].
    fn callback_once<E, M>(&self, f: impl Fn(E) -> M + 'static) -> Callback<E>
    where
        M: Into<<Self::Store as Store>::Input>,
    {
        let bridge = self.bridge();
        let f = Rc::new(f);
        Callback::once(move |e| {
            let msg = f(e);
            bridge.borrow_mut().send_store(msg.into())
        })
    }

    /// Send a message that applies given function to shared state. Changes are not immediate, and
    /// [should be handled as needed](Dispatch::bridge_state).
    ///
    /// ```ignore
    /// let onclick = dispatch.reduce(|_| StoreMsg::AddOne);
    /// html! { <button onclick=onclick>{"+1"}</button> }
    /// ```
    fn reduce<F, R>(&self, f: F)
    where
        F: FnOnce(&mut Model<Self::Store>) -> R + 'static,
    {
        self.bridge()
            .borrow_mut()
            .send_service(ServiceRequest::Reduce(Box::new(move |state| {
                f(state);
            })))
    }

    /// Like [reduce](Self::reduce) but from a callback.
    ///
    /// ```ignore
    /// let onclick = dispatch.reduce_callback(|s| s.count += 1);
    /// html! { <button onclick=onclick>{"+1"}</button> }
    /// ```
    fn reduce_callback<F, R, E>(&self, f: F) -> Callback<E>
    where
        F: Fn(&mut Model<Self::Store>) -> R + 'static,
        E: 'static,
    {
        let bridge = self.bridge();
        let f = Rc::new(f);
        Callback::from(move |_| {
            bridge.borrow_mut().send_service(ServiceRequest::Reduce({
                let f = f.clone();
                Box::new(move |state| {
                    f(state);
                })
            }))
        })
    }

    /// Similar to [Self::reduce_callback] but also provides the fired event.
    ///
    /// ```ignore
    /// let oninput = dispatch.reduce_callback(|s, i: InputData| s.user_name i.value);
    /// html! { <input oninput=oninput>{"+1"}</input> }
    /// ```
    fn reduce_callback_with<F, R, E>(&self, f: F) -> Callback<E>
    where
        F: Fn(&mut Model<Self::Store>, E) -> R + 'static,
        E: 'static,
    {
        let bridge = self.bridge();
        let f = Rc::new(f);
        Callback::from(move |e: E| {
            let f = f.clone();
            bridge.borrow_mut().send_service(ServiceRequest::Reduce({
                Box::new(move |state| {
                    f(state, e);
                })
            }))
        })
    }

    /// Once variation of [Self::reduce_callback].
    fn reduce_callback_once<F, R, E>(&self, f: F) -> Callback<E>
    where
        F: FnOnce(&mut Model<Self::Store>) -> R + 'static,
        E: 'static,
    {
        let bridge = self.bridge();
        Callback::once(move |_| {
            bridge
                .borrow_mut()
                .send_service(ServiceRequest::Reduce(Box::new(move |state| {
                    f(state);
                })))
        })
    }

    /// Once variation of [Self::reduce_callback_with].
    fn reduce_callback_once_with<F, R, E>(&self, f: F) -> Callback<E>
    where
        F: FnOnce(&mut Model<Self::Store>, E) -> R + 'static,
        E: 'static,
    {
        let bridge = self.bridge();
        Callback::once(move |e: E| {
            bridge
                .borrow_mut()
                .send_service(ServiceRequest::Reduce(Box::new(move |state| {
                    f(state, e);
                })))
        })
    }

    fn future<F, FU, OUT>(&self, f: F)
    where
        F: FnOnce(Self) -> FU + 'static,
        FU: Future<Output = OUT> + 'static,
        OUT: 'static,
        Self: Clone + 'static,
    {
        let this = self.clone();
        self.bridge()
            .borrow_mut()
            .send_service(ServiceRequest::Future(Box::pin(async move {
                f(this).await;
            })))
    }

    fn future_callback<F, FU, OUT, E>(&self, f: F) -> Callback<E>
    where
        F: Fn(Self) -> FU + 'static,
        FU: Future<Output = OUT>,
        OUT: 'static,
        Self: Clone + 'static,
        E: 'static,
    {
        let this = self.clone();
        let bridge = this.bridge();
        let f = Rc::new(f);
        Callback::from(move |_| {
            let this = this.clone();
            bridge
                .borrow_mut()
                .send_service(ServiceRequest::Future(Box::pin({
                    let f = f.clone();
                    async move {
                        f(this).await;
                    }
                })))
        })
    }

    fn future_callback_with<F, FU, OUT, E>(&self, f: F) -> Callback<E>
    where
        F: Fn(Self, E) -> FU + 'static,
        FU: Future<Output = OUT>,
        OUT: 'static,
        Self: Clone + 'static,
        E: 'static,
    {
        let this = self.clone();
        let bridge = this.bridge();
        let f = Rc::new(f);
        Callback::from(move |e| {
            let this = this.clone();
            bridge
                .borrow_mut()
                .send_service(ServiceRequest::Future(Box::pin({
                    let f = f.clone();
                    async move {
                        f(this, e).await;
                    }
                })))
        })
    }

    fn future_callback_once<F, FU, OUT, E>(&self, f: F) -> Callback<E>
    where
        F: FnOnce(Self) -> FU + 'static,
        FU: Future<Output = OUT>,
        OUT: 'static,
        Self: Clone + 'static,
        E: 'static,
    {
        let this = self.clone();
        let bridge = this.bridge();
        Callback::once(move |_| {
            bridge
                .borrow_mut()
                .send_service(ServiceRequest::Future(Box::pin({
                    async move {
                        f(this).await;
                    }
                })))
        })
    }

    fn future_callback_once_with<F, FU, OUT, E>(&self, f: F) -> Callback<E>
    where
        F: FnOnce(Self, E) -> FU + 'static,
        FU: Future<Output = OUT>,
        OUT: 'static,
        Self: Clone + 'static,
        E: 'static,
    {
        let this = self.clone();
        let bridge = this.bridge();
        Callback::once(move |e| {
            bridge
                .borrow_mut()
                .send_service(ServiceRequest::Future(Box::pin({
                    async move {
                        f(this, e).await;
                    }
                })))
        })
    }
}

/// A basic [Dispatcher].
pub struct Dispatch<STORE: Store, SCOPE: 'static = STORE> {
    pub(crate) bridge: Rc<RefCell<ServiceBridge<STORE, SCOPE>>>,
}

impl<STORE: Store, SCOPE: 'static> Dispatch<STORE, SCOPE> {
    /// Dispatch without receiving capabilities. Able to send messages, though all state/output
    /// responses are ignored.
    pub fn new() -> Self {
        Self {
            bridge: Rc::new(RefCell::new(ServiceBridge::new(Callback::noop()))),
        }
    }

    /// Dispatch with callbacks to receive new state and Store output messages.
    pub fn bridge(
        on_state: Callback<Rc<STORE::Model>>,
        on_output: Callback<STORE::Output>,
    ) -> Self {
        let cb = Callback::from(move |msg| match msg {
            ServiceOutput::Store(msg) => on_output.emit(msg),
            ServiceOutput::Service(msg) => match msg {
                ServiceResponse::State(state) => on_state.emit(state),
            },
        });
        Self {
            bridge: Rc::new(RefCell::new(ServiceBridge::new(cb))),
        }
    }

    /// Dispatch with callback to receive new state.
    pub fn bridge_state(on_state: Callback<Rc<STORE::Model>>) -> Self {
        let cb = Callback::from(move |msg| match msg {
            ServiceOutput::Store(_) => {}
            ServiceOutput::Service(msg) => match msg {
                ServiceResponse::State(state) => on_state.emit(state),
            },
        });
        Self {
            bridge: Rc::new(RefCell::new(ServiceBridge::new(cb))),
        }
    }
}

impl<STORE: Store> Dispatcher for Dispatch<STORE> {
    type Store = STORE;

    fn bridge(&self) -> Rc<RefCell<ServiceBridge<Self::Store>>> {
        Rc::clone(&self.bridge)
    }
}

impl<STORE: Store, SCOPE: 'static> Clone for Dispatch<STORE, SCOPE> {
    fn clone(&self) -> Self {
        Self {
            bridge: self.bridge.clone(),
        }
    }
}

impl<STORE: Store, SCOPE: 'static> PartialEq for Dispatch<STORE, SCOPE> {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.bridge, &other.bridge)
    }
}

impl<STORE: Store, SCOPE: 'static> std::fmt::Debug for Dispatch<STORE, SCOPE> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Dispatch").finish()
    }
}

impl<STORE: Store, SCOPE: 'static> Default for Dispatch<STORE, SCOPE> {
    fn default() -> Self {
        Self::new()
    }
}

/// Dispatch for component properties. Use with [WithDispatch](crate::prelude::WithDispatch) to
/// automatically manage message passing.
///
/// # Panics
/// Accessing methods from a component that isn't wrapped in `WithDispatch` will panic.
#[derive(Properties)]
pub struct DispatchProps<STORE: Store, SCOPE: 'static = STORE> {
    #[prop_or_default]
    pub(crate) state: RefCell<Option<Rc<Model<STORE>>>>,
    #[prop_or_default]
    pub(crate) dispatch: RefCell<Dispatch<STORE, SCOPE>>,
}

impl<STORE: Store, SCOPE: 'static> DispatchProps<STORE, SCOPE> {
    #[allow(unused)]
    pub(crate) fn new(on_state: Callback<Rc<STORE::Model>>) -> Self {
        Self {
            state: Default::default(),
            dispatch: Dispatch::bridge_state(on_state).into(),
        }
    }

    pub fn state(&self) -> Rc<Model<STORE>> {
        Rc::clone(
            self.state
                .borrow()
                .as_ref()
                .expect("State accessed prematurely. Missing WithDispatch?"),
        )
    }
}

impl<STORE: Store> Dispatcher for DispatchProps<STORE> {
    type Store = STORE;

    fn bridge(&self) -> Rc<RefCell<ServiceBridge<Self::Store>>> {
        self.dispatch.borrow().bridge()
    }
}

impl<STORE: Store> WithDispatchProps for DispatchProps<STORE> {
    type Store = STORE;

    fn dispatch(&self) -> &DispatchProps<Self::Store> {
        self
    }
}

impl<STORE: Store, SCOPE: 'static> Default for DispatchProps<STORE, SCOPE> {
    fn default() -> Self {
        Self {
            state: Default::default(),
            dispatch: Default::default(),
        }
    }
}

impl<STORE: Store, SCOPE: 'static> Clone for DispatchProps<STORE, SCOPE> {
    fn clone(&self) -> Self {
        Self {
            state: self.state.clone(),
            dispatch: self.dispatch.clone(),
        }
    }
}

impl<STORE: Store, SCOPE: 'static> PartialEq for DispatchProps<STORE, SCOPE> {
    fn eq(&self, other: &Self) -> bool {
        self.dispatch == other.dispatch
            && self
                .state
                .borrow()
                .as_ref()
                .zip(other.state.borrow().as_ref())
                .map(|(a, b)| Rc::ptr_eq(a, b))
                .unwrap_or(false)
    }
}

impl<STORE: Store, SCOPE: 'static> std::fmt::Debug for DispatchProps<STORE, SCOPE>
where
    STORE::Model: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DispatchProps")
            .field("state", &self.state)
            .finish()
    }
}

/// Allows any properties to work with [WithDispatch](crate::prelude::WithDispatch).
///
/// # Example
///
/// ```
/// # #[derive(Clone, Default)]
/// # struct MyState;
/// # use yew::Properties;
/// # use yewdux::prelude::*;
///
/// #[derive(Clone, PartialEq, Properties)]
/// struct Props {
///     dispatch: DispatchProps<BasicStore<MyState>>,
/// }
///
/// impl WithDispatchProps for Props {
///     type Store = BasicStore<MyState>;
///
///     fn dispatch(&self) -> &DispatchProps<Self::Store> {
///         &self.dispatch
///     }
/// }
pub trait WithDispatchProps {
    type Store: Store;

    fn dispatch(&self) -> &DispatchProps<Self::Store>;
}

#[cfg(test)]
pub mod tests {
    use crate::prelude::BasicStore;

    use super::*;

    #[test]
    fn dispatch_impl_debug() {
        #[derive(Debug)]
        struct Foo {
            dispatch: Dispatch<BasicStore<()>>,
        }
    }

    #[test]
    fn dispatch_props_impl_debug() {
        #[derive(Debug)]
        struct Foo {
            dispatch: DispatchProps<BasicStore<()>>,
        }
    }
}
