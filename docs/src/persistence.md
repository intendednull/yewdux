# Persistence

Yewdux provides the `#[store]` macro to easily persist your state in either local or session storage.

```rust
use yewdux::prelude::*;
use serde::{Serialize, Deserialize};

#[derive(Default, PartialEq, Serialize, Deserialize, Store)]
#[store(storage = "local")] // can also be "session"
struct Counter {
    count: u32,
}
```

This can also be done manually.

```rust
use yewdux::{prelude::*, storage};

impl Store for Counter {
    fn new(cx: &yewdux::Context) -> Self {
        init_listener(storage::StorageListener::<Self>::new(storage::Area::Local), cx);

        storage::load(storage::Area::Local)
            .expect("Unable to load state")
            .unwrap_or_default()
    }

    fn should_notify(&self, other: &Self) -> bool {
        self != other
    }
}
```

## Tab sync

Normally if your application is open in multiple tabs, the store is not updated in any tab other
than the current one. If you want storage to sync in all tabs, add `storage_tab_sync` to the macro.

```rust
#[derive(Default, Clone, PartialEq, Eq, Deserialize, Serialize, Store)]
#[store(storage = "local", storage_tab_sync)]
struct State {
    count: u32,
}
```

## Additional Listeners

You can inject additional listeners into the `#[store]` macro.

```rust
#[derive(Default, Clone, PartialEq, Eq, Deserialize, Serialize, Store)]
#[store(storage = "local", listener(LogListener))]
struct State {
    count: u32,
}

struct LogListener;
impl Listener for LogListener {
    type Store = State;

    fn on_change(&mut self, state: Rc<Self::Store>) {
        log!(Level::Info, "Count changed to {}", state.count);
    }
}
```

