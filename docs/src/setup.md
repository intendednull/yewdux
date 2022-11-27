# Setup

Add Yewdux to your project's `Cargo.toml`. Make sure Yew has the "csr" feature (client side rendering):

### Stable release:

```toml
[dependencies]
yew = { version = "0.20", features = ["csr"] }
yewdux = "0.9"
```

### Development branch:

```toml
[dependencies]
yew = { git = "https://github.com/yewstack/yew.git", features = ["csr"] }
yewdux = { git = "https://github.com/intendednull/yewdux.git" }
```


