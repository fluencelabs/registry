#!/usr/bin/env bash
set -o errexit -o nounset -o pipefail

# set current working directory to script directory to run script from everywhere
cd "$(dirname "$0")"

# Build the service
./service/build.sh

DISTRO_TARGET=distro/registry-service
mkdir -p "$DISTRO_TARGET"

npx aqua --no-relay --air -i ./aqua/registry-scheduled-scripts.aqua -o "$DISTRO_TARGET/air"

cp service/artifacts/registry.wasm service/artifacts/sqlite3.wasm service/Config.toml "$DISTRO_TARGET"

cd distro
cargo build
