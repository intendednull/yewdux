# SSR Support

By default Yewdux uses a global `Context` that is shared thread-locally. This means we can share
state from anywhere in our code as long as it's within the same thread. Wasm applications are
strictly single threaded (without workers), so it isn't a problem.

However the same cannot be said for server side rendering. It is very possible the server is
executing in a multi-threaded environment, which could cause various problems for Yewdux's
single-threaded assumption.

While multi-threaded globally shared state is technically possible, it is currently not supported.

Instead Yewdux offers a custom component to hold your shared application state: `YewduxRoot`. This
ensures all state is kept inside your Yew app.

```rust
# extern crate yew;
# extern crate yewdux;
use yew::prelude::*;
use yewdux::prelude::*;

#[derive(Default, Clone, PartialEq, Eq, Store)]
struct State {
    count: u32,
}

#[function_component]
fn Counter() -> Html {
    let (state, dispatch) = use_store::<State>();
    let onclick = dispatch.reduce_mut_callback(|state| state.count += 1);
    html! {
        <>
        <p>{ state.count }</p>
        <button {onclick}>{"+1"}</button>
        </>
    }
}

#[function_component]
fn App() -> Html {
    // YewduxRoot must be kept above all components that use any of your stores.
    html! {
        <YewduxRoot>
            <Counter />
        </YewduxRoot>
    }
}
```

Yewdux hooks automatically detect when YewduxRoot is present, and use it accordingly.

## SSR with struct components

For struct component support, refer to the [higher order components
pattern](https://yew.rs/docs/advanced-topics/struct-components/hoc).

```rust
# extern crate yew;
# extern crate yewdux;
use std::rc::Rc;

use yew::prelude::*;
use yewdux::prelude::*;

#[derive(Default, Clone, PartialEq, Eq, Store)]
struct State {
    count: u32,
}

#[derive(Properties, Clone, PartialEq)]
struct Props {
    dispatch: Dispatch<State>,
}

enum Msg {
    StateChanged(Rc<State>),
}

struct MyComponent {
    state: Rc<State>,
    dispatch: Dispatch<State>,
}

impl Component for MyComponent {
    type Properties = Props;
    type Message = Msg;

    fn create(ctx: &Context<Self>) -> Self {
        let callback = ctx.link().callback(Msg::StateChanged);
        let dispatch = ctx.props().dispatch.clone().subscribe_silent(callback);
        Self {
            state: dispatch.get(),
            dispatch,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::StateChanged(state) => {
                self.state = state;
                true
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

#[function_component]
fn MyComponentHoc() -> Html {
    let dispatch = use_dispatch::<State>();

    html! {
        <MyComponent {dispatch} />
    }
}


#[function_component]
fn App() -> Html {
    // YewduxRoot must be kept above all components that use any of your stores.
    html! {
        <YewduxRoot>
            <MyComponentHoc />
        </YewduxRoot>
    }
}
```
