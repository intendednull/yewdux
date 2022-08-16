# Yewdux

Simple state management for [Yew](https://yew.rs) applications.

This crate tries to provide a dead-simple, zero-cost approach to global state management. It does
*not* try to provide any additional patterns or features which aren't directly related to accessing
or manipulating shared state.

Some key features include:
- Zero-clone - user has complete control over how state is changed. Yewdux will never deep-copy your
    state unless explicitly told to. CoW behavior is provided by the `Dispatch::reduce_mut*`
    variants (marked by a `Clone` trait requirement).
- Selective rendering - subscribers are only notified when state has changed, avoiding any
    unnecessary re-renders. Can be further optimized with `use_selector` hooks.
- Access from anywhere - users can create a dispatch to access a store from anywhere, they are not
    restricted to only inside components. This boasts greater flexibility over application flow and
    setup.
- Ergonomic interface - accessing a store is as simple as creating a dispatch with your desired
    store type. From this dispatch you can modify the store directly, create callbacks to trigger
    from events, or even execute in an async context.
- Minimal trait requirements - The only trait required for a type to be a store is the `Store` trait
    itself. While the `Store` macro does need `Default` and `PartialEq` to work, it is also very
    simple to implement `Store` yourself, no additional requirements necessary!
- Complete component support - Yewdux supports both struct components and functional components.
    Although functional is usually the most convenient option, the utility and flexibility of struct
    components cannot be denied.

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
struct User {
    first_name: String,
    last_name: String,
}

#[function_component]
fn DisplayFirst() -> Html {
    // This will only re-render when the first name has changed. It will **not** re-render if any
    // other field has changed.
    //
    // Note: we are cloning a string. Probably insignificant for this example, however
    // sometimes it may be beneficial to wrap fields that are expensive to clone in an `Rc`.
    let first_name = use_selector(|state: &User| state.first_name.clone());

    html! {
        <p>{ first_name }</p>
    }
}
```

For selectors that need to capture variables from their environment, be sure to provide them as
dependencies to `use_selector_with_deps`. Otherwise you may notice some odd behaior!

```rust
#[derive(Default, Clone, PartialEq, Store)]
struct Items {
    inner: HashMap<u32, String>,
}

#[derive(Clone, PartialEq, Properties)]
struct DisplayItemProps {
    item_id: u32,
}

#[function_component]
fn DisplayItem(props: &DisplayItemProps) -> Html {
    // For multiple dependencies, try using a tuple: (dep1, dep2, ..)
    let item = use_selector_with_deps(
        |state: &Items, item_id| state.inner.get(item_id).cloned(),
        props.item_id,
    );
    // Only render the item if it exists.
    let item = match item {
        Some(item) => item,
        None => return Default::default(),
    };

    html! {
        <p>{ item }</p>
    }
}
```

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

## Tab sync

Normally if your application is open in multiple tabs, the persistent storage is not updated in any
tab other than the current one. If you want a store to sync in all tabs, add `storage_tab_sync` to
the macro.

```rust
#[derive(Default, Clone, PartialEq, Eq, Deserialize, Serialize, Store)]
#[store(storage = "local", storage_tab_sync)]
struct State {
    count: u32,
}
```

# Future support

Because a `Dispatch` may be created and executed from anywhere, Yewdux has innate future support.
Just use it normally, no additonal setup is needed.

For stores that have async methods, dispatch provides some options for your convenience.

The following can be executed immediately, in an async context.

`reduce_future` for the pure approach.
```rust
dispatch
    .reduce_future(|state| async move {
        let mut state = state.as_ref().clone();
        state.update_user().await;

        state
    })
    .await;
```

`reduce_mut_future` for the `CoW` approach. Note the extra `Box::pin` that is required
here. This is due to a current limitation of Rust, and should be phased out in the future.
```rust
dispatch
    .reduce_mut_future(|state| {
        Box::pin(async move {
            state.update_user().await;
        })
    })
    .await;
```

You can also create callbacks that execute a future when called. Note these are simple wrappers over
`yew::platform::spawn_local`.

`reduce_future_callback` for the pure approach.
```rust
let cb = dispatch.reduce_future_callback(|state| async move {
    let mut state = state.as_ref().clone();
    state.update_user().await;

    state
});
```

`reduce_mut_future_callback` for the `CoW` approach. Note the extra `Box::pin` that is required
here. This is due to a current limitation of Rust, and should be phased out in the future.

```rust
let cb = dispatch.reduce_mut_future_callback(|state| {
    Box::pin(async move {
        state.update_user().await;
    })
});
```

# Tips

## Setting default store values

The best way to define the default value of your store is by manually implementing `Default`.

```rust
#[derive(PartialEq, Store)]
struct MyStore {
    foo: String,
    bar: String,
}

impl Default for MyStore {
    fn default() -> Self {
        Self {
            foo: "foo".to_string(),
            bar: "bar".to_string(),
        }
    }
}

```

However sometimes you may need additional context to set the initial value of your store. To do
this, there are a couple options.

You can set the value at the beginning of your application, before your app renders (like in your
main fn).

```rust
fn main() {
    // .. other setup logic and whatnot
    Dispatch::<MyStore>::new().set(MyStore { ... });
    // ... now you can render your app!
}
```

Or inside some component using `use_effect_with_deps`, provided deps of `()`. Be sure to keep it in
a root component!

```rust
use_effect_with_deps(
    move || {
        // .. other setup logic and whatnot
        Dispatch::<MyStore>::new().set(MyStore { ... });
        || {}
    },
    (),
);
```

Keep in mind your store will still be initialized with `Store::new` (usually that's set to
`Default::default()`), however, because Rust is awesome, no fields are allocated initially, and
overwriting is very cheap.
