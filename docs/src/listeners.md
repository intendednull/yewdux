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

struct StateLogger;
impl Listener for StateLogger {
    // Here's where we define which store we are listening to.
    type Store = State;
    // Here's where we decide what happens when `State` changes.
    fn on_change(&self, _cx: &yewdux::Context, state: Rc<Self::Store>) {
        yewdux::log::info!("state changed: {:?}", state);
    }
}
```

Can can start the listener by calling `init_listener` somewhere in our code. A good place to put it is
the store constructor.

**NOTE**: Successive calls to `init_listener` on the same type will do nothing.

```rust
# extern crate yewdux;
# use std::rc::Rc;
# use yewdux::prelude::*;
# #[derive(Default, PartialEq, Debug)]
# struct State {
#     count: u32,
# }
# struct StateLogger;
# impl Listener for StateLogger {
#     // Here's where we say which store we want to subscribe to.
#     type Store = State;
#
#     fn on_change(&self, _cx: &yewdux::Context, state: Rc<Self::Store>) {
#         yewdux::log::info!("state changed: {:?}", state);
#     }
# }

impl Store for State {
    fn new(cx: &yewdux::Context) -> Self {
        init_listener(|| StateLogger, cx);
        Default::default()
    }

    fn should_notify(&self, other: &Self) -> bool {
        self != other
    }
}
```

## Tracking state

Sometimes it's useful to keep track of how a store has been changing over time. However this should
not be done in the listener itself. Notice `Listener::on_change` takes an immutable reference. This
is necessary because otherwise we start to run into borrowing issues when listeners are triggered
recursively.

To track changes we can instead use a separate store that listens to the store we want to track.

```rust
# extern crate yewdux;
# use std::rc::Rc;
# use yewdux::prelude::*;
# #[derive(Default, PartialEq, Debug)]
# struct State {
#     count: u32,
# }

#[derive(Default, PartialEq, Debug, Store, Clone)]
struct ChangeTracker  {
    count: u32,
}

struct ChangeTrackerListener;
impl Listener for StateLogger {
    type Store = State;

    fn on_change(&self, cx: &yewdux::Context, state: Rc<Self::Store>) {
       let dispatch = Dispatch::<ChangeTracker>::new(cx);
       dipatch.reduce_mut(|state| state.count += 1);
       let count = dispatch.get().count;
       println!("State has changed {} times", count);
    }
}

impl Store for State {
    fn new(cx: &yewdux::Context) -> Self {
        init_listener(|| ChangeTrackerListener, cx);
        Default::default()
    }

    fn should_notify(&self, other: &Self) -> bool {
        self != other
    }
}
