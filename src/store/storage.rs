use std::any::type_name;
use std::rc::Rc;

use serde::{Deserialize, Serialize};
use yew::format::Json;
use yew_services::{storage::Area, StorageService};

use super::{Store, StoreLink};

/// Allows state to be stored persistently in local or session storage.
pub trait StorageModel: Serialize + for<'a> Deserialize<'a> {
    /// The key used to save and load state from storage.
    fn key() -> &'static str {
        type_name::<Self>()
    }
    /// The area to store state.
    fn area() -> Area {
        Area::Local
    }
}

/// Handler for shared state with persistent storage.
///
/// If persistent storage is disabled it just behaves like a `SharedHandler`.
#[derive(Default)]
pub struct Storage<T> {
    state: Rc<T>,
    storage: Option<StorageService>,
}

impl<T> Storage<T>
where
    T: StorageModel + Default,
{
    pub fn new() -> Self {
        let mut this: Self = Default::default();
        this.storage = StorageService::new(T::area()).ok();
        this.load_state();
        this
    }

    pub fn load_state(&mut self) {
        let result = self.storage.as_mut().map(|s| s.restore(T::key()));
        if let Some(Json(Ok(state))) = result {
            self.state = state;
        }
    }

    pub fn save_state(&mut self) {
        if let Some(storage) = &mut self.storage {
            storage.store(T::key(), Json(&self.state));
        }
    }
}

impl<T> Store for Storage<T>
where
    T: Default + Clone + StorageModel,
{
    type Model = T;
    type Message = ();
    type Input = ();
    type Output = ();

    fn new(_link: StoreLink<Self>) -> Self {
        Self::new()
    }

    fn state(&mut self) -> &mut Rc<Self::Model> {
        &mut self.state
    }

    fn changed(&mut self) {
        self.save_state();
    }
}

impl<T> Clone for Storage<T>
where
    T: Default + Clone + StorageModel,
{
    fn clone(&self) -> Self {
        let mut new = Self::new();
        new.state = self.state.clone();
        new
    }
}

impl<T: StorageModel> StorageModel for Option<T> {
    fn key() -> &'static str {
        T::key()
    }

    fn area() -> Area {
        T::area()
    }
}

impl<T: StorageModel> StorageModel for Rc<T> {
    fn key() -> &'static str {
        T::key()
    }

    fn area() -> Area {
        T::area()
    }
}
