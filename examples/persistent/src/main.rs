use yew::prelude::*;
use yewdux::prelude::*;

use serde::{Deserialize, Serialize};

#[derive(Default, Clone, PartialEq, Deserialize, Serialize, Store)]
#[store(storage = "local")]
struct State {
    count: u32,
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
