# Yew State

This crate provides ergonomic access to shared state and optional persistent storage (session or local).

Initially this was a [PR](https://github.com/yewstack/yew/pull/1372), but became big
enough to warrant a standalone crate.

This is just one approach to shared state! It uses a component wrapper to manage message passing, and `Rc`
to minimize cloning. If you have suggestions please open an issue, or join in on the
[discussion](https://github.com/yewstack/yew/issues/576).

## Usage

Give your component `GlobalHandle` properties and wrap it with `SharedStateComponent`.
This may be done for any `T` that implements `Clone` + `Default`.
```rust
struct Model {
    handle: GlobalHandle<T>,
}

impl Component for Model {
    type Properties = GlobalHandle<T>;
    ...
}

type MyComponent = SharedStateComponent<Model>;
```

Access current state with `state`.
```rust
let state: &T = self.handle.state();
```

Modify shared state from anywhere using `reduce`
```rust
// GlobalHandle<MyAppState>
self.handle.reduce(|state| state.user = new_user);
```

or from a callback with `reduce_callback`.
```rust
// GlobalHandle<usize>
let onclick = self.handle.reduce_callback(|state| *state += 1);
html! {
    <button onclick = onclick>{"+1"}</button>
}
```

`reduce_callback_with` provides the fired event.
```rust
let oninput = self
    .handle
    .reduce_callback_with(|i: InputData, state| state.user.name = i.value);

html! {
    <input type="text" placeholder = "Enter your name" oninput = oninput />
}
```

### Properties with Shared State

Get shared state in custom props with `SharedState`.
```rust
#[derive(Clone, Properties)]
pub struct Props {
    #[prop_or_default]
    pub handle: GlobalHandle<AppState>,
}

impl SharedState for Props {
    type Handle = GlobalHandle<AppState>;

    fn handle(&mut self) -> &mut Self::Handle {
        &mut self.handle
    }
}
```

### State Persistence

Persistent storage requires that `T` also implement `Serialize`,
`Deserialize`, and `Storable`.
```rust
use serde::{Serialize, Deserialize};
use yew_state::Storable;
use yew::services::storage::Area;

#[derive(Serialize, Deserialize)]
struct T;

impl Storable for T {
    fn key() -> &'static str {
        "myapp.storage.t"
    }

    fn area() -> Area {
        // or Area::Session
        Area::Local
    }
}
```

Then use `StorageHandle` instead of `GlobalHandle`.

## Example

Lets make a counting app using shared state!

First the display:
```rust
// display.rs
use yew::prelude::*;
use yewtil::NeqAssign;
use yew_state::{Handle, GlobalHandle, SharedStateComponent};

pub struct Model {
    handle: GlobalHandle<usize>,
}

impl Component for Model {
    type Message = ();
    type Properties = GlobalHandle<usize>;

    fn create(handle: Self::Properties, _link: ComponentLink<Self>) -> Self {
        Self { handle }
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        true
    }

    fn change(&mut self, handle: Self::Properties) -> ShouldRender {
        self.handle.neq_assign(handle)
    }

    fn view(&self) -> Html {
        html! {
            <p>{ format!("Count: {}", self.handle.state()) }</p>
        }
    }
}

pub type Display = SharedStateComponent<Model>;
```

Now for the button:
```rust
// input.rs
use yew::prelude::*;
use yewtil::NeqAssign;
use yew_state::{GlobalHandle, Handle, SharedStateComponent};

pub struct Model {
    handle: GlobalHandle<usize>,
}

impl Component for Model {
    type Message = ();
    type Properties = GlobalHandle<usize>;

    fn create(handle: Self::Properties, _link: ComponentLink<Self>) -> Self {
        Model { handle }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        true
    }

    fn change(&mut self, handle: Self::Properties) -> ShouldRender {
        self.handle.neq_assign(handle)
    }

    fn view(&self) -> Html {
        let onclick = self.handle.reduce_callback(|state| *state += 1);
        html! {
            <button onclick = onclick>{"+1"}</button>
        }
    }
}

pub type Input = SharedStateComponent<Model>;
```

Finally the app:
```rust
// app.rs
use yew::prelude::*;

use crate::{display::Display, input::Input};


pub struct App;
impl Component for App {
    type Message = ();
    type Properties = ();

    fn create(_props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        Self
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        true
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        true
    }

    fn view(&self) -> Html {
        html! {
            <>
                <Display />
                <div>
                    <p>{"Start counting!"}</p>
                    <Input />
                </div>
            </>
        }
    }
}
```
