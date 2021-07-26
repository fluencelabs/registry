#!/usr/bin/env bash
set -o errexit -o nounset -o pipefail

# set current working directory to script directory to run script from everywhere
cd "$(dirname "$0")"

# build aqua-dht.wasm
cargo update
marine build --release

# copy .wasm to artifacts
rm -f artifacts/*
mkdir -p artifacts
cp target/wasm32-wasi/release/aqua-dht.wasm artifacts/

# generate Aqua bindings
marine aqua artifacts/aqua-dht.wasm -s AquaDHT -i aqua-dht >../aqua/src/dht.aqua
