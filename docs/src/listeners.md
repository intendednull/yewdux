# Listeners

Listeners are component-less subscribers. They are used to describe side-effects that should happen
whenever state changes. They live for application lifetime, and are created with `init_listener`.

To see how this is useful, let's recreate store [persistence](./persistence.md). The full example can
be seen [here](https://github.com/intendednull/yewdux/blob/master/examples/listener/src/main.rs).

First we define `StorageListener`. Its implementation is super simple: whenever state changes we
save it to local storage.

```rust
// Doesn't hold any state, so we'll use an empty type.
struct StorageListener;
impl Listener for StorageListener {
    // Here's where we say which store we want to subscribe to.
    type Store = State;

    fn on_change(&mut self, state: Rc<Self::Store>) {
        storage::save(state.as_ref(), storage::Area::Local).expect("unable to save state");
    }
}
```

Now we define our store. To initialize `StorageListener`, we call `init_listener(StorageListener)`
in the `Store::new` method. This ensures it is always created along with `State`.

**NOTE**: Successive calls to `init_listener` on the same type will replace the existing listener
with the new one.

```rust
#[derive(Default, Clone, PartialEq, Eq, Deserialize, Serialize)]
struct State {
    count: u32,
}

impl Store for State {
    fn new() -> Self {
        init_listener(StorageListener, &yewdux::Context::global());

        storage::load(storage::Area::Local)
            .ok()
            .flatten()
            .unwrap_or_default()
    }

    fn should_notify(&self, other: &Self) -> bool {
        self != other
    }
}
```

Finally we use the store normally. If all goes well, your counter will now persist through page
visits!

```rust
#[function_component]
fn App() -> Html {
    let (state, dispatch) = use_store::<State>();
    let onclick = dispatch.reduce_mut_callback(|state| state.count += 1);

    html! {
        <>
        <p>{ state.count }</p>
        <button {onclick}>{"+1"}</button>
        </>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
```
