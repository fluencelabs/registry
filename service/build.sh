#!/usr/bin/env bash
set -o errexit -o nounset -o pipefail

# set current working directory to script directory to run script from everywhere
cd "$(dirname "$0")"

# build registry.wasm
marine build --release

# copy .wasm to artifacts
rm -f artifacts/*
mkdir -p artifacts
cp target/wasm32-wasi/release/registry.wasm artifacts/

# download SQLite 3 to use in tests
curl -L https://github.com/fluencelabs/sqlite/releases/download/sqlite-wasm-v0.18.2/sqlite3.wasm -o artifacts/sqlite3.wasm

# generate Aqua bindings
marine aqua artifacts/registry.wasm -s Registry -i registry >../aqua/registry-service.aqua

mkdir -p ../distro/registry-service
cp artifacts/registry.wasm artifacts/sqlite3.wasm ../distro/registry-service
