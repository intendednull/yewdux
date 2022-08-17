# Defining a Store

A `Store` represents state that is shared application-wide. It is initialized on first access, and
lives for application lifetime.

Implement `Store` for your state using the macro.

```rust
#[derive(Default, PartialEq, Store)]
struct Counter {
    count: u32,
}
```

Or do it manually.

```rust
#[derive(PartialEq)]
struct Counter {
    count: u32,
}

impl Store for Counter {
    fn new() {
        Self {
            count: Default::default(),
        }
    }

    fn should_notify(&self, other: &Self) -> bool {
        // When this returns true, all components are notified and consequently re-render.
        self != other
    }
}
```

*Note: implementing `Store` doesn't require any additional traits, however `Default` and
`PartialEq` are required for the macro.*
