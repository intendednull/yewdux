# Future support

Because a `Dispatch` may be created and executed from anywhere, Yewdux has innate future support.
Just use it normally, no additonal setup is needed.

```rust
yew::platform::spawn_local(async {
    let user = get_user().await;
    Dispatch::<User>::new().set(user);
})
```

## Async associated functions 
For stores that have async methods, dispatch provides some options for your convenience.

### `Dispatch::reduce_future` 

Executes immediately.

```rust
dispatch
    .reduce_future(|state| async move {
        let mut state = state.as_ref().clone();
        state.update_user().await;

        state
    })
    .await;
```

### `Dispatch::reduce_mut_future` 

For the `CoW` approach. Note `Box::pin` is required here. This is due to a current limitation of
Rust, and should be phased out in the future.

```rust
dispatch
    .reduce_mut_future(|state| {
        Box::pin(async move {
            state.update_user().await;
        })
    })
    .await;
```

## Async callbacks 

You can also create callbacks that execute a future when called. Note these are simple wrappers over
`yew::platform::spawn_local`.

### `Dispatch::reduce_future_callback`

```rust
let cb = dispatch.reduce_future_callback(|state| async move {
    let mut state = state.as_ref().clone();
    state.update_user().await;

    state
});
```

### `Dispatch::reduce_mut_future_callback` 

```rust
let cb = dispatch.reduce_mut_future_callback(|state| {
    Box::pin(async move {
        state.update_user().await;
    })
});
```
