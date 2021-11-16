use std::any::type_name;
use std::rc::Rc;

use serde::{Deserialize, Serialize};
use web_sys::Storage;

use super::{Store, StoreLink};

pub enum Area {
    Local,
    Session,
}

/// Allows state to be stored persistently in local or session storage.
pub trait Persistent: Serialize + for<'a> Deserialize<'a> {
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
pub struct PersistentStore<T> {
    state: Rc<T>,
    storage: Option<Storage>,
}

impl<T> PersistentStore<T>
where
    T: Persistent + Default,
{
    pub fn new() -> Self {
        let mut this: Self = Default::default();
        let window = web_sys::window().expect("no window available");
        this.storage = match T::area() {
            Area::Local => window.local_storage().ok().flatten(),
            Area::Session => window.session_storage().ok().flatten(),
        };
        this.load_state();
        this
    }

    pub fn load_state(&mut self) {
        let result = self.storage.as_mut().map(|s| s.get(T::key()));
        if let Some(Ok(Some(result))) = result {
            if let Ok(state) = serde_json::from_str(&result) {
                self.state = state;
            }
        }
    }

    pub fn save_state(&mut self) {
        if let Some(storage) = &mut self.storage {
            if let Ok(data) = serde_json::to_string(&self.state) {
                storage.set(T::key(), &data).ok();
            }
        }
    }
}

impl<T> Store for PersistentStore<T>
where
    T: Default + Clone + Persistent + 'static,
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

impl<T> Clone for PersistentStore<T>
where
    T: Default + Clone + Persistent,
{
    fn clone(&self) -> Self {
        let mut new = Self::new();
        new.state = self.state.clone();
        new
    }
}

impl<T: Persistent> Persistent for Option<T> {
    fn key() -> &'static str {
        T::key()
    }

    fn area() -> Area {
        T::area()
    }
}

impl<T: Persistent> Persistent for Rc<T> {
    fn key() -> &'static str {
        T::key()
    }

    fn area() -> Area {
        T::area()
    }
}
