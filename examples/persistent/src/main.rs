use yew::prelude::*;
use yewdux::prelude::*;

use serde::{Deserialize, Serialize};

#[derive(Default, Clone, PartialEq, Deserialize, Serialize)]
struct State {
    count: u32,
}

impl Store for State {
    fn new() -> Self {
        yewdux::storage::load(storage::Area::Local)
            .expect("Unable to load state")
            .unwrap_or_default()
    }

    fn changed(&mut self) {
        yewdux::storage::save(self, storage::Area::Local).expect("Unable to save state");
    }
}

#[function_component]
fn App() -> Html {
    let (state, dispatch) = use_store::<State>();
    let onclick = dispatch.reduce_callback(|state| state.count += 1);

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
