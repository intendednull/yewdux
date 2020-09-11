This crate provides ergonomic access to shared state via component wrapper, with optional
local/session persistence and custom scoping. 

Initially this was a [PR](https://github.com/yewstack/yew/pull/1372), but became big
enough to warrant a standalone crate.

If you have suggestions please open an issue, or join in on the [discussion](https://github.com/yewstack/yew/issues/576).

# Quickstart
To get started use the `SharedStateComponent` wrapper or `StateView` component.

## SharedStateComponent
Give your component any properties that implement `SharedState` then wrap it with 
`SharedStateComponent`.

IMPORTANT: Changes **must** be handled in the component's `change` method.
```rust
use yew::prelude::*;
use yew_state::{SharedHandle, SharedStateComponent};
use yewtil::NeqAssign;

#[derive(Clone, Default)]
pub type AppState {
    pub count: usize,
}

pub struct Model {
    handle: SharedHandle<AppState>,
}

impl Component for Model {
    type Message = ();
    type Properties = SharedHandle<AppState>;

    fn create(handle: Self::Properties, _link: ComponentLink<Self>) -> Self {
        handle.reduce(|state| state.count = 1);  // Magically set count to one for example
        Model { handle }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        true
    }

    fn change(&mut self, handle: Self::Properties) -> ShouldRender {
        self.handle.neq_assign(handle)
    }

    fn view(&self) -> Html {
        let onclick = self.handle.reduce_callback(|state| state.count += 1);
        let count = self.handle.state().count;
        html! {
            <p>{count}</p>
            <button onclick=onclick>{"+1"}</button>
        }
    }
}

pub type App = SharedStateComponent<Model>;
```

## StateView
For something simpler, `StateView` can *handle* shared state with less boilerplate.

Keep in mind you can't selectively re-render changes this way.

```rust
use yew::prelude::*;
use yew_state::{view_state, StateView, SharedHandle};

type CountHandle = SharedHandle<usize>;

fn view_counter() -> Html {
    html! {
        <>
            { view_display() }
            { view_input() }
        </>
    }
}

fn view_display() -> Html {
    let view = view_state(|handle: &CountHandle| {
        html! {
            <p>{handle.state()}</p>
        }
    });
    html! {
        <StateView<CountHandle> view=view />
    }
}

fn view_input() -> Html {
    let view = view_state(|handle: &CountHandle| {
        let onclick = handle.reduce_callback(|count| *count += 1);
        html! {
            <button onclick=onclick>{"+1"}</button>
        }
    });
    html! {
        <StateView<CountHandle> view=view />
    }
}
```

# Handling State
State handles provide an interface to shared state. `SharedHandle` for basic access, while 
`StorageHandle` also does persistent local/session storage.

IMPORTANT: Changes to state do not take effect immediately! New state must be handled in the
component's `change` method.

`state` provides current state.
```rust
let state: &T = self.handle.state();
```

`reduce` can be used from anywhere to modify shared state.
```rust
// SharedHandle<MyAppState>
self.handle.reduce(move |state| state.user = new_user);
```

`reduce_callback` allows modifying shared state from a callback.
```rust
// SharedHandle<usize>
let onclick = self.handle.reduce_callback(|state| *state += 1);
html! {
    <button onclick=onclick>{"+1"}</button>
}
```

`reduce_callback_with` provides the fired event as well.
```rust
let oninput = self
    .handle
    .reduce_callback_with(|state, i: InputData| state.user.name = i.value);

html! {
    <input type="text" placeholder="Enter your name" oninput=oninput />
}
```

## Custom Properties
`SharedState` can be implemented for any properties.

TODO: This could be a macro.
```rust
#[derive(Clone, Properties)]
pub struct Props {
    #[prop_or_default]
    handle: SharedHandle<AppState>,
}

impl SharedState for Props {
    type Handle = SharedHandle<AppState>;

    fn handle(&mut self) -> &mut Self::Handle {
        &mut self.handle
    }
}
```

## Persistence
To make state persistent use `StorageHandle` instead of `SharedHandle`. This requires state to also implement `Serialize`,
`Deserialize`, and `Storable`.

TODO: This could be a macro.
```rust
use serde::{Serialize, Deserialize};
use yew_state::{Storable, Area};

#[derive(Clone, Default, Serialize, Deserialize)]
struct T;

impl Storable for T {
    fn area() -> Area {
        
        Area::Session // Defaults to Area::Local
    }
}
```
## Scoping
Sometimes it's useful to only share state within a specific scope. This may be done by providing a
custom scope to `SharedStateComponent` or `StateView`:

```rust
pub struct MyScope;
pub struct MyComponent = SharedStateComponent<MyModel, MyScope>;
```
### Example
This example demonstrates how two counters with different scopes can increment shared state 
independently.
```rust
use yew::prelude::*;
use yew_state::{view_state, StateView, SharedHandle};

struct FooScope;
struct BarScope;

type CountHandle = SharedHandle<usize>;

fn view_input<SCOPE: 'static>() -> Html {
    let view = view_state(|handle: &CountHandle| {
        let onclick = handle.reduce_callback(|count| *count += 1);
        html! {
            <button onclick=onclick>{"+1"}</button>
        }
    });
    html! {
        <StateView<CountHandle, SCOPE> view=view />
    }
}
fn view_display<SCOPE: 'static>() -> Html {
    let view = view_state(|handle: &CountHandle| {
        html! {
            <p>{handle.state()}</p>
        }
    });
    html! {
        <StateView<CountHandle, SCOPE> view=view />
    }
}

pub struct App;
impl Component for App {
    type Message = ();
    type Properties = ();

    fn create(_props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        Self
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        true
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        true
    }

    fn view(&self) -> Html {
        html! {
            <>
                <h1>{"FooScope"}</h1>
                { view_display::<FooScope>() }
                { view_input::<FooScope>() }
                <h1>{"BarScope"}</h1>
                { view_display::<BarScope>() }
                { view_input::<BarScope>() }
            </>
        }
    }
}
```
