# Yewdux

Simple state management for [Yew](https://yew.rs/docs/en/) applications.

This is the development branch. Latest stable release may be found
[here](https://github.com/intendednull/yewdux/tree/0.6.2).

# Install

Add Yewdux to your project's `Cargo.toml`:

```toml
[dependencies]
yewdux = { git = "https://github.com/intendednull/yewdux.git" }
```

# Usage

`Dispatch` is the primary interface for Yewdux. It allows shared mutable access to global
application state, held in specialized agent-like containers called `Store`s.

## Writing to shared state

Creating a dispatch is simple, just give it a store type. 

```rust
use yewdux::prelude::*;

#[derive(Clone, Default)]
struct MyState {
    count: usize,
}

let dispatch = Dispatch::<BasicStore<MyState>>::new();
```

Note we're using `BasicStore` above, which is one of the default store types provided by Yewdux. [It
is also possible to define your own store type](https://github.com/intendednull/yewdux/blob/master/examples/store/src/main.rs).

Mutating state is done through `reduce`. Here we immediately send a message to increment count by 1.

```rust
dispatch.reduce(|state| state.count += 1);
```

We may also create callbacks that do the same. This button sends the message every time it is clicked.

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

## Reading shared state

To read state we need only a bridge to receive it. State is received once when a bridge is
created, and every time state is changed afterwards.

```rust
use std::rc::Rc;

use yew::prelude::*;
use yewdux::prelude::*;

...

struct MyComponent {
    dispatch: Dispatch<BasicStore<MyState>>,
    state: Option<Rc<MyState>>,
}

enum Msg {
    State(Rc<MyState>),
}

impl Component for MyComponent {
    type Properties = (); 
    type Message = Msg;

    fn create(ctx: &Context<Self>) -> Self {
        // Create a bridge to receive new state. Changes are handled in `update`.
        let dispatch = Dispatch::bridge_state(ctx.link().callback(Msg::State));
        Self {
            dispatch,
            state: Default::default()
        }
    }

    fn update(&self, ctx: &Context<Self>, msg: Msg) -> bool {
        match msg {
            // Receive new state
            Msg::State(state) => {
                self.state = Some(state);
                true
            }
        }
    }

    ...
}
```

### Less boilerplate please

Setting up a bridge for every component can be cumbersome. A solution is provided to handle this
automatically: `WithDispatch` and `DispatchProps`.

Simply give your component `DispatchProps` properties and wrap it with the `WithDispatch` component
wrapper.

**IMPORTANT**: `WithDispatch` and `DispatchProps` **must** be used together, or your app will panic.

```rust
struct MyComponentBase;
// I suggest using a type alias. Saves you some typing :)
type MyComponent = WithDispatch<MyComponentBase>;

impl Component for MyComponentBase {
    type Properties = DispatchProps<BasicStore<MyState>>; 
    ...
}
html! {
    <MyComponent />
}
```

Now your component will automatically receive updates to state. Its properties also behave exactly
like a regular `Dispatch`, with the notable addition of a single method for getting state.

```rust
let onclick = ctx.props().reduce_callback(|s| s.count + 1);
let count = ctx.props().state().count;

html! {
    <>
    <p>{"Count is "}{ count }</p>
    <button {onclick}>{"+1"}</button>
    </>
}
```

Did you notice we don't have to deal with an `Option` this way? The component wrapper postpones
rendering until it receives state for the first time, making it a little more ergonomic to use. 


#### Need custom properties?

`WithDispatch` wrapper works with any component that has properties which implement
`WithDispatchProps`. Simply implement it for your properties and you're good to go! This is likely
to be a macro in the future.

```rust
#[derive(Properties, Clone, PartialEq)]
struct Props {
    dispatch: DispatchProps<BasicStore<MyState>>,
    ...
}

impl WithDispatchProps for Props {
    type Store = BasicStore<MyState>;

    fn dispatch(&self) -> &DispatchProps<Self::Store> {
        &self.dispatch
    }
}
```

# Persistence

Yewdux supports state persistence so you don't lose it when your app reloads. This requires your
state to also implement `Serialize`, `Deserialize`, and `Persistent`.

```rust
use serde::{Serialize, Deserialize};
use yewdux::prelude::*;

#[derive(Clone, Default, Serialize, Deserialize)]
struct MyState { ... };

impl Persistent for MyState {
    fn area() -> Area {
        Area::Session // Default is Area::Local
    }
}

struct MyComponent {
    dispatch: Dispatch<PersistentStore<State>>,
}
```

A persistent store checks for previously saved state on startup, using default if none is found.
State is saved on every change.

# Functional

Yewdux supports functional! 

Add it to your project:

```toml
[dependencies]
yewdux-functional = { git = "https://github.com/intendednull/yewdux.git" }
```

And enjoy the terse goodness:

```rust
use yew::{prelude::*, functional::*};
use yewdux::prelude::*;
use yewdux_functional::*;


#[derive(function_component(MyComponent))]
fn my_component() -> Html {
    let store = use_store::<BasicStore<MyState>>();
    let onclick = store.dispatch().reduce_callback(|s| s.count += 1);
    let count = store.state().map(|s| s.count).unwrap_or_default();

    html! {
        <>
        <p>{"Count is "}{ count }</p>
        <button {onclick}>{"+1"}</button>
        </>
    }
}
```

# Examples

Complete working examples can be found
[here](https://github.com/intendednull/yewdux/tree/master/examples).

To run an example you'll need to install [trunk](https://github.com/thedodd/trunk), then run
(replacing [example] with your desired example name):

    trunk serve examples/[example]/index.html --open

