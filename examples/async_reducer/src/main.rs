use std::rc::Rc;

use yew::prelude::*;
use yewdux::{async_trait, prelude::*};

#[derive(Default, Clone, PartialEq, Eq, Store)]
struct Counter {
    count: u32,
}

enum Msg {
    AddOne,
}

#[async_trait(?Send)]
impl AsyncReducer<Counter> for Msg {
    async fn apply(self, mut counter: Rc<Counter>) -> Rc<Counter> {
        let state = Rc::make_mut(&mut counter);

        match self {
            Msg::AddOne => state.count += 1,
        };

        counter
    }
}

#[function_component]
fn App() -> Html {
    let (counter, dispatch) = use_store::<Counter>();
    let onclick = dispatch.apply_future_callback(|_| Msg::AddOne);

    html! {
        <>
        <p>{ counter.count }</p>
        <button {onclick}>{"+1"}</button>
        </>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
