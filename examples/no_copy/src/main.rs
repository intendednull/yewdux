use yew::prelude::*;
use yewdux::{mrc::Mrc, prelude::*};

// Notice we don't implement Clone or PartialEq.
#[derive(Default)]
struct MyLargeData(u32);

#[derive(Default, Clone, PartialEq, Store)]
struct State {
    // Your expensive-clone field here.
    data: Mrc<MyLargeData>,
}

#[function_component]
fn App() -> Html {
    let (state, dispatch) = use_store::<State>();
    let onclick = dispatch.reduce_mut_callback(|state| {
        let mut data = state.data.borrow_mut();

        data.0 += 1;
    });

    html! {
        <>
        <p>{state.data.borrow().0}</p>
        <button {onclick}>{"+1"}</button>
        </>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
