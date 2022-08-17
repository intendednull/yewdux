# Changing global state with Dispatch

`Dispatch` provides many options for changing state.

## `Dispatch::set` 

Assign the store to the given value.

```rust
dispatch.set(Counter { count: 0 });
```

## `Dispatch::set_callback` 

Generate a callback that will set the store to a given value.

```rust
let onclick = dispatch.set_callback(|_| Counter { count: 0 });
html! {
    <button {onclick}>{"Reset counter"}</button>
}
```

## `Dispatch::reduce` 

Assign the state of the store using a reducer function.

```rust
dispatch.reduce(|counter| Counter { count: counter.count + 1});
```

## `Dispatch::reduce_callback` 

Generate a callback that assigns state using a reducer function.

```rust
let onclick = dispatch.reduce_callback(|counter| Counter { count: counter.count + 1});
html! {
    <button {onclick}>{"Increment (+1)"}</button>
}
```

## `Dispatch::reduce_callback_with` 

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
