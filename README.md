# Yewdux

Ergonomic state management for [Yew](https://yew.rs) applications.

See the [book](https://intendednull.github.io/yewdux/) for more details.

## Example

```rust
use yew::prelude::*;
use yewdux::prelude::*;

#[derive(Default, Clone, PartialEq, Store)]
struct State {
    count: u32,
}

#[function_component]
fn ViewCount() -> Html {
    let (state, _) = use_store::<State>();
    html!(state.count)
}

#[function_component]
fn IncrementCount() -> Html {
    let (_, dispatch) = use_store::<State>();
    let onclick = dispatch.reduce_mut_callback(|counter| counter.count += 1);

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

fn main() {
    yew::Renderer::<App>::new().render();
}
```
