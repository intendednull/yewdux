use std::rc::Rc;

use yew::prelude::*;
use yewdux::prelude::*;

#[derive(Default, Clone, PartialEq, Eq, Store)]
struct Count {
    count: u32,
}

#[derive(Default, Clone, PartialEq, Eq)]
struct CountIsEven {
    status: bool,
}

impl DerivedFromMut<Count> for CountIsEven {
    fn on_change(&mut self, state: Rc<Count>) {
        self.status = state.count % 2 == 0;
    }
}

impl Store for CountIsEven {
    fn new(cx: &yewdux::Context) -> Self {
        // Don't forget to register this state as derived from `Count`.
        cx.derived_from_mut::<Count, Self>();

        let status = cx.get::<Count>().count % 2 == 0;

        Self { status }
    }

    fn should_notify(&self, old: &Self) -> bool {
        self != old
    }
}

#[function_component]
fn App() -> Html {
    let (state, dispatch) = use_store::<Count>();
    let is_even = use_store_value::<CountIsEven>();
    let onclick = dispatch.reduce_mut_callback(|state| state.count += 1);

    html! {
        <>
        <p>{ state.count }</p>
        <p>{ is_even.status }</p>
        <button {onclick}>{"+1"}</button>
        </>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
