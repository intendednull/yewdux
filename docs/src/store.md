# Defining a Store

A [Store](https://docs.rs/yewdux/0.8.1/yewdux/store/trait.Store.html) represents state that is
shared application-wide. It is initialized on first access, and lives for application lifetime.

Implement `Store` for your state using the macro.

```rust
#[derive(Default, PartialEq, Store)]
struct Counter {
    count: u32,
}
```

It is also simple to define a `Store` manually. This is useful when you need finer control over how
it is created, or when to notify components.

```rust
#[derive(PartialEq)]
struct Counter {
    count: u32,
}

impl Store for Counter {
    fn new(_cx: &Context) {
        Self {
            count: Default::default(),
        }
    }

    fn should_notify(&self, old: &Self) -> bool {
        // When this returns true, all components are notified and consequently re-render.
        self != old
    }
}
```

*Note: implementing `Store` doesn't require any additional traits, however `Default` and
`PartialEq` are required for the macro.*
