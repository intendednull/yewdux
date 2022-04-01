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

#[derive(Default, Clone, PartialEq, Store)]
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
#[derive(Default, Clone, PartialEq, Store)]
struct Counter {
    count: u32,
}
```

`Clone` and `PartialEq` are required for all `Store`s, however `Default` is only needed for the macro. You can just
as well define it manually.

```rust
#[derive(Clone, PartialEq)]
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
    fn apply(&mut self, counter: &mut Counter) {
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
use std::rc::Rc;

use yew::prelude::*;
use yewdux::prelude::*;

struct MyComponent {
    dispatch: Dispatch<Counter>,
    counter: Rc<Counter>,

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

    fn update(&mut self, ctx: &Context<Self>, msg: Msg) -> bool {
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

# Persistence

Yewdux provides an easy way to persist your state in either local or session storage.

```rust
use yewdux::prelude::*;
use serde::{Serialize, Deserialize};

#[derive(Clone, PartialEq, Serialize, Deserialize)]
struct Counter {
    count: u32,
}

impl Store for Counter {
    fn new() -> Self {
        storage::load(storage::Area::Local)
            .expect("Unable to load state")
            .unwrap_or_default()
    }

    fn changed(&mut self) {
        storage::save(self, storage::Area::Local).expect("Unable to save state");
    }
}
```

# Additional examples

Complete working examples can be found
[here](https://github.com/intendednull/yewdux/tree/master/examples).

To run an example you'll need to install [trunk](https://github.com/thedodd/trunk), then run
(replacing [example] with your desired example name):

    trunk serve examples/[example]/index.html --open


# Design decisions

For the curious, here's my rationale for Yewdux design decisions.

## Why does my state need to implement Clone and PartialEq?

Yewdux uses a CoW (clone on write) state management strategy. This means state is cloned once every
time a mutation occurs. You might be thinking: but wait, isn't that slow? Well, it's complicated.
Compared to interior mutability, the actual mutation is slower because of the extra cloning step.
However we must also consider how many components will consequentially re-render.

When state changes, each subscriber is notified and a re-render is triggered. We have to assume the
number of subscribers could be in the 10s or 100s (possibly 1000s), meaning a lot of potential
re-renders per mutation. If we're lucky, these components are simply updating their relevant view,
however worst case scenario they could be doing a lot of extra work, with long rendering times.
Compared to this, state cloning cost is insignificant (most of the time).

What's best way to reduce rendering time? Simple. Don't render! Yewdux checks if state changed after
a mutable borrow, and only notify subscribers when it has.

Here's real-world example where this is useful:

```rust
let onchange = dispatch.reduce_callback_with(|counter: &mut Counter, e: Event| {
    let input = e.target_unchecked_into::<HtmlInputElement>();

    if let Ok(val) = input.value().parse() {
        counter.count = val;
    }
});
```

This closure executes after we've already borrowed `counter` as mutable, however it only actually
mutates `counter` when it successfully parses the input. If we were using interior mutability there
would be no way to compare the past value with the present, because there would be no past. We would
have to notify subscribers even though it's possible nothing has changed. With Clone + PartialEq we
can compare the past and present to make smarter rendering decisions.

### But my state is large, and cloning every mutation isn't feasible!

While rare, there are cases where cloning cost will outweigh rendering cost. One example could be a
large array of non-trivial structs, with a single component in charge of rendering those items (one
re-render per change). For these cases I suggest selective interior mutability. By wrapping your
expensive field in `Rc<RefCell<T>>`, you'll avoid all the extra cost of Clone + PartialEq.

Yewdux does provide a simple (and optional) wrapper type to make this a little more ergonomic `Mrc`:

```rust
use yew::prelude::*;
use yewdux::{prelude::*, util::Mrc};

// Notice we don't implement Clone or PartialEq.
#[derive(Default)]
struct MyLargeData(u32);

#[derive(Default, Clone, PartialEq, Store)]
struct State {
    // Your expensive-clone field here.
    data: Mrc<MyLargeData>,
}
```

Mutating would be done normally:

```rust
let onclick = dispatch.reduce_callback(|state| {
    let mut data = state.data.borrow_mut();

    data.0 += 1;
});
```


The full example may be found in [examples/no_copy](./examples/no_copy/src/main.rs).


## I've read dispatch.rs and I know your secrets! 

Shh! Keep your voice down. Yes `Dispatch` is not required to interact with state. You could just as
well do `yewdux::dispatch::reduce::<Counter>(..)`. `Dispatch` is merely a ergonomic abstraction over
functional side-effects. Basically making callbacks a little nicer to work with.
