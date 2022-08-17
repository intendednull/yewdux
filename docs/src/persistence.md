# Persistence

Yewdux provides a macro to easily persist your state in either local or session storage.

```rust
use yewdux::prelude::*;
use serde::{Serialize, Deserialize};

#[derive(Default, PartialEq, Serialize, Deserialize, Store)]
#[store(storage = "local")] // can also be "session"
struct Counter {
    count: u32,
}
```

You can also implement it yourself.

```rust
use yewdux::{prelude::*, storage};

impl Store for Counter {
    fn new() -> Self {
        init_listener(storage::StorageListener::<Self>::new(storage::Area::Local));

        storage::load(storage::Area::Local)
            .expect("Unable to load state")
            .unwrap_or_default()
    }

    fn changed(&self, other: &Self) -> bool {
        self != other
    }
}
```

## Tab sync

Normally if your application is open in multiple tabs, the persistent storage is not updated in any
tab other than the current one. If you want a store to sync in all tabs, add `storage_tab_sync` to
the macro.

```rust
#[derive(Default, Clone, PartialEq, Eq, Deserialize, Serialize, Store)]
#[store(storage = "local", storage_tab_sync)]
struct State {
    count: u32,
}
```

