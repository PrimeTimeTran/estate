```sh
rustup toolchain install stable
rustup target add wasm32-unknown-unknown --toolchain stable

cargo +stable build \
  --bin web \
  --no-default-features \
  --features web \
  --target wasm32-unknown-unknown
```
