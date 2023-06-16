#!/usr/bin/env bash
set -o pipefail -o nounset -o errexit

# set current working directory to script directory to run script from everywhere
cd "$(dirname "$0")"
PACKAGE_DIR="$(pwd)/../registry"
SCHEDULED="$PACKAGE_DIR/scheduled"

(
    rm -rf $PACKAGE_DIR
    mkdir -p $PACKAGE_DIR
)
(
    echo "*** compile scheduled scripts ***"
    cd ../aqua
    npx fluence --version
    npx fluence aqua --no-relay --air -i ./registry-scheduled-scripts.aqua -o "$SCHEDULED"
)

(
    echo "*** copy wasm files ***"
    cd ../service
    cp artifacts/*.wasm "$PACKAGE_DIR"
)

REGISTRY_CID=$(ipfs add -q --only-hash --cid-version=1 --chunker=size-262144 $PACKAGE_DIR/registry.wasm)
SQLITE_CID=$(ipfs add -q --only-hash --cid-version=1 --chunker=size-262144 $PACKAGE_DIR/sqlite3.wasm)
mv $PACKAGE_DIR/registry.wasm "$PACKAGE_DIR"/"$REGISTRY_CID".wasm
mv $PACKAGE_DIR/sqlite3.wasm "$PACKAGE_DIR"/"$SQLITE_CID".wasm
cp registry_config.json "$PACKAGE_DIR"/"$REGISTRY_CID"_config.json
cp sqlite3_config.json "$PACKAGE_DIR"/"$SQLITE_CID"_config.json

# write blueprint.json
echo "{}" | jq --arg registry_cid "$REGISTRY_CID" --arg sqlite_cid "$SQLITE_CID" '{"name": "registry", "dependencies":[{"/":$sqlite_cid},{"/":$registry_cid}]}' > "$PACKAGE_DIR/blueprint.json"


(
    echo "*** create builtin distribution package ***"
    cd ..
    tar -f registry.tar.gz -zcv ./registry
)

echo "*** done ***"
