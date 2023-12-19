# Quickstart example

Below you'll find a simple counter example, demonstrating how to read and write to shared state.

```rust
# extern crate yewdux;
# extern crate yew;
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
```

## Additional examples

Complete working examples can be found in the
[examples](https://github.com/intendednull/yewdux/tree/master/examples) folder of github.

To run an example you'll need to install [trunk](https://github.com/thedodd/trunk) (a rust wasm
bundler), then run the following command (replacing [example] with your desired example name):
```bash
    trunk serve examples/[example]/index.html --open
```
