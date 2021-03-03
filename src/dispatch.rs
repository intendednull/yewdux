//! Ergonomic interface with shared state.
use std::cell::RefCell;
use std::rc::Rc;

use yew::{Callback, Properties};

use crate::{
    service::{ServiceBridge, ServiceOutput, ServiceRequest, ServiceResponse},
    store::{Store, StoreLink},
};

type Model<T> = <T as Store>::Model;

pub trait DispatchProp {
    type Store: Store;

    fn dispatch(&mut self) -> &mut Dispatch<Self::Store>;
}

/// Interface to shared state
#[derive(Properties)]
pub struct Dispatch<STORE: Store, SCOPE: 'static = STORE> {
    #[prop_or_default]
    pub state: Option<Rc<Model<STORE>>>,
    #[prop_or_default]
    pub(crate) bridge: Option<Rc<RefCell<ServiceBridge<STORE, SCOPE>>>>,
}

impl<STORE: Store, SCOPE: 'static> Dispatch<STORE, SCOPE> {
    pub fn new(on_state: Callback<Rc<STORE::Model>>) -> Self {
        let cb = Callback::from(move |msg| match msg {
            ServiceOutput::Store(_) => {}
            ServiceOutput::Service(msg) => match msg {
                ServiceResponse::State(state) => on_state.emit(state),
            },
        });
        Self {
            state: Default::default(),
            bridge: Some(Rc::new(RefCell::new(ServiceBridge::new(cb)))),
        }
    }

    pub fn bridged(
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
            state: Default::default(),
            bridge: Some(Rc::new(RefCell::new(ServiceBridge::new(cb)))),
        }
    }

    /// Whether or not this dispatch is ready to be used.
    pub fn is_ready(&self) -> bool {
        self.state.is_some()
    }

    pub fn state(&self) -> &Model<STORE> {
        self.state.as_ref().expect("State accessed prematurely.")
    }

    fn bridge(&self) -> Rc<RefCell<ServiceBridge<STORE, SCOPE>>> {
        self.bridge
            .as_ref()
            .map(Rc::clone)
            .expect("Bridge accessed prematurely.")
    }

    /// Send a Store input message.
    pub fn send(&self, msg: impl Into<STORE::Input>) {
        self.bridge().borrow_mut().send_store(msg.into())
    }

    /// Callback for sending input message to store.
    pub fn callback<E>(&self, f: impl Fn(E) -> STORE::Input + 'static) -> Callback<E> {
        let bridge = self.bridge();
        let f = Rc::new(f);
        Callback::from(move |e| {
            let msg = f(e);
            bridge.borrow_mut().send_store(msg)
        })
    }

    /// Callback for sending input message to store.
    pub fn callback_once<E>(&self, f: impl Fn(E) -> STORE::Input + 'static) -> Callback<E> {
        let bridge = self.bridge();
        let f = Rc::new(f);
        Callback::once(move |e| {
            let msg = f(e);
            bridge.borrow_mut().send_store(msg)
        })
    }

    /// Apply a function that may mutate shared state.
    /// Changes are not immediate, and must be handled in `Component::change`.
    pub fn reduce(&self, f: impl FnOnce(&mut Model<STORE>) + 'static) {
        self.bridge()
            .borrow_mut()
            .send_service(ServiceRequest::ApplyOnce(Box::new(f)))
    }

    /// Convenience method for modifying shared state directly from a `Callback`.
    /// The callback event is ignored here, see `reduce_callback_with` for the alternative.
    pub fn reduce_callback<E: 'static>(
        &self,
        f: impl Fn(&mut Model<STORE>) + 'static,
    ) -> Callback<E> {
        let bridge = self.bridge();
        let f = Rc::new(f);
        Callback::from(move |_| {
            bridge
                .borrow_mut()
                .send_service(ServiceRequest::Apply(f.clone()))
        })
    }

    /// Convenience method for modifying shared state directly from a `CallbackOnce`.
    /// The callback event is ignored here, see `reduce_callback_once_with` for the alternative.
    pub fn reduce_callback_once<E: 'static>(
        &self,
        f: impl FnOnce(&mut Model<STORE>) + 'static,
    ) -> Callback<E> {
        let bridge = self.bridge();
        let f = Box::new(f);
        Callback::once(move |_| {
            bridge
                .borrow_mut()
                .send_service(ServiceRequest::ApplyOnce(f))
        })
    }

    /// Convenience method for modifying shared state directly from a `Callback`.
    /// Similar to `reduce_callback` but it also accepts the fired event.
    pub fn reduce_callback_with<E: 'static>(
        &self,
        f: impl Fn(&mut Model<STORE>, E) + 'static,
    ) -> Callback<E>
    where
        E: Clone,
    {
        let bridge = self.bridge();
        let f = Rc::new(f);
        Callback::from(move |e: E| {
            let e = e.clone();
            let f = f.clone();
            bridge
                .borrow_mut()
                .send_service(ServiceRequest::Apply(Rc::new(move |state| {
                    f.clone()(state, e.clone())
                })))
        })
    }

    /// Convenience method for modifying shared state directly from a `CallbackOnce`.
    /// Similar to `reduce_callback` but it also accepts the fired event.
    pub fn reduce_callback_once_with<E: 'static>(
        &self,
        f: impl FnOnce(&mut Model<STORE>, E) + 'static,
    ) -> Callback<E> {
        let bridge = self.bridge();
        Callback::once(move |e: E| {
            bridge
                .borrow_mut()
                .send_service(ServiceRequest::ApplyOnce(Box::new(|state| f(state, e))))
        })
    }
}

impl<STORE> DispatchProp for Dispatch<STORE>
where
    STORE: Store,
{
    type Store = STORE;

    fn dispatch(&mut self) -> &mut Dispatch<Self::Store> {
        self
    }
}

impl<STORE: Store, SCOPE: 'static> Default for Dispatch<STORE, SCOPE>
where
    STORE: Store,
{
    fn default() -> Self {
        Self {
            state: Default::default(),
            bridge: Default::default(),
        }
    }
}

impl<STORE: Store, SCOPE: 'static> Clone for Dispatch<STORE, SCOPE>
where
    STORE: Store,
    StoreLink<STORE>: Clone,
{
    fn clone(&self) -> Self {
        Self {
            state: self.state.clone(),
            bridge: self.bridge.clone(),
        }
    }
}

impl<STORE: Store, SCOPE: 'static> PartialEq for Dispatch<STORE, SCOPE>
where
    STORE: Store,
    STORE::Model: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.bridge
            .as_ref()
            .zip(other.bridge.as_ref())
            .map(|(a, b)| Rc::ptr_eq(a, b))
            .unwrap_or(false)
            && self.state == other.state
    }
}
