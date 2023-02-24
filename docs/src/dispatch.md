# Dispatch

A [Dispatch](https://docs.rs/yewdux/latest/yewdux/dispatch/struct.Dispatch.html) is the primary
interface to [Store](https://docs.rs/yewdux/latest/yewdux/store/trait.Store.html). It is used to
read and write changes to state in various ways.

# Creating a Dispatch

To create a dispatch, you need only provide the desired store type.

```rust
let dispatch = Dispatch::<Counter>::global();
```

A dispatch is also given when using the functional hook.

```rust
let (state, dispatch) = use_store::<Counter>();
```

# Changing state

`Dispatch` provides many options for changing state.

### `Dispatch::set`

Assign the store to the given value.

```rust
dispatch.set(Counter { count: 0 });
```

### `Dispatch::set_callback`

Generate a callback that will set the store to a given value.

```rust
let onclick = dispatch.set_callback(|_| Counter { count: 0 });
html! {
    <button {onclick}>{"Reset counter"}</button>
}
```

### `Dispatch::reduce`

Assign the state of the store using a reducer function.

```rust
dispatch.reduce(|counter| Counter { count: counter.count + 1});
```

### `Dispatch::reduce_callback`

Generate a callback that assigns state using a reducer function.

```rust
let onclick = dispatch.reduce_callback(|counter| Counter { count: counter.count + 1});
html! {
    <button {onclick}>{"Increment (+1)"}</button>
}
```

### `Dispatch::reduce_callback_with`

Similar to `Dispatch::reduce_callback`, but also includes the fired event.

```rust
let onchange = dispatch.reduce_callback_with(|counter, e: Event| {
    let input = e.target_unchecked_into::<HtmlInputElement>();

    if let Ok(count) = input.value().parse() {
        Counter { count }.into()
    } else {
        counter
    }
});

html! {
    <input placeholder="Set counter" {onchange} />
}
```

# Mutation with less boilerplate (CoW)

There are `_mut` variants to every reducer function. This way has less boilerplate, and requires
your `Store` to implement `Clone`.

### `Dispatch::reduce_mut`

```rust
dispatch.reduce_mut(|counter| counter.count += 1);
```

### `Dispatch::reduce_mut_callback`

```rust
let onclick = dispatch.reduce_mut_callback(|counter| counter.count += 1);
html! {
    <button {onclick}>{"Increment (+1)"}</button>
}
```

### `Dispatch::reduce_mut_callback_with`

```rust
let onchange = dispatch.reduce_mut_callback_with(|counter, e: Event| {
    let input = e.target_unchecked_into::<HtmlInputElement>();

    if let Ok(val) = input.value().parse() {
        counter.count = val;
    }
});

html! {
    <input placeholder="Set counter" {onchange} />
}
```

# Mutate state predictably

Yewdux supports predictable mutation. Simply define your message and apply it.

```rust
struct Msg {
    AddOne,
}

impl Reducer<Counter> for Msg {
    fn apply(self, counter: Rc<Counter>) -> Rc<Counter> {
        match self {
            Msg::AddOne => Counter { count: counter.count + 1 },
        }
    }
}
```

### Tip

`Rc::make_mut` is handy if you prefer CoW:

```rust
impl Reducer<Counter> for Msg {
    fn apply(self, mut counter: Rc<Counter>) -> Rc<Counter> {
        let state = Rc::make_mut(&mut counter);

        match self {
            Msg::AddOne => state.count += 1,
        };

        counter
    }
}
```


### `Dispatch::apply`

Execute immediately.

```rust
dispatch.apply(Msg::AddOne);
```

### `Dispatch::apply_callback`

Generate (you guessed it) a callback.

```rust
let onclick = dispatch.apply_callback(|_| Msg::AddOne);
html! {
    <button {onclick}>{"Increment (+1)"}</button>
}
```

# Future support

Because a `Dispatch` may be created and executed from anywhere, Yewdux has innate future support.
Just use it normally, no additonal setup is needed.

```rust
yew::platform::spawn_local(async {
    let user = get_user().await;
    Dispatch::<User>::global().set(user);
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
Rust's type system, and should be phased out in the future.

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
