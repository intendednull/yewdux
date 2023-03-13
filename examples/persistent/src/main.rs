use std::rc::Rc;

use yew::prelude::*;
use yewdux::{prelude::*, log::{log, Level}};

use serde::{Deserialize, Serialize};

#[derive(Default, Clone, PartialEq, Eq, Deserialize, Serialize, Store)]
#[store(storage = "local")]
struct State {
    count: u32,
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
    wasm_logger::init(wasm_logger::Config::default());
    yew::Renderer::<App>::new().render();
}


struct LogListener;
impl Listener for LogListener {
    type Store = State;

    fn on_change(&mut self, _state: Rc<Self::Store>) {
        log!(Level::Info, "State has changed 123");
    }
}