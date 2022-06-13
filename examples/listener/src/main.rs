use std::rc::Rc;

use yew::prelude::*;
use yewdux::{prelude::*, storage};

use serde::{Deserialize, Serialize};

struct StorageListener;
impl Listener for StorageListener {
    type Store = State;

    fn on_change(&mut self, state: Rc<Self::Store>) {
        if let Err(err) = storage::save(state.as_ref(), storage::Area::Local) {
            println!("Error saving state to storage: {:?}", err);
        }
    }
}

#[derive(Default, Clone, PartialEq, Eq, Deserialize, Serialize)]
struct State {
    count: u32,
}

impl Store for State {
    fn new() -> Self {
        init_listener(StorageListener);

        storage::load(storage::Area::Local)
            .ok()
            .flatten()
            .unwrap_or_default()
    }

    fn changed(&self, other: &Self) -> bool {
        self != other
    }
}

#[function_component]
fn App() -> Html {
    let (state, dispatch) = use_store::<State>();
    let onclick = dispatch.reduce_mut_callback(|state| state.count += 1);

    html! {
        <>
        <p>{ state.count }</p>
        <button {onclick}>{"+1"}</button>
        </>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
