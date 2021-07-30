# Aqua DHT service

Rust implementation of the AquaDHT service.

## How to build .wasm
* Install dependencies

```bash
rustup toolchain install nightly-2021-03-24-x86_64-unknown-linux-gnu
rustup default nightly-2021-03-24-x86_64-unknown-linux-gnu
rustup target add wasm32-wasi
cargo install +nightly marine
```

* Compile compile .wasm and generate aqua file

```bash
./build.sh
```

## How to run tests
```bash
cargo test --release
```
