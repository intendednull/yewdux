# Example

```rust
use yew::prelude::*;
use yewdux::prelude::*;

#[derive(Default, Clone, PartialEq, Eq, Store)]
struct Counter {
    count: u32,
}

#[function_component]
fn App() -> Html {
    let (counter, dispatch) = use_store::<Counter>();
    let onclick = dispatch.reduce_mut_callback(|counter| counter.count += 1);

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
```

## Additional examples

Complete working examples can be found in the
[examples](https://github.com/intendednull/yewdux/tree/master/examples) folder of github.

To run an example you'll need to install [trunk](https://github.com/thedodd/trunk), then run the
following command (replacing [example] with your desired example name):

    trunk serve examples/[example]/index.html --open
