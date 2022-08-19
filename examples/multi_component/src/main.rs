use yew::prelude::*;
use yewdux::prelude::*;

#[derive(Default, Clone, PartialEq, Eq, Store)]
struct State {
    count: u32,
}

#[function_component(ViewCount)]
fn view_count() -> Html {
    let (state, _) = use_store::<State>();

    html! {
        <p>{"Count is: "}{ state.count }</p>
    }
}

#[function_component(IncrementCount)]
fn increment_count() -> Html {
    let (_, dispatch) = use_store::<State>();
    let onclick = dispatch.reduce_mut_callback(|state| state.count += 1);

    html! {
        <button {onclick}>{"+1"}</button>
    }
}

#[function_component(App)]
fn app() -> Html {
    html! {
        <>
        <ViewCount />
        <IncrementCount />
        </>
    }
}

pub fn main() {
    yew::start_app::<App>();
}
