# Setting default store values

The best way to define the default value of your store is by manually implementing `Default`.

```rust
#[derive(PartialEq, Store)]
struct MyStore {
    foo: String,
    bar: String,
}

impl Default for MyStore {
    fn default() -> Self {
        Self {
            foo: "foo".to_string(),
            bar: "bar".to_string(),
        }
    }
}

```

However sometimes you may need additional context to set the initial value of your store. To do
this, there are a couple options.

You can set the value at the beginning of your application, before your app renders (like in your
main fn).

```rust
fn main() {
    // .. other setup logic and whatnot
    Dispatch::<MyStore>::new().set(MyStore { ... });
    // ... now you can render your app!
}
```

Or inside some component using `use_effect_with_deps`, provided deps of `()`. Be sure to keep it in
a root component!

```rust
use_effect_with_deps(
    move || {
        // .. other setup logic and whatnot
        Dispatch::<MyStore>::new().set(MyStore { ... });
        || {}
    },
    (),
);
```

Keep in mind your store will still be initialized with `Store::new` (usually that's set to
`Default::default()`), however, because Rust is awesome, no fields are allocated initially, and
overwriting is very cheap.
