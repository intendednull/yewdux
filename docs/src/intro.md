# Yewdux

Ergonomic state management for [Yew](https://yew.rs) applications.

This crate tries to provide a dead-simple, ergonomic approach to global state management. It does
**not** try to provide any additional patterns or features which aren't directly related to
accessing or manipulating shared state.

Some key features include:

- **Simple** - the only required trait is [Store](./store.md).
- **Ergonomic** - boilerplate is optional!
- **No-copy** - you have complete control over how state is changed. 
- **Selective** - only render when you need to (see [selectors](./selectors.md)).
- **Context agnostic** - users can create and execute a [dispatch](./dispatch.md) from anywhere.
- **Complete component support** - compatible with both functional and struct components.

## Alternatives

- [Bounce](https://github.com/bounce-rs/bounce) - The uncomplicated Yew State management library
