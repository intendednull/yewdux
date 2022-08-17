# Mutation with less boilerplate (CoW)

There are `_mut` variants to every reducer function. This way has less boilerplate, and requires
your `Store` to implement `Clone`.

## `Dispatch::reduce_mut`

```rust
dispatch.reduce_mut(|counter| counter.count += 1);
```

## `Dispatch::reduce_mut_callback`

```rust
let onclick = dispatch.reduce_mut_callback(|counter| counter.count += 1);
html! {
    <button {onclick}>{"Increment (+1)"}</button>
}
```

## `Dispatch::reduce_mut_callback_with`

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
