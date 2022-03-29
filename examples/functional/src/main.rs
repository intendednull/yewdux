use yew::{functional::*, prelude::*};
use yewdux::prelude::*;
use yewdux_functional::*;

#[derive(Default, Clone)]
struct State {
    count: u32,
}

impl Store for State {
    type Message = ();

    fn new() -> Self {
        Default::default()
    }
}

#[function_component(App)]
fn app() -> Html {
    let (state, dispatch) = use_store::<State>();
    let onclick = dispatch.reduce_callback(|s| s.count += 1);

    html! {
        <>
        <p>{ state.count }</p>
        <button onclick={onclick}>{"+1"}</button>
        </>
    }
}

pub fn main() {
    yew::Renderer::<App>::new().render();
}
