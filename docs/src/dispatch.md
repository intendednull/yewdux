# Creating a Dispatch

`Dispatch` provides an interface to your `Store`. To create one you need only provide the store
type.

```rust
let dispatch = Dispatch::<Counter>::new();
```

A dispatch is also given when using the functional hook.

```rust
let (state, dispatch) = use_store::<Counter>();
```
