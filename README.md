State management is hard, especially when many potentially isolated components need mutable access
to shared state. Yewdux provides an ergonomic interface that handles all the boring message passing
and property propagation for you!

Redux users should feel at home, however Yewdux is expressed in Rust so the api is not 1:1 (nor does
it try to be).

# Install

Install using [cargo-edit](https://github.com/killercup/cargo-edit):

```
$ cargo install cargo-edit
$ cargo add yewdux
```

Or add it to your project's `Cargo.toml`:

```toml
[dependencies]
yewdux = "^0.6"
```

# Quickstart

## WithDispatch

Tired of message passing? Yewdux handles it all for you! Simply give your component `Dispatch`
properties and wrap it in the `WithDispatch` component wrapper.

```rust
use yew::prelude::*;
use yewdux::{Dispatch, WithDispatch, BasicStore};
use yewtil::NeqAssign;

#[derive(Clone, PartialEq)]
struct Counter {
    count: u64,
}

struct Model {
    dispatch: Dispatch<BasicStore<Counter>>,
}

impl Component for Model {
    type Properties = Dispatch<BasicStore<Counter>>;
    type Message = ();

    fn create(dispatch: Self::Properties, _link: ComponentLink<Self>) -> Self {
        // Magically increment counter when created, for this example
        dispatch.reduce(|s| s.count += 1);
        Self { dispatch }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        false
    }

    fn change(&mut self, dispatch: Self::Properties) -> ShouldRender {
        // Receive new state here.
        // IMPORTANT: Without this we don't get new state!
        self.dispatch.neq_assign(dispatch)
    }

    fn view(&self) -> Html {
        // Increment count by 1
        let incr = self.dispatch.reduce_callback(|s| s.count += 1);
        // Current state of counter count
        let count = self.dispatch.state().count;

        html! {
            <button onclick=incr>{ count }</button>
        }
    }
}

// Wrap our component, so it receives updates to state.
// IMPORTANT: Without this we don't get new state!
type App = WithDispatch<Model>;

// Start our app
fn main() {
    yew::start_app::<App>();
}
```

### DispatchProp

Custom properties work too! Just implement `DispatchProp`:

```rust
struct Props {
    dispatch: Dispatch<BasicStore<Counter>>,
}

impl DispatchProp for Props {
    type Store = BasicStore<Counter>;

    fn dispatch(&mut self) -> &mut Dispatch<Self::Store> {
        &mut self.dispatch
    }
}
```


# Persistence

Yewdux supports state persistence, so you don't lose it when your app reloads. This requires your
state to also implement `Serialize`, `Deserialize`, and `Persistent`.

```rust
use serde::{Serialize, Deserialize};
use yewdux::{Persistent, Area, Dispatch, PersistentStore};

#[derive(Clone, Default, Serialize, Deserialize)]
struct T;

impl Persistent for T {
    fn area() -> Area {
        Area::Session // Default is Area::Local
    }
}

struct App {
    dispatch: Dispatch<PersistentStore<T>>,
}
```

