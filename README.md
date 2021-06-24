# aqua-dht

Distributed Hash Table (DHT) implementation for the Fluence network with Aqua interface.

## Getting started

- Install dependencies

```bash
rustup toolchain install nightly-2021-03-24-x86_64-unknown-linux-gnu
rustup default nightly-2021-03-24-x86_64-unknown-linux-gnu
rustup target add wasm32-wasi
cargo install +nightly marine
```

- To compile .wasm and generate aqua file

```bash
./build.sh
```

- To run tests

```bash
cargo test --release
```

- To deploy service

```bash
./deploy.sh
```

## How to Use

see [PubSub](/npm/pubsub.aqua)

## Deploy As A Builtin Service

see [Tutorials](https://doc.fluence.dev/docs/tutorials_tutorials/add-your-own-builtin)
