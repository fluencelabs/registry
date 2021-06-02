#!/bin/bash

IFS=$'\n' read -rd '' -a node_list1 <<<"$(fldist env --env krasnodar)"
IFS=$'\n' read -rd '' -a node_list2 <<<"$(fldist env --env testnet)"
node_list=("${node_list1[@]}" "${node_list2[@]}")

sk=NAB5rGwT4qOEB+6nLQawkTfCOV2eiFSjgQK8bfEdZXY=
for i in "${node_list[@]}"
do
  IFS=$'\n' read -rd '' -a out <<<"$(fldist --node-addr $i new_service --ms artifacts/sqlite3.wasm:sqlite3_cfg.json artifacts/aqua-dht.wasm:aqua-dht_cfg.json -n aqua-dht --sk $sk -v)"
  service_id=$(cut -d ":" -f2- <<< "${out[${#out[@]}-2]}" | xargs)
  echo $service_id deployed
  fldist --node-addr $i --sk $sk -v run_air -p alias.air -d "{\"service\": \"$service_id\", \"alias\": \"aqua-dht\"}"
done
