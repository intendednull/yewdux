Shared state containers for [Yew](https://yew.rs/docs/en/) applications.

State management in Yew can get cumbersome, especially when you need to give many (potentially
isolated) components mutable access to shared state. Normally you would need to write individual
properties and callbacks for each component to propagate changes -- too much typing if you as me!
Yewdux provides an ergonomic interface for shared state containers. They can be accessed from any
component or agent, live for entire application lifetime, and are clone-on-write by default.

# Install

Use [cargo-edit](https://github.com/killercup/cargo-edit):

```
$ cargo install cargo-edit
$ cargo add yewdux
```

Or add Yewdux to your project's `Cargo.toml`:

```toml
[dependencies]
yewdux = "^0.6"
```

# Usage

## Dispatch

Let's implement a global counter using Yewdux!

`Dispatch` provides various methods to interact with `Store`s. Here we'll use `BasicStore`, but you
could also write your own store implementation! 
```rust
use std::rc::Rc;

use yew::prelude::*;
use yewdux::prelude::*;

#[derive(Default, Clone)]
struct State {
    count: u32,
}

struct App {
    /// Our local version of state.
    state: Rc<State>,
    dispatch: Dispatch<BasicStore<State>>,
}

enum Msg {
    /// Message to receive new state.
    State(Rc<State>),
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        // Create Dispatch with a bridge that receives new state.
        let dispatch = Dispatch::bridge_state(link.callback(Msg::State)); 
        // Magically increment our counter for this example.
        // NOTE: Changes aren't immediate! We won't see new state until we receive it in our update
        // method.
        dispatch.reduce(|s| s.count += 1);

        Self {
            dispatch, 
            state: Default::default(),
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::State(state) => {
                // Receive new state and re-render.
                self.state = state;
                true
            }
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        let count = self.state.count;
        // We can modify state with callbacks too! 
        let incr = self.dispatch.reduce_callback(|s| s.count += 1);

        html! {
            <>
            <h1>{ count }</h1>
            <button onclick=incr>{"+1"}</button>
            </>
        }
    }
}


pub fn main() {
    yew::start_app::<App>();
}
```


## Automatic message passing

`Dispatch` is neat and all, but all that manual message passing is making me tired. Fortunately
Yewdux can handle that too! 

`WithDispatch` is a simple component wrapper that handles all that boring message passing for you.
Simply give your component `DispatchProps` properties. 

```rust
use yew::prelude::*;
use yewdux::prelude::*;
use yewtil::NeqAssign;

#[derive(Default, Clone)]
struct State {
    count: u32,
}

struct App {
    dispatch: DispatchProps<BasicStore<State>>,
}

impl Component for App {
    type Message = ();
    type Properties = DispatchProps<BasicStore<State>>;

    fn create(dispatch: Self::Properties, _link: ComponentLink<Self>) -> Self {
        Self { dispatch }
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        false
    }

    fn change(&mut self, dispatch: Self::Properties) -> ShouldRender {
        self.dispatch.neq_assign(dispatch)
    }

    fn view(&self) -> Html {
        let count = self.dispatch.state().count;
        let onclick = self.dispatch.reduce_callback(|s| s.count += 1);
        html! {
            <>
            <h1>{ count }</h1>
            <button onclick=onclick>{"+1"}</button>
            </>
        }
    }
}

pub fn main() {
    // IMPORTANT: Don't forget to wrap your component in `WithDispatch` or it will panic!
    yew::start_app::<WithDispatch<App>>();
}
```


### Custom properties 

`WithDispatch` works with any props that implement `DispatchPropsMut`:

```rust
#[derive(Properties, Clone)]
struct Props {
    dispatch: Dispatch<BasicStore<Counter>>,
    ...
}

impl DispatchPropsMut for Props {
    type Store = BasicStore<State>;

    fn dispatch(&mut self) -> &mut Dispatch<Self::Store> {
        &mut self.dispatch
    }
}
```


## Reducer

`ReducerStore` provides predictable, Redux-like behavior for your state containers. They need only
implement `Reducer`:

```rust
use yew::prelude::*;
use yewdux::prelude::*;
use yewtil::NeqAssign;

enum Action {
    Increment,
}

#[derive(Clone)]
struct Counter {
    count: u64,
}

impl Reducer for Counter {
    type Action = Action;

    fn new() -> Self {
        Self { count: 0 }
    }

    fn reduce(&mut self, action: Self::Action) -> ShouldNotify {
        match action {
            Action::Increment => {
                self.count += 1;
                true
            }
        }
    }
}

type AppDispatch = DispatchProps<ReducerStore<Counter>>;

struct App {
    dispatch: AppDispatch,
}

impl Component for App {
    type Message = ();
    type Properties = AppDispatch;

    fn create(dispatch: Self::Properties, _link: ComponentLink<Self>) -> Self {
        // Magically increment counter for this example.
        self.dispatch.send(Action::Increment);

        Self { dispatch }
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        false
    }

    fn change(&mut self, dispatch: Self::Properties) -> ShouldRender {
        self.dispatch.neq_assign(dispatch)
    }

    fn view(&self) -> Html {
        let count = self.dispatch.state().count;
        let increment = self.dispatch.callback(|_| Action::Increment);
        html! {
            <>
            <h1>{ count }</h1>
            <button onclick=increment>{"+1"}</button>
            </>
        }
    }
}

fn main() {
    yew::start_app::<WithDispatch<App>>();
}
```



## Persistence

Yewdux supports state persistence so you don't lose it when your app reloads. This requires your
state to also implement `Serialize`, `Deserialize`, and `Persistent`.

```rust
use serde::{Serialize, Deserialize};
use yewdux::prelude::*;

#[derive(Clone, Default, Serialize, Deserialize)]
struct State;

impl Persistent for State {
    fn area() -> Area {
        Area::Session // Default is Area::Local
    }
}

struct App {
    dispatch: Dispatch<PersistentStore<State>>,
}
```

## Examples

More examples can be found [here](https://github.com/intendednull/yewdux/tree/master/examples).

To run an example you'll need to install [trunk](https://github.com/thedodd/trunk), then run:

    trunk serve examples/[example]/index.html --open

