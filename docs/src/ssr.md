# SSR Support

The global context is thread-local, meaning it is shared by everything in the same thread. In a web
environment (strictly single-threaded), this effectively means app-wide shared state. However during
SSR time, this isn't always true. In fact, **using the global context during SSR time can be
unsafe**. For example if user details are saved in global context during SSR time of one session,
they could accidentally *leak* into every other active session in that thread.

To avoid accidental leakage, use Yewdux's dedicated context provider `YewduxRoot`:

```rust
use yew::prelude::*;
use yewdux::prelude::*;

#[derive(Default, Clone, PartialEq, Eq, Store)]
struct State {
    count: u32,
}

#[function_component]
fn Counter() -> Html {
    let (state, dispatch) = use_store::<State>();
    let onclick = dispatch.reduce_mut_callback(|state| state.count += 1);
    html! {
        <>
        <p>{ state.count }</p>
        <button {onclick}>{"+1"}</button>
        </>
    }
}

#[function_component]
fn App() -> Html {
    html! {
        <YewduxRoot>
            <Counter />
        </YewduxRoot>
    }
}
```

Hooks will detect the context provided by YewduxRoot. If no root is provided, the global context is
used.

For struct component support, refer to the [higher order components
pattern](https://yew.rs/docs/advanced-topics/struct-components/hoc).

