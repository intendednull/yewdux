use std::rc::Rc;

use yew::prelude::*;
use yewdux::prelude::*;

#[derive(Default, Clone, PartialEq, Store)]
struct State {
    count: u32,
}

enum Msg {
    AddOne,
}

impl Reducer<State> for Msg {
    fn apply(&self, state: Rc<State>) -> Rc<State> {
        match self {
            Msg::AddOne => State {
                count: state.count + 1,
            }
            .into(),
        }
    }
}

#[function_component]
fn App() -> Html {
    let (state, dispatch) = use_store::<State>();
    let onclick = dispatch.apply_callback(|_| Msg::AddOne);

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
