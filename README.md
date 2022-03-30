# Yewdux

Simple state management for [Yew](https://yew.rs/docs/en/) applications.

This is the development branch. Latest stable release may be found
[here](https://github.com/intendednull/yewdux/tree/0.7.0).

# Setup

Add Yewdux to your project's `Cargo.toml`:

```toml
[dependencies]
yewdux = { git = "https://github.com/intendednull/yewdux.git" }
```

# Example

```rust

use yew::prelude::*;
use yewdux::prelude::*;

#[derive(Default, Clone, Store)]
struct State {
    count: u32,
}

#[function_component]
fn App() -> Html {
    let (state, dispatch) = use_store::<State>();
    let onclick = dispatch.reduce_callback(|state| state.count += 1);

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
```

# Usage

`Dispatch` is the primary interface for Yewdux. It allows shared mutable access to global
application state.

## Writing to shared state

If we look at the example above, we can see one method of getting a dispatch. This way also
automatically subscribes to changes to state, which is important to know when to re-render the
component.

You can also create a dispatch (without a subscription) using `Dispatch::new`.

```rust
let dispatch = Dispatch::<State>::new();
```

### Dispatch methods

`set` will set shared state to a given value.

```rust
dispatch.set(State { count: 1 });
```

`reduce` lets you mutate with a function. Here we immediately increment count by 1.

```rust
dispatch.reduce(|state| state.count += 1);
```

We may also create callbacks that do the same. This button sends the message every time it is
clicked.

```rust
let onclick = dispatch.reduce_callback(|state| state.count += 1);
html! {
    <button {onclick}>{"+1"}</button>
}
```

If the callback parameter is needed, it may be accessed using the \*_with variant. The following
creates a new callback, then immediately calls it, incrementing count by 5.

```rust
let cb = dispatch.reduce_callback_with(|state, incr: usize| state.count += incr);
cb.emit(5);
```

#### Predictable mutations

Yewux supports predictable mutation. Simply define your message and apply it.

```rust
struct Msg {
    AddOne,
}

impl Message<State> for Msg {
    fn apply(&self, state: &mut State) {
        match self {
            Msg::AddOne => state.count += 1,
        }
    }
}

// Send message immediately
dispatch.apply(Msg::AddOne);

// Send message from a callback
dispatch.apply_callback(|_| Msg::AddOne);
```

## Reading shared state

`get` provides the current value of shared state.

```rust
let state = dispatch.get();
```

However this **does not** notify you when state changes. If we want to re-render when state
changes, we need to subscribe first.

The [example](#example) shows `use_store`, which automatically subscribes to state, and triggers a
re-render when state changes.

You may also subscribe manually, as shown below.

```rust
use std::rc::Rc;

use yew::prelude::*;
use yewdux::prelude::*;

struct MyComponent {
    dispatch: Dispatch<State>,
    state: Rc<MyState>,
}

enum Msg {
    State(Rc<State>),
}

impl Component for MyComponent {
    type Properties = (); 
    type Message = Msg;

    fn create(ctx: &Context<Self>) -> Self {
        // Subscribe to changes in state. New state is received in `update`.
        let dispatch = Dispatch::<State>::subscribe(ctx.link().callback(Msg::State));
        Self {
            state: dispatch.get(),
            dispatch,
        }
    }

    fn update(&self, ctx: &Context<Self>, msg: Msg) -> bool {
        match msg {
            // Receive new state.
            Msg::State(state) => {
                self.state = Some(state);
                true
            }
        }
    }

    ...
}
```

# Additional Examples

Complete working examples can be found
[here](https://github.com/intendednull/yewdux/tree/master/examples).

To run an example you'll need to install [trunk](https://github.com/thedodd/trunk), then run
(replacing [example] with your desired example name):

    trunk serve examples/[example]/index.html --open

