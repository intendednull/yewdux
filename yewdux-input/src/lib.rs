#[cfg(feature = "functional")]
pub mod functional;

use std::{cell::RefCell, rc::Rc};

use yew::{
    prelude::{Callback, InputData},
    ChangeData,
};
use yew_services::reader::{File, FileData, ReaderService};
use yewdux::{
    dispatch::{Dispatch, Dispatcher},
    service::ServiceBridge,
    store::Store,
};

#[derive(Clone, PartialEq)]
/// A basic [Dispatcher].
pub struct InputDispatch<S: Store>(pub(crate) Dispatch<S>);

impl<S: Store> InputDispatch<S> {
    /// Dispatch without receiving capabilities. Able to send messages, though all state/output
    /// responses are ignored.
    pub fn new() -> Self {
        Self(Dispatch::new())
    }

    /// Callback that sets state, ignoring callback event.
    pub fn set<E: 'static>(&self, f: impl FnOnce(&mut S::Model) + 'static) -> Callback<E> {
        self.0.reduce_callback_once(f)
    }

    /// Callback that sets state from callback event
    pub fn set_with<E: 'static>(&self, f: impl FnOnce(&mut S::Model, E) + 'static) -> Callback<E> {
        self.0.reduce_callback_once_with(f)
    }

    /// Callback for setting state from `InputData`.
    pub fn input(&self, f: impl Fn(&mut S::Model, String) + 'static) -> Callback<InputData> {
        self.0
            .reduce_callback_with(f)
            .reform(|data: InputData| data.value)
    }

    /// Callback for setting state from `InputData`.
    pub fn input_once(
        &self,
        f: impl FnOnce(&mut S::Model, String) + 'static,
    ) -> Callback<InputData> {
        self.0
            .reduce_callback_once_with(f)
            .reform(|data: InputData| data.value)
    }

    /// Callback for setting state from `InputData`.
    pub fn text(&self, f: impl Fn(&mut S::Model, String) + 'static) -> Callback<ChangeData> {
        let on_change = self.0.reduce_callback_with(f);
        Callback::from(move |data: ChangeData| {
            if let ChangeData::Value(val) = data {
                on_change.emit(val);
            }
        })
    }

    /// Callback for setting state from `InputData`.
    pub fn text_once(
        &self,
        f: impl FnOnce(&mut S::Model, String) + 'static,
    ) -> Callback<ChangeData> {
        let on_change = self.0.reduce_callback_once_with(f);
        Callback::from(move |data: ChangeData| {
            if let ChangeData::Value(val) = data {
                on_change.emit(val);
            }
        })
    }

    pub fn select(&self, f: impl Fn(&mut S::Model, String) + 'static) -> Callback<ChangeData> {
        let on_change = self.0.reduce_callback_with(f);
        Callback::from(move |data: ChangeData| {
            if let ChangeData::Select(el) = data {
                on_change.emit(el.value());
            }
        })
    }

    pub fn select_once(
        &self,
        f: impl FnOnce(&mut S::Model, String) + 'static,
    ) -> Callback<ChangeData> {
        let on_change = self.0.reduce_callback_once_with(f);
        Callback::from(move |data: ChangeData| {
            if let ChangeData::Select(el) = data {
                on_change.emit(el.value());
            }
        })
    }

    /// Callback for setting files
    pub fn file(
        &self,
        f: impl Fn(&mut S::Model, FileData) + Copy + 'static,
    ) -> Callback<ChangeData> {
        let set_file = self.set_with(f);
        Callback::from(move |data| {
            if let ChangeData::Files(files) = data {
                for file in js_sys::try_iter(&files)
                    .unwrap()
                    .unwrap()
                    .into_iter()
                    .map(|v| File::from(v.unwrap()))
                {
                    ReaderService::read_file(file, set_file.clone()).ok();
                }
            }
        })
    }
}

impl<STORE: Store> Dispatcher for InputDispatch<STORE> {
    type Store = STORE;

    fn bridge(&self) -> &Rc<RefCell<ServiceBridge<Self::Store>>> {
        &self.0.bridge()
    }
}
