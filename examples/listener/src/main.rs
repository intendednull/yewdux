use std::rc::Rc;

use yew::prelude::*;
#[cfg(target_arch = "wasm32")]
use yewdux::storage;
use yewdux::{
    log::{log, Level},
    prelude::*,
    Context,
};

use serde::{Deserialize, Serialize};

struct LogListener;
impl Listener for LogListener {
    type Store = State;

    fn on_change(&self, _cx: &Context, state: Rc<Self::Store>) {
        log!(Level::Info, "Count changed to {}", state.count);
    }
}

struct StorageListener;
impl Listener for StorageListener {
    type Store = State;

    fn on_change(&self, _cx: &yewdux::Context, state: Rc<Self::Store>) {
        #[cfg(target_arch = "wasm32")]
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
    #[cfg(not(target_arch = "wasm32"))]
    fn new(_cx: &yewdux::Context) -> Self {
        Default::default()
    }

    #[cfg(target_arch = "wasm32")]
    fn new(cx: &yewdux::Context) -> Self {
        init_listener(|| StorageListener, cx);
        init_listener(|| LogListener, cx);

        storage::load(storage::Area::Local)
            .ok()
            .flatten()
            .unwrap_or_default()
    }

    fn should_notify(&self, other: &Self) -> bool {
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
    wasm_logger::init(wasm_logger::Config::default());
    yew::Renderer::<App>::new().render();
}
