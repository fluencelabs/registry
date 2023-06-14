#!/usr/bin/env bash
set -o errexit -o nounset -o pipefail

# set current working directory to script directory to run script from everywhere
cd "$(dirname "$0")"

# Build the service
./service/build.sh

DISTRO_TARGET=distro/registry-service
mkdir -p "$DISTRO_TARGET"

cd ./aqua
npx fluence aqua --no-relay --air -i ../spell/spell.aqua -o "../$DISTRO_TARGET/air"
cd -

cp service/artifacts/registry.wasm service/artifacts/sqlite3.wasm distro/Config.toml "$DISTRO_TARGET"

cd distro
cargo build
