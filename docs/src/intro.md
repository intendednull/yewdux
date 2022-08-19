# Introduction

A state management solution for the [Yew](https://yew.rs) front-end library.

This crate was inspired by [Redux](https://redux.js.org/), however some deviation was taken in
the spirit of Rust.

This book is currently in development. If it is confusing in any way, or you have suggestions,
please post an issue in the [repo](https://github.com/intendednull/yewdux) or ask in the
[Yew discord](https://discord.gg/UmS6FKYa5a).

## Why Yewdux?

State management in Yew can be difficult. Especially when many different components need access to
the same state. Properties and callbacks work great for simple relationships, however quickly become
cumbersome when you need to propagate state through many (potentially isolated) layers of
components. Yew's [context manager](https://yew.rs/docs/concepts/contexts) does a decent job, and is
worth serious consideration, however it requires substantial boilerplate and is not that easy to
use.

This crate aims to provide a dead-simple, ergonomic approach to global state management. It
encourages modular state by providing easy setup and access to your shared state, allowing you to
write cleaner code while remaining productive.

It does **not** try to provide any additional patterns or features which aren't directly related to
accessing or manipulating shared state.

Yewdux was built with the following goals:

- **Simple** - the only required trait is [Store](./store.md).
- **Ergonomic** - boilerplate is optional!
- **Predictable** - you have complete control over how state is changed.
- **Selective** - only render when you need to (see [selectors](./reading.md#selectors)).
- **Context agnostic** - you can create and execute a [dispatch](./dispatch.md) from anywhere.
- **Complete component support** - compatible with both functional and struct components.

## Alternatives

- [Bounce](https://github.com/bounce-rs/bounce) - The uncomplicated Yew State management library
