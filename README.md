# aqua-dht

[Distributed Hash Table](https://en.wikipedia.org/wiki/Distributed_hash_table) (DHT) implementation for the Fluence network with an Aqua interface.

## Learn Aqua

* [Aqua Book](https://app.gitbook.com/@fluence/s/aqua-book/)
* [Aqua Playground](https://github.com/fluencelabs/aqua-playground)
* [Aqua repo](https://github.com/fluencelabs/aqua)

## Getting started

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

* Run tests

```bash
cargo test --release
```

* Deploy service

```bash
./deploy.sh
```

## How to Use

See the Aqua [PubSub](./npm/pubsub.aqua) script

## Deploy As A Builtin Service

See [Tutorials](https://doc.fluence.dev/docs/tutorials_tutorials/add-your-own-builtin)