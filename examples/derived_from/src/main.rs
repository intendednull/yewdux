use std::rc::Rc;

use yew::prelude::*;
use yewdux::prelude::*;

#[derive(Default, Clone, PartialEq, Eq, Store)]
struct Count {
    count: u32,
}

// Example using DerivedFromMut - mutable version
#[derive(Default, Clone, PartialEq, Eq, Store)]
#[store(derived_from_mut(Count))]
struct CountIsEven {
    status: bool,
}

impl DerivedFromMut<Count> for CountIsEven {
    fn on_change(&mut self, state: Rc<Count>) {
        self.status = state.count.is_multiple_of(2);
    }
}

// Example using DerivedFrom - immutable version
#[derive(Default, Clone, PartialEq, Eq, Store)]
#[store(derived_from(Count))]
struct CountMultiplied {
    value: u32,
}

impl DerivedFrom<Count> for CountMultiplied {
    fn on_change(&self, state: Rc<Count>) -> Self {
        Self {
            value: state.count * 10,
        }
    }
}

#[function_component]
fn App() -> Html {
    let (state, dispatch) = use_store::<Count>();
    let is_even = use_store_value::<CountIsEven>();
    let multiplied = use_store_value::<CountMultiplied>();
    let onclick = dispatch.reduce_mut_callback(|state| state.count += 1);

    html! {
        <>
        <p>{"Count: "}{ state.count }</p>
        <p>{"Is Even: "}{ is_even.status.to_string() }</p>
        <p>{"Multiplied by 10: "}{ multiplied.value }</p>
        <button {onclick}>{"+1"}</button>
        </>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
