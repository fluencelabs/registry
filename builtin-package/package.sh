#!/usr/bin/env bash
set -o pipefail -o nounset -o errexit

# set current working directory to script directory to run script from everywhere
cd "$(dirname "$0")"
SCRIPT_DIR="$(pwd)"
SCHEDULED="${SCRIPT_DIR}/scheduled"

(
    echo "*** compile scheduled scripts ***"
    cd ../aqua
    npx aqua --no-relay --air -i ./registry-scheduled-scripts.aqua -o "$SCHEDULED"
)

(
    echo "*** copy wasm files ***"
    cd ../service
    cp artifacts/*.wasm "$SCRIPT_DIR"
)

(
    echo "*** create builtin distribution package ***"
    cd ..
    mv builtin-package registry
    tar --exclude="package.sh" -f registry.tar.gz -zcv ./registry
    mv registry builtin-package
)

echo "*** done ***"
