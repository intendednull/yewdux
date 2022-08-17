# Reading state

Reading global state is a little trickier than it might seem. Most of the time, components need to
not only know the current state, but also get the new state whenever it changes. For that, we must
subscribe to changes.

## Subscribing to changes

Components need to know when to re-render for changes. To do this they can subscribe to a store.

Functional hooks like `use_store` will subscribe automatically.

```rust
// `counter` is automatically updated when global state changes.
let (counter, dispatch) = use_store::<Counter>();
```

You may also subscribe manually, as shown below. At the cost of boilerplate, doing it this way
allows much finer control.

```rust
use std::rc::Rc;

use yew::prelude::*;
use yewdux::prelude::*;

struct MyComponent {
    dispatch: Dispatch<Counter>,
    counter: Rc<Counter>,

}

enum Msg {
    UpdateCounter(Rc<Counter>),
}

impl Component for MyComponent {
    type Properties = ();
    type Message = Msg;

    fn create(ctx: &Context<Self>) -> Self {
        // The callback for receiving updates to state.
        let callback = ctx.link().callback(Msg::UpdateCounter);
        // Subscribe to changes in state. New state is received in `update`. Be sure to save this,
        // dropping it will unsubscribe.
        let dispatch = Dispatch::<Counter>::subscribe(callback);
        Self {
            // Get the current state.
            counter: dispatch.get(),
            dispatch,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Msg) -> bool {
        match msg {
            // Receive new state.
            Msg::UpdateCounter(counter) => {
                self.counter = counter;

                // Only re-render this component if count is greater that 0 (for example).
                if self.counter.count > 0 {
                    true
                } else {
                    false
                }
            }
        }
    }

    ...
}
```

## Immediate state

It is also possible to retrieve the current state of a store without subscribing to changes. This is
useful when you don't really care when/if state has changed, just what the current value is.

`Dispatch::get` will lookup the current value immediately:

```rust
let state = dispatch.get();
```

