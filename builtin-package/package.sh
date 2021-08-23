#!/usr/bin/env bash
set -o pipefail -o nounset -o errexit

# set current working directory to script directory to run script from everywhere
cd "$(dirname "$0")"
SCRIPT_DIR="$(pwd)"
SCHEDULED="${SCRIPT_DIR}/scheduled"

(
    echo "*** compile scheduled scripts ***"
    cd ../aqua
    npx aqua --no-relay --air -i ./dht-scheduled-scripts.aqua -o "$SCHEDULED"
)
# mv "${SCHEDULED}/dht-scheduled-scripts.clearExpired.air" "clearExpired_86400.air"
# mv "${SCHEDULED}/dht-scheduled-scripts.replicate.air" "replicate_3600.air"

(
    echo "*** copy wasm files ***"
    cd ../service
    cp artifacts/*.wasm "$SCRIPT_DIR"
)

echo "*** done ***"
