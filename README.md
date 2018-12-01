# Ziggurat

[Play it](http://matt-williams.github.io/ziggurat)

This is an attempt at a [Ludum Dare 43](http://ldjam.com/events/ludum-dare/43) entry.

## Build

### Setup

```
# Install Rust
curl https://sh.rustup.rs -sSf | sh

# Install the wasm32-unknown-unknown toolchain to compile to WebAssembly.
rustup update
rustup target add wasm32-unknown-unknown

# Install cargo-web to make it easier to build WebAssembly projects.
cargo install cargo-web
```

### Release build

```
cargo web deploy --release
rm -rf docs/* && cp -R target/deploy/* docs/
```

### Development server

```
cargo web start
```
