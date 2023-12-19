# Persistence

Yewdux provides the `#[store]` macro to easily persist your state in either local or session storage.

```rust
# extern crate yewdux;
# extern crate serde;
use yewdux::prelude::*;
use serde::{Serialize, Deserialize};

#[derive(Default, PartialEq, Serialize, Deserialize, Store)]
#[store(storage = "local")] // can also be "session"
struct State {
    count: u32,
}
```

This can also be done
[manually](https://github.com/intendednull/yewdux/blob/master/examples/listener/src/main.rs).

## Tab sync

Normally if your application is open in multiple tabs, the store is not updated in any tab other
than the current one. If you want storage to sync in all tabs, add `storage_tab_sync` to the macro.

```rust
# extern crate yewdux;
# extern crate serde;
# use yewdux::prelude::*;
# use serde::{Serialize, Deserialize};
#[derive(Default, Clone, PartialEq, Eq, Deserialize, Serialize, Store)]
#[store(storage = "local", storage_tab_sync)]
struct State {
    count: u32,
}
```

## Additional Listeners

You can inject additional listeners into the `#[store]` macro.

```rust
# extern crate yewdux;
# extern crate serde;
# use std::rc::Rc;
# use yewdux::prelude::*;
# use serde::{Serialize, Deserialize};
#[derive(Default, Clone, PartialEq, Eq, Deserialize, Serialize, Store)]
#[store(storage = "local", listener(LogListener))]
struct State {
    count: u32,
}

struct LogListener;
impl Listener for LogListener {
    type Store = State;

    fn on_change(&mut self, _cx: &yewdux::Context, state: Rc<Self::Store>) {
        yewdux::log::info!("Count changed to {}", state.count);
    }
}
```

