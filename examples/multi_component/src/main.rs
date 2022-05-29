use yew::prelude::*;
use yewdux::prelude::*;

#[derive(Default, Clone, PartialEq, Eq, Store)]
struct State {
    count: u32,
}

#[function_component]
fn ViewCount() -> Html {
    let (state, _) = use_store::<State>();

    html! {
        <p>{"Count is: "}{ state.count }</p>
    }
}

#[function_component]
fn IncrementCount() -> Html {
    let (_, dispatch) = use_store::<State>();
    let onclick = dispatch.reduce_mut_callback(|state| state.count += 1);

    html! {
        <button {onclick}>{"+1"}</button>
    }
}

#[function_component]
fn App() -> Html {
    html! {
        <>
        <ViewCount />
        <IncrementCount />
        </>
    }
}

pub fn main() {
    yew::Renderer::<App>::new().render();
}
