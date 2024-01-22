#!/usr/bin/env bash

set -o errexit -o nounset -o pipefail
set -x

# set current working directory to script directory to run script from everywhere
cd "$(dirname "$0")"

# Build the service
./service/build.sh

DISTRO_TARGET=distro/registry-service
mkdir -p "$DISTRO_TARGET"

cd ./aqua
npm pack
cd -

packed_archive_file_name_pattern="fluencelabs-registry-"
packed_archive_file_name=$(find "./aqua" -type f -name "${packed_archive_file_name_pattern}*")

cd ./aqua-tests
echo "    '@fluencelabs/registry': file:../../.$packed_archive_file_name" >> "./fluence.yaml"
fluence dep i
fluence aqua -i ./spell/spell.aqua --no-relay --air -o "../$DISTRO_TARGET/air"
cd -

cp service/artifacts/registry.wasm service/artifacts/sqlite3.wasm distro/Config.toml "$DISTRO_TARGET"

cd distro
cargo build
