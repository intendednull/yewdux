//! Store persistence through session or local storage
//!
//! ```
//! use yewdux::prelude::*;
//! use serde::{Serialize, Deserialize};
//!
//! #[derive(Clone, PartialEq, Serialize, Deserialize)]
//! struct Counter {
//!     count: u32,
//! }
//!
//! impl Store for Counter {
//!     fn new() -> Self {
//!         storage::load(storage::Area::Local)
//!             .expect("Unable to load state")
//!             .unwrap_or_default()
//!     }
//!
//!     fn changed(&mut self) {
//!         storage::save(self, storage::Area::Local).expect("Unable to save state");
//!     }
//! }
//! ```

use std::{any::type_name, rc::Rc};

use serde::{de::DeserializeOwned, Serialize};
use wasm_bindgen::JsValue;
use web_sys::Storage;

use crate::{listener::Listener, store::Store};

#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("Window not found")]
    WindowNotFound,
    #[error("Could not access {0:?} storage")]
    StorageAccess(Area),
    #[error("A web-sys error occurred")]
    WebSys(JsValue),
    #[error("A serde error occurred")]
    Serde(#[from] serde_json::Error),
}

#[derive(Debug, Clone, Copy)]
pub enum Area {
    Local,
    Session,
}

/// A [Listener] that will save state to browser storage whenever state has changed.
pub struct StorageListener<T> {
    area: Area,
    _marker: std::marker::PhantomData<T>,
}

impl<T> StorageListener<T> {
    pub fn new(area: Area) -> Self {
        Self {
            area,
            _marker: Default::default(),
        }
    }
}

impl<T> Listener for StorageListener<T>
where
    T: Store + Serialize,
{
    type Store = T;

    fn on_change(&mut self, state: Rc<Self::Store>) {
        if let Err(err) = save(state.as_ref(), self.area) {
            crate::log::error!("Error saving state to storage: {:?}", err);
        }
    }
}

fn get_storage(area: Area) -> Result<Storage, StorageError> {
    let window = web_sys::window().ok_or(StorageError::WindowNotFound)?;
    let storage = match area {
        Area::Local => window.local_storage(),
        Area::Session => window.session_storage(),
    };

    storage
        .map_err(StorageError::WebSys)?
        .ok_or(StorageError::StorageAccess(area))
}

/// Save state to session or local storage.
pub fn save<T: Serialize>(state: &T, area: Area) -> Result<(), StorageError> {
    let storage = get_storage(area)?;

    let value = &serde_json::to_string(state).map_err(StorageError::Serde)?;
    storage
        .set(type_name::<T>(), value)
        .map_err(StorageError::WebSys)?;

    Ok(())
}

/// Load state from session or local storage.
pub fn load<T: DeserializeOwned>(area: Area) -> Result<Option<T>, StorageError> {
    let storage = get_storage(area)?;

    let value = storage
        .get(type_name::<T>())
        .map_err(StorageError::WebSys)?;

    match value {
        Some(value) => {
            let state = serde_json::from_str(&value).map_err(StorageError::Serde)?;

            Ok(Some(state))
        }
        None => Ok(None),
    }
}
