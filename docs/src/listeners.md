# Listeners

Listeners are component-less subscribers. They are used to describe side-effects that should happen
whenever state changes. They live for application lifetime, and are created with `init_listener`.

Here's a simple listener that logs the current state whenever it changes.
```rust
# extern crate yew;
# extern crate yewdux;
# use yew::prelude::*;
use std::rc::Rc;

use yewdux::prelude::*;

#[derive(Default, Clone, PartialEq, Debug, Store)]
struct State {
    count: u32,
}

// The listener itself doesn't hold any state in this case, so we'll use an empty type. It's also
// possible to have stateful listeners.
struct StateLogger;
impl Listener for StateLogger {
    // Here's where we define which store we are listening to.
    type Store = State;
    // Here's where we decide what happens when `State` changes.
    fn on_change(&mut self, _cx: &yewdux::Context, state: Rc<Self::Store>) {
        yewdux::log::info!("state changed: {:?}", state);
    }
}
```

Can can start the listener by calling `init_listener` somewhere in our code. A good place to put it is
the store constructor.

**NOTE**: Successive calls to `init_listener` on the same type will replace the existing listener
with the new one.

```rust
# extern crate yewdux;
# use std::rc::Rc;
# use yewdux::prelude::*;
# #[derive(Default, PartialEq, Debug)]
# struct State {
#     count: u32,
# }
# // Doesn't hold any state, so we'll use an empty type.
# struct StateLogger;
# impl Listener for StateLogger {
#     // Here's where we say which store we want to subscribe to.
#     type Store = State;
#
#     fn on_change(&mut self, _cx: &yewdux::Context, state: Rc<Self::Store>) {
#         yewdux::log::info!("state changed: {:?}", state);
#     }
# }

impl Store for State {
    fn new(cx: &yewdux::Context) -> Self {
        init_listener(StateLogger, cx);
        Default::default()
    }

    fn should_notify(&self, other: &Self) -> bool {
        self != other
    }
}
```

