# Yewdux

Simple state management for [Yew](https://yew.rs) applications.

This is the development branch. Latest stable release may be found
[here](https://github.com/intendednull/yewdux/tree/0.7.0).

## Alternatives

- [Bounce](https://github.com/futursolo/bounce) - The uncomplicated Yew State management library

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
[examples](https://github.com/intendednull/yewdux/tree/master/examples) folder of this project.

To run an example you'll need to install [trunk](https://github.com/thedodd/trunk), then run the
following command (replacing [example] with your desired example name):

    trunk serve examples/[example]/index.html --open

# Usage

## Store

`Store` represents state that is shared application-wide. It is initialized the first time it is
accessed, and lives for application lifetime. 

Implement `Store` for your state using the macro.

```rust
#[derive(Default, PartialEq, Store)]
struct Counter {
    count: u32,
}
```

Or do it manually. 

```rust
#[derive(PartialEq)]
struct Counter {
    count: u32,
}

impl Store for Counter {
    fn new() {
        Self {
            count: Default::default(),
        }
    }

    fn changed(&self, other: &Self) -> bool {
        // When this returns true, all components are notified and consequently re-render.
        //
        // We're using `PartialEq` here to keep it simple, but it's possible to use any custom
        // logic that you'd want.
        self != other
    }
}
```

*Note: implementing `Store` doesn't require any additional traits, however `Default` and
`PartialEq` are required for the macro.*

## Dispatch

`Dispatch` provides an interface to your `Store`. To create one you need only provide the store
type. 

```rust
let dispatch = Dispatch::<Counter>::new();
```

### Changing state

`Dispatch` provides many options for changing state.

`set` will assign to a given value.

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

`reduce` lets you change state with a function.

```rust
dispatch.reduce(|counter| Counter { count: counter.count + 1});
```

`reduce_callback`, as you might expect, generates a callback that does the same.

```rust
let onclick = dispatch.reduce_callback(|counter| Counter { count: counter.count + 1});
html! {
    <button {onclick}>{"Increment (+1)"}</button>
}
```

`reduce_callback_with` is similar to `reduce_callback`, but also includes the fired event.

```rust
let onchange = dispatch.reduce_callback_with(|counter, e: Event| {
    let input = e.target_unchecked_into::<HtmlInputElement>();

    if let Ok(count) = input.value().parse() {
        Counter { count }.into()
    } else {
        counter
    }
});

html! {
    <input placeholder="Set counter" {onchange} />
}
```

#### Succinct mutations

There are `_mut` variants to every reducer function. This way has less boilerplate, and requires
your `Store` to implement `Clone`.

`reduce_mut`

```rust
dispatch.reduce_mut(|counter| counter.count += 1);
```

`reduce_mut_callback`

```rust
let onclick = dispatch.reduce_mut_callback(|counter| counter.count += 1);
html! {
    <button {onclick}>{"Increment (+1)"}</button>
}
```

`reduce_mut_callback_with`

```rust
let onchange = dispatch.reduce_mut_callback_with(|counter, e: Event| {
    let input = e.target_unchecked_into::<HtmlInputElement>();

    if let Ok(val) = input.value().parse() {
        counter.count = val;
    }
});

html! {
    <input placeholder="Set counter" {onchange} />
}
```

#### Predictable mutation

Yewdux supports predictable mutation. Simply define your message and apply it.

```rust
struct Msg {
    AddOne,
}

impl Reducer<Counter> for Msg {
    fn apply(&self, counter: Rc<Counter>) -> Rc<Counter> {
        match self {
            Msg::AddOne => Counter { count: counter.count + 1 },
        }
    }
}
```

`apply` executes immediately.

```rust
dispatch.apply(Msg::AddOne);
```

`apply_callback` generates (you guessed it) a callback.

```rust
let onclick = dispatch.apply_callback(|_| Msg::AddOne);
html! {
    <button {onclick}>{"Increment (+1)"}</button>
}
```

`Rc::make_mut` is handy if you prefer mutation:

```rust
impl Reducer<Counter> for Msg {
    fn apply(&self, mut counter: Rc<Counter>) -> Rc<Counter> {
        let state = Rc::make_mut(&mut counter);

        match self {
            Msg::AddOne => state.count += 1,
        };

        counter
    }
}
```

### Subscribing to changes

Components need to know when to re-render for changes. To do this they can subscribe to a store.

Functional hooks like `use_store` will subscribe automatically.

```rust
// `counter` is automatically updated when global state changes.
let (counter, dispatch) = use_store::<Counter>();
```

You may also subscribe manually, as shown below. At the cost of boilerplate, doing it this way
allows much finer control.

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

It is also possible to retrieve the current state of a store without subscribing to changes. This is
useful when you don't really care when/if state has changed, just what the current value is.

`Dispatch::get` will lookup the current value immediately:

```rust
let state = dispatch.get();
```

#### Selectors

Sometimes a component will only care about a particular part of state, and only needs to re-render
when that part changes. For this we have the `use_selector` hook.

Consider the following example.

```rust
#[derive(Default, Clone, PartialEq, Store)]
struct Counter {
    count_1: u32,
    count_2: u32,
}

#[function_component]
fn CountOne() -> Html {
    // Only re-render when `Counter::count_1` changes.
    let count = use_selector(|state: &Counter| state.count_1);

    html! {
        <p>{ count }</p>
    }
}

#[function_component]
fn CountTwo() -> Html {
    // Only re-render when `Counter::count_2` changes.
    let count = use_selector(|state: &Counter| state.count_2);

    html! {
        <p>{ count }</p>
    }
}


#[function_component]
fn App() -> Html {
    let dispatch = Dispatch::<Counter>::new();
    let incr_one = dispatch.reduce_mut_callback(|counter| counter.count_1 += 1);
    let incr_two = dispatch.reduce_mut_callback(|counter| counter.count_2 += 1);

    html! {
        <>
        <CountOne />
        <button onclick={incr_one}>{"Incr One"}</button>
        <CountTwo />
        <button onclick={incr_two}>{"Incr Two"}</button>
        </>
    }
}
```

Here we have two components accessing the same store, but each only cares about one field of that
store. They only re-render when the field they have selected has changed, and won't needlessly
re-render if it hasn't.

# Persistence

Yewdux provides a macro to easily persist your state in either local or session storage.

```rust
use yewdux::prelude::*;
use serde::{Serialize, Deserialize};

#[derive(Default, PartialEq, Serialize, Deserialize, Store)]
#[store(storage = "local")] // can also be "session"
struct Counter {
    count: u32,
}
```

You can also implement it yourself.

```rust
use yewdux::{prelude::*, storage};

impl Store for Counter {
    fn new() -> Self {
        init_listener(storage::StorageListener::<Self>::new(storage::Area::Local));

        storage::load(storage::Area::Local)
            .expect("Unable to load state")
            .unwrap_or_default()
    }

    fn changed(&self, other: &Self) -> bool {
        self != other
    }
}
```

