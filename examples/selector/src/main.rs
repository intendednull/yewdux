use yew::prelude::*;
use yewdux::prelude::*;

#[derive(Default, Clone, PartialEq, Store)]
struct State {
    count: u32,
}

#[function_component]
fn App() -> Html {
    let count = use_selector(|state: &State| state.count);
    let dispatch = Dispatch::<State>::new();
    let onclick = dispatch.reduce_mut_callback(|state| state.count += 1);

    html! {
        <>
        <p>{ count }</p>
        <button {onclick}>{"+1"}</button>
        </>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
