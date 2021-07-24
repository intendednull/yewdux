use yew::{functional::*, prelude::*};
use yewdux::prelude::*;
use yewdux_functional::*;

#[derive(Default, Clone)]
struct State {
    count: u32,
}

#[function_component(App)]
fn app() -> Html {
    let store = use_store::<BasicStore<State>>();
    let count = store.state().map(|s| s.count).unwrap_or_default();
    let onclick = store.dispatch().reduce_callback(|s| s.count += 1);

    html! {
        <>
        <p>{ count }</p>
        <button onclick={onclick}>{"+1"}</button>
        </>
    }
}

pub fn main() {
    yew::start_app::<App>();
}
