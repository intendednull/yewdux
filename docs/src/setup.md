# Setup

Add Yewdux to your project's `Cargo.toml`. Make sure Yew has the "csr" feature (client side rendering):

### Stable release:

```toml
[dependencies]
yew = { version = "0.21", features = ["csr"] }
yewdux = "0.10"
```

### Development branch:

```toml
[dependencies]
yew = { version = "0.22", features = ["csr"] }
yewdux = { git = "https://github.com/intendednull/yewdux.git" }
```
