# Yewdux Development Guide

## Build Commands
- Build all crates: `cargo build`
- Build specific crate: `cargo build -p yewdux`
- Run tests (for non-WASM target): `cargo test --target x86_64-unknown-linux-gnu`
- Run tests without doctests: `cargo test --lib --no-default-features --target x86_64-unknown-linux-gnu`
- Run specific test: `cargo test test_name --target x86_64-unknown-linux-gnu`
- Run example: `cd examples/[example_name] && trunk serve`
- Build documentation: `cd docs && mdbook build`

## Code Style
- Use Rust 2021 edition
- Follow standard Rust naming conventions (snake_case for functions, CamelCase for types)
- Format code with `cargo fmt`
- Fix lints with `cargo clippy`
- Derive `Clone`, `PartialEq` for all Store implementations
- Add unit tests for new functionality
- Document public APIs with rustdoc comments
- Use thiserror for error handling
- Keep components small and focused on a single responsibility
- Leverage Yew's function components with hooks pattern

## Project Structure
- Core library: `crates/yewdux/src/`
- Macros: `crates/yewdux-macros/src/`
- Examples demonstrate usage patterns in `examples/`