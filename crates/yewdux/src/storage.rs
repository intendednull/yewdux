//! Store persistence through session or local storage
//!
//! ```
//! use std::rc::Rc;
//!
//! use yewdux::{prelude::*, storage};
//!
//! use serde::{Deserialize, Serialize};
//!
//! struct StorageListener;
//! impl Listener for StorageListener {
//!     type Store = State;
//!
//!     fn on_change(&mut self, _cx: &Context, state: Rc<Self::Store>) {
//!         if let Err(err) = storage::save(state.as_ref(), storage::Area::Local) {
//!             println!("Error saving state to storage: {:?}", err);
//!         }
//!     }
//! }
//!
//! #[derive(Default, Clone, PartialEq, Eq, Deserialize, Serialize)]
//! struct State {
//!     count: u32,
//! }
//!
//! impl Store for State {
//!     fn new(cx: &yewdux::Context) -> Self {
//!         init_listener(StorageListener, cx);
//!
//!         storage::load(storage::Area::Local)
//!             .ok()
//!             .flatten()
//!             .unwrap_or_default()
//!     }
//!
//!     fn should_notify(&self, other: &Self) -> bool {
//!         self != other
//!     }
//! }
//! ```

use std::{any::type_name, rc::Rc};

use serde::{de::DeserializeOwned, Serialize};
use wasm_bindgen::{prelude::Closure, JsCast, JsValue};
use web_sys::{Event, Storage};

use crate::{dispatch::Dispatch, listener::Listener, store::Store, Context};

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

    fn on_change(&self, _cx: &Context, state: Rc<Self::Store>) {
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

/// Synchronize state across all tabs. **WARNING**: This provides no protection for multiple
/// calls. Doing so will result in repeated loading. Using the macro is advised.
pub fn init_tab_sync<S: Store + DeserializeOwned>(
    area: Area,
    cx: &Context,
) -> Result<(), StorageError> {
    let cx = cx.clone();
    let closure = Closure::wrap(Box::new(move |_: &Event| match load(area) {
        Ok(Some(state)) => {
            Dispatch::<S>::new(&cx).set(state);
        }
        Err(e) => {
            crate::log::error!("Unable to load state: {:?}", e);
        }
        _ => {}
    }) as Box<dyn FnMut(&Event)>);

    web_sys::window()
        .ok_or(StorageError::WindowNotFound)?
        .add_event_listener_with_callback("storage", closure.as_ref().unchecked_ref())
        .map_err(StorageError::WebSys)?;

    closure.forget();

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    use serde::Deserialize;

    #[derive(Deserialize)]
    struct TestStore;
    impl Store for TestStore {
        fn new(_cx: &Context) -> Self {
            Self
        }

        fn should_notify(&self, _old: &Self) -> bool {
            true
        }
    }

    #[test]
    fn tab_sync() {
        init_tab_sync::<TestStore>(Area::Local, &Context::global()).unwrap();
    }
}
