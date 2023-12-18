# Reading state

To get the current state of your store immediately, use `Dispatch::get`:

**IMPORTANT**: Reading the state this way **does not** provide any sort of change detection, and
your component **will not** automatically re-render when state changes.

```rust
# extern crate yewdux;
use std::rc::Rc;

use yewdux::prelude::*;

#[derive(PartialEq, Default, Store)]
struct State {
    count: u32,
}

// Create a dispatch from the global context. This works for non-global contexts too, we would just
// pass in the context we want.
let dispatch = Dispatch::<State>::global();
let state: Rc<State> = dispatch.get();
```

## Subscribing to your store

In order for your component to know when state changes, we need to subscribe.

### Function components

The `use_store` hook automatically subscribes to your store, and re-renders when state changes. This
**must** be called at the top level of your function component.

```rust
# extern crate yewdux;
# extern crate yew;
# use yewdux::prelude::*;
# use yew::prelude::*;
# #[derive(PartialEq, Default, Store)]
# struct State {
#     count: u32,
# }
#[function_component]
fn ViewCount() -> Html {
    let (state, dispatch) = use_store::<State>();
    html!(state.count)
}
```

### Struct components

For struct components we need to subscribe manually. This way allows much finer control, at the cost
of extra boilerplate.

**IMPORTANT**: Remember to hold onto your dispatch instance. Dropping it will drop the entire
subscription, and you will **not** receive changes to state.

```rust
# extern crate yewdux;
# extern crate yew;
use std::rc::Rc;

use yew::prelude::*;
use yewdux::prelude::*;

#[derive(PartialEq, Default, Clone, Store)]
struct State {
    count: u32,
}

struct MyComponent {
    dispatch: Dispatch<State>,
    state: Rc<State>,

}

enum Msg {
    StateChanged(Rc<State>),
}

impl Component for MyComponent {
    type Properties = ();
    type Message = Msg;

    fn create(ctx: &Context<Self>) -> Self {
        // The callback for receiving updates to state.
        let callback = ctx.link().callback(Msg::StateChanged);
        // Subscribe to changes in state. New state is received in `update`. Be sure to save this,
        // dropping it will unsubscribe.
        let dispatch = Dispatch::<State>::global().subscribe_silent(callback);
        Self {
            // Get the current state.
            state: dispatch.get(),
            dispatch,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Msg) -> bool {
        match msg {
            // Receive new state.
            Msg::StateChanged(state) => {
                self.state = state;

                // Only re-render this component if count is greater that 0 (for this example).
                if self.state.count > 0 {
                    true
                } else {
                    false
                }
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let count = self.state.count;
        let onclick = self.dispatch.reduce_mut_callback(|s| s.count += 1);
        html! {
            <>
            <h1>{ count }</h1>
            <button onclick={onclick}>{"+1"}</button>
            </>
        }
    }

}
```

# Selectors

Sometimes a component will only care about a particular part of state, and only needs to re-render
when that part changes. For this we have the `use_selector` hook.

```rust
# extern crate yewdux;
# extern crate yew;
use yewdux::prelude::*;
use yew::prelude::*;

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

## Capturing your environment

For selectors that need to capture variables from their environment, be sure to provide them as
dependencies to `use_selector_with_deps`. Otherwise your selector won't update correctly!

```rust
# extern crate yewdux;
# extern crate yew;
use std::collections::HashMap;

use yewdux::prelude::*;
use yew::prelude::*;

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
    let item = match item.as_ref() {
        Some(item) => item,
        None => return Default::default(),
    };

    html! {
        <p>{ item }</p>
    }
}
```
