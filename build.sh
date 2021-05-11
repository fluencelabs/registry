#!/usr/bin/env bash -o errexit -o nounset -o pipefail

# This script builds all subprojects and puts all created Wasm modules in one dir
cargo update
marine build --release

rm -f artifacts/*
mkdir -p artifacts
cp target/wasm32-wasi/release/aqua-dht.wasm artifacts/
curl -L https://github.com/fluencelabs/sqlite/releases/download/v0.14.0_w/sqlite3.wasm -o artifacts/sqlite3.wasm
