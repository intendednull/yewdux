# Creating a dispatch

A [Dispatch](https://docs.rs/yewdux/latest/yewdux/dispatch/struct.Dispatch.html) is the primary
interface to access your [Store](https://docs.rs/yewdux/latest/yewdux/store/trait.Store.html). It
can be used to read and write changes to state in various ways.

## Hooks

A dispatch is provided when using the functional hook, which is only available in yew functional
components.

**IMPORTANT**: Like other hooks, all yewdux hooks must be used at the top level of a function
component.

```rust
# extern crate yewdux;
# extern crate yew;
# use yewdux::prelude::*;
# use yew::prelude::*;
#[derive(Default, PartialEq, Store)]
struct State {
    count: u32,
}

#[function_component]
fn MyComponent() -> Html {
    let (state, dispatch) = use_store::<State>();
    html! {
        // Component stuff here
    }
}
```

See [the docs](https://docs.rs/yewdux/latest/yewdux/functional/index.html) for a full list of
available hooks.

## Manually

To create a dispatch, you need only provide the desired store type. This is available in **any**
rust code, not just yew components.

```rust
# extern crate yewdux;
# use yewdux::prelude::*;
# #[derive(Default, PartialEq, Store)]
# struct State {
#     count: u32,
# }
let dispatch = Dispatch::<State>::global();
```

**NOTE**: Here we create a global dispatch, which is only available for wasm targets. See
[SSR support](./ssr.md) for alternatives.

# Changing state

`Dispatch` provides many options for changing state. Here are a few handy methods. For a full list
see the [docs](https://docs.rs/yewdux/latest/yewdux/dispatch/struct.Dispatch.html#)


```rust
# extern crate yewdux;
# extern crate yew;
# use yewdux::prelude::*;
# use yew::prelude::*;
#[derive(Default, PartialEq, Store)]
struct State {
    count: u32,
}

// Create a global dispatch
let dispatch = Dispatch::<State>::global();

// Set the value immediately
dispatch.set(State { count: 0 });

// Set the value immediately based on the last value
dispatch.reduce(|state| State { count: state.count + 1}.into());

// Create a callback to set the value when a button is clicked
let onclick = dispatch.reduce_callback(|state| State { count: state.count + 1}.into());
html! {
    <button {onclick}>{"Increment (+1)"}</button>
};
```

## Mut reducers

There are `_mut` variants to every reducer function. This way has less boilerplate, and requires
your `Store` to implement `Clone`. Your `Store` *may* be cloned once per mutation,

```rust
# extern crate yewdux;
# extern crate yew;
# use yewdux::prelude::*;
# use yew::prelude::*;
#[derive(Default, PartialEq, Clone, Store)]
struct State {
    count: u32,
}

// Create a global dispatch
let dispatch = Dispatch::<State>::global();

// Mutate the current value
dispatch.reduce_mut(|state| state.count += 1);

// Create a callback to mutate the value when a button is clicked
let onclick = dispatch.reduce_mut_callback(|counter| counter.count += 1);
html! {
    <button {onclick}>{"Increment (+1)"}</button>
};
```

## Predictable mutations

Yewdux supports predictable mutation. Simply define your message and apply it.

```rust
# extern crate yewdux;
# extern crate yew;
use std::rc::Rc;

use yew::prelude::*;
use yewdux::prelude::*;

#[derive(Default, PartialEq, Clone, Store)]
struct State {
    count: u32,
}

enum Msg {
    AddOne,
}

impl Reducer<State> for Msg {
    fn apply(self, state: Rc<State>) -> Rc<State> {
        match self {
            Msg::AddOne => State { count: state.count + 1 }.into(),
        }
    }
}

let dispatch = Dispatch::<State>::global();

dispatch.apply(Msg::AddOne);

let onclick = dispatch.apply_callback(|_| Msg::AddOne);
html! {
    <button {onclick}>{"Increment (+1)"}</button>
};
```

### Tip

`Rc::make_mut` is handy if you prefer CoW:

```rust
# extern crate yewdux;
# use std::rc::Rc;
# use yewdux::prelude::*;
# #[derive(Default, PartialEq, Clone, Store)]
# struct State {
#     count: u32,
# }
# enum Msg {
#     AddOne,
# }
impl Reducer<State> for Msg {
    fn apply(self, mut state: Rc<State>) -> Rc<State> {
        let state_mut = Rc::make_mut(&mut state);

        match self {
            Msg::AddOne => state_mut.count += 1,
        };

        state
    }
}
```

## Future support

Because a `Dispatch` may be created and executed from anywhere, Yewdux has innate future support.
Just use it normally, no additonal setup is needed.

```rust
# extern crate yewdux;
# extern crate yew;
# use std::rc::Rc;
# use yewdux::prelude::*;
# use yew::prelude::*;

#[derive(Default, PartialEq, Store)]
struct User {
    name: Option<Rc<str>>,
}

async fn get_user() -> User {
    User { name: Some("bob".into()) }
}

let dispatch = Dispatch::<User>::global();
// Use yew::platform::spawn_local to run a future.
let future = async move {
    let user = get_user().await;
    dispatch.set(user);
};
```

