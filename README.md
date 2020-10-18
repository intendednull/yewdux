State management in Yew can become complex, especially when many components need mutable access to
shared state. This crate simplifies that access (comparable to context hooks in React) so you can
spend less time writing boilerplate!

# Install

Install this package from your terminal:

```
$ cargo install cargo-edit
$ cargo add yew-state
```

Or add it to your project's `Cargo.toml`:

```toml
[dependencies]
yew-state = "^0.4"
```

# Quickstart

## SharedStateComponent

Give your components shared state by adding `SharedHandle` properties and wrapping them in
`SharedStateComponent`.

```rust
use yew::prelude::*;
use yew_state::{SharedHandle, SharedStateComponent};
use yewtil::NeqAssign;

type Handle = SharedHandle<u64>;

enum Msg {
    Reset,
}

struct Model {
    handle: Handle,
    link: ComponentLink<Self>,
}

impl Component for Model {
    type Properties = Handle;
    type Message = Msg;

    fn create(handle: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self { handle, link }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Reset => {
                // Reset count to 0.
                self.handle.reduce(|count| *count = 0);
                // Don't render yet, receive changes in `change`.
                false
            }
        }
    }

    fn change(&mut self, handle: Self::Properties) -> ShouldRender {
        // Receive new state here.
        self.handle.neq_assign(handle)
    }

    fn view(&self) -> Html {
        // Increment count by 1.
        let incr = self.handle.reduce_callback(|count| *count += 1);
        // Emit message to reset count.
        let reset = self.link.callback(|_| Msg::Reset);

        html! {
            <>
            <button onclick=incr>{ self.handle.state() }</button>
            <button onclick=reset>{"Reset"}</button>
            </>
        }
    }
}

type App = SharedStateComponent<Model>;
```

## StateView

`StateView` components are a convenient way to write quick and simple access to shared state. At the
cost of a little control, they require almost no boilerplate:

```rust
use yew::prelude::*;
use yew_state::{component, SharedHandle, StateView};

fn view_counter() -> Html {
    type Handle = SharedHandle<u64>;

    let view = component::view(|handle: &Handle| {
        // Increment count by 1.
        let incr = handle.reduce_callback(|count| *count += 1);
        // Reset count to 0.
        let reset = handle.reduce_callback(|count| *count = 0);

        html! {
            <>
            <button onclick=incr>{ handle.state() }</button>
            <button onclick=reset>{"Reset"}</button>
            </>
        }
    });

    html! {
        <StateView<Handle> view=view />
    }
}
```

# Usage

Shared state is accessed through state handles (`SharedHandle` or `StorageHandle`), which are
managed by the component wrapper `SharedStateComponent`. This wrapper takes care of all the boring
message passing for sending and receiving updates to state. Updated state is then passed to your
component like any other properties, handled in `Component::change`.

Add the following to your component to give it shared state (other details omitted):

```rust
use yew::prelude::*;
use yew_state::{SharedHandle, SharedStateComponent};

#[derive(Default, Clone)]
struct MyState {
    // ..
}

struct Model {
    handle: SharedHandle<MyState>,
}

impl Component for Model {
    type Properties = SharedHandle<MyState>;
    // ..
}

type MyComponent = SharedStateComponent<Model>;
```

## State Handles

`state` provides a reference to current state:

```rust
let state: &MyState = self.handle.state();
```

`reduce` lets you mutate shared state directly:

```rust
// SharedHandle<UserState>
self.handle.reduce(move |user| *user = new_user);
```

`reduce_callback` allows modifying shared state from a callback:

```rust
// SharedHandle<usize>
let onclick = self.handle.reduce_callback(|state| *state += 1);
html! {
    <button onclick=onclick>{"+1"}</button>
}
```

`reduce_callback_with` provides the fired event as well:

```rust
// SharedHandle<UserState>
let oninput = self
    .handle
    .reduce_callback_with(|user, i: InputData| user.name = i.value);

html! {
    <input type="text" placeholder="Enter your name" oninput=oninput />
}
```

`reduce_callback_once` and `reduce_callback_once_with` are also provided for `Callback::Once`
variants.

## More on StateView

`StateView` supports a couple other hooks in addition to `view` which allow a little more control
over component behavior: `rendered` and `change`.

```rust
use yew::prelude::*;
use yew_state::{component, SharedHandle, StateView};

fn view_counter() -> Html {
    type Handle = SharedHandle<usize>;

    // Display counter button.
    let view = component::view(|handle: &Handle| {
        let onclick = handle.reduce_callback(|count| *count += 1);
        html! {
            <button onclick=onclick>{ handle.state() }</button>
        }
    });
    // Magically set count to 1 for example.
    let rendered = component::rendered(|handle: &Handle, first_render| {
        if first_render {
            handle.reduce(|count| *count = 1);
        }
    });
    // Reset count to 0 if greater than 10.
    let change = component::change(|old: &Handle, new: &Handle| -> ShouldRender {
        if *new.state() > 10 {
            new.reduce(|count| *count = 0);
        }

        old != new
    });

    html! {
        <StateView<Handle, SCOPE> view=view rendered=rendered change=change />
    }
}
```

## SharedState Properties

State handles derive `Properties` for convenience, but they can also be used from your own
properties. Just implement `SharedState`:

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

TODO: Add derive macro for `SharedState`

## Persistence

To make state persistent use a `StorageHandle`. This requires your state to also implement
`Serialize`, `Deserialize`, and `Storable`.

```rust
use serde::{Serialize, Deserialize};
use yew_state::{Storable, Area};

#[derive(Clone, Default, Serialize, Deserialize)]
struct T;

impl Storable for T {
    fn area() -> Area {
        Area::Session // Default is Area::Local
    }
}
```

Now your state won't be lost on refresh or if the user navigates away.

TODO: Add derive macro for `Storable`

## Scoping

By default all components use the same scope. Components only share state with other components that
have the same scope; changes to shared state in one scope do not affect components in a different
one.

To change a component's scope simply give it a different scope type:

```rust
struct MyScope;
type MyComponent = SharedStateComponent<MyModel, MyScope>;
```

### Example

This example demonstrates how two counters with different scopes can be incremented 
independently.

```rust
use yew::prelude::*;
use yew_state::{component, SharedHandle, StateView};

struct FooScope;
struct BarScope;

fn view_counter<SCOPE: 'static>() -> Html {
    type Handle = SharedHandle<usize>;

    let view = component::view(|handle: &Handle| {
        let onclick = handle.reduce_callback(|count| *count += 1);
        html! {
            <button onclick=onclick>{ handle.state() }</button>
        }
    });

    html! {
        <StateView<Handle, SCOPE> view=view />
    }
}

struct App;
impl Component for App {
    type Message = ();
    type Properties = ();

    fn create(_props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        Self
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        false
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        html! {
            <>
                <h1>{"FooScope"}</h1>
                { view_counter::<FooScope>() }
                <h1>{"BarScope"}</h1>
                { view_counter::<BarScope>() }
            </>
        }
    }
}
```

# Tips and Tricks

## Performance

### CoW says moo

We use a clone on write pattern to make changes to shared state. This lets components decide when to
receive changes in `Component::change`. If you need to share state that is expensive to clone, be
sure to wrap it in an `Rc`!

### Break it up

It helps to break up your app state so components only share what they need. This way components aren't
notified of changes that they don't care about. For example layout components might share a
`LayoutState` that can be updated without affecting your other components every time the layout
changes.

## No spaghetti please

For sanity's sake try to only modify shared state from a few components. As your app grows in
complexity it can become increasingly difficult to keep track of which components are mutating
state.

## Beware infinite render loops

Consider our quickstart example with a slight modification:

```rust
fn view_counter() -> Html {
    type Handle = SharedHandle<u64>;

    let view = component::view(|handle: &Handle| {
        // Increment count by 1 right away.
        // THIS WILL NEVER STOP COUNTING!
        handle.reduce(|count| *count += 1);
        // Increment count by 1.
        let incr = handle.reduce_callback(|count| *count += 1);
        // Reset count to 0.
        let reset = handle.reduce_callback(|count| *count = 0);

        html! {
            <>
            <button onclick=incr>{ handle.state() }</button>
            <button onclick=reset>{"Reset"}</button>
            </>
        }
    });

    html! {
        <StateView<Handle> view=view />
    }
}
```

This will compile, but as soon as `view_counter` is rendered your app will freeze as the counter
infinitely increments itself. 

The above example can be fixed like so:

```rust
if *handle.state() == 0 {
    handle.reduce(|count| *count += 1);
}
```

This is a simple example but it can happen many different ways. If your
app is freezing, chances are you've got a component caught in a render loop.

