#![cfg(target_arch = "wasm32")]

use yew::prelude::*;
use yewdux::prelude::*;

#[derive(Default, Clone, PartialEq, Eq, Store)]
struct State {
    count: u32,
}

#[function_component]
fn Counter() -> Html {
    let global_state = Dispatch::<State>::global().get();
    let (state, dispatch) = use_store::<State>();
    let onclick = dispatch.reduce_mut_callback(|state| state.count += 1);
    html! {
        <>
        <p>{ global_state.count }</p>
        <p>{ state.count }</p>
        <button {onclick}>{"+1"}</button>
        </>
    }
}

#[function_component]
fn App() -> Html {
    html! {
        <YewduxRoot>
            <Counter />
        </YewduxRoot>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
