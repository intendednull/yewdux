# Yewdux

Simple state management for [Yew](https://yew.rs/docs/en/) applications.

This is the development branch. Latest stable release may be found
[here](https://github.com/intendednull/yewdux/tree/0.7.0).

# Setup

Add Yewdux to your project's `Cargo.toml`:

```toml
[dependencies]
yew = { git = "https://github.com/yewstack/yew.git", features = ["csr"] }
yewdux = { git = "https://github.com/intendednull/yewdux.git" }
```

# Example

```rust
use yew::prelude::*;
use yewdux::prelude::*;

#[derive(Default, Clone, Store)]
struct Counter {
    count: u32,
}

#[function_component]
fn App() -> Html {
    let (counter, dispatch) = use_store::<Counter>();
    let onclick = dispatch.reduce_callback(|counter| counter.count += 1);

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

# Usage

First, you'll need to implement `Store` for your state:

```rust
#[derive(Default, Clone, Store)]
struct Counter {
    count: u32,
}
```

`Clone` is required for all `Store`s, however `Default` is only needed for the macro. You can just
as well define it manually.

```rust
#[derive(Clone)]
struct Counter {
    count: u32,
}

impl Store for Counter {
    fn new() {
        Self {
            count: Default::default(),
        }
    }
}
```

Now simply create a dispatch.

```rust
let dispatch = Dispatch::<Counter>::new();
```

## Mutating shared state

`Dispatch` provides many options for mutating state.

`set` will set shared state to a given value.

```rust
dispatch.set(Counter { count: 0 });
```

`set_callback` generates a callback that does the same.

```rust
let onclick = dispatch.set_callback(|_| Counter { count: 0 });
html! {
    <button {onclick}>{"Reset counter"}</button>
}
```

`reduce` lets you mutate with a function.

```rust
dispatch.reduce(|counter| counter.count += 1);
```

`reduce_callback`, as you might expect, generates a callback that does the same.

```rust
let onclick = dispatch.reduce_callback(|counter| counter.count += 1);
html! {
    <button {onclick}>{"Increment (+1)"}</button>
}
```

`reduce_callback_with` is similar to `reduce_callback`, but also includes the fired event.

```rust
let onchange = dispatch.reduce_callback_with(|counter, e: Event| {
    let input = e.target_unchecked_into::<HtmlInputElement>();

    if let Ok(val) = input.value().parse() {
        counter.count = val;
    }
});

html! {
    <input placeholder="Set counter" {onchange} />
}

```

### Predictable mutation

Yewdux supports predictable mutation. Simply define your message and apply it.

```rust
struct Msg {
    AddOne,
}

impl Message<Counter> for Msg {
    fn apply(&self, counter: &mut Counter) {
        match self {
            Msg::AddOne => counter.count += 1,
        }
    }
}
```

`apply` executes immediately.

```rust
dispatch.apply(Msg::AddOne);
```

`apply_callback` does it (you guessed it) from a callback.

```rust
let onclick = dispatch.apply_callback(|_| Msg::AddOne);
html! {
    <button {onclick}>{"Increment (+1)"}</button>
}
```

## Reading shared state

`get` provides the current state (with a minor lookup cost).

```rust
let counter = dispatch.get();
```

However most components will also need to know when state changes so they can re-render. This can be
done by subscribing to changes.

`use_store` automatically subscribes, meaning the component will re-render every time `counter` changes (no additional setup required).

```rust
let (counter, dispatch) = use_store::<Counter>();
```

You may also subscribe manually, as shown below. At the cost of boilerplate, doing it this way
allows finer control over when exactly you'd like to re-render.

```rust
struct MyComponent {
    dispatch: Dispatch<Counter>,
    counter: std::rc::Rc<Counter>,

}

enum Msg {
    UpdateCounter(Rc<Counter>),
}

impl Component for MyComponent {
    type Properties = (); 
    type Message = Msg;

    fn create(ctx: &Context<Self>) -> Self {
        // The callback for receiving updates to state.
        let callback = ctx.link().callback(Msg::UpdateCounter);
        // Subscribe to changes in state. New state is received in `update`. Be sure to save this,
        // dropping it will unsubscribe.
        let dispatch = Dispatch::<Counter>::subscribe(callback);
        Self {
            // Get the current state.
            counter: dispatch.get(),
            dispatch,
        }
    }

    fn update(&self, ctx: &Context<Self>, msg: Msg) -> bool {
        match msg {
            // Receive new state.
            Msg::UpdateCounter(counter) => {
                self.counter = counter;

                // Only re-render this component if count is greater that 0 (for example).
                if self.counter.count > 0 {
                    true
                } else {
                    false
                }
            }
        }
    }

    ...
}
```

*Because `Dispatch::get` comes with a minor lookup cost, it's marginally more efficient to use the
value given to you by the subscription.*

# Additional examples

Complete working examples can be found
[here](https://github.com/intendednull/yewdux/tree/master/examples).

To run an example you'll need to install [trunk](https://github.com/thedodd/trunk), then run
(replacing [example] with your desired example name):

    trunk serve examples/[example]/index.html --open

