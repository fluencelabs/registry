#!/bin/bash

IFS=$'\n' read -rd '' -a node_list <<<"$(fldist env --env krasnodar)"

for i in "${node_list[@]}"
do
  IFS=$'\n' read -rd '' -a out <<<"$(fldist --node-addr $i new_service --ms artifacts/sqlite3.wasm:sqlite3_cfg.json artifacts/aqua-dht.wasm:aqua-dht_cfg.json -n aqua-dht --seed 4VzczoMt73wVenKyJhZ6MySsT8jCy1MXKC6kD8vvgDoK -v)"
  service_id=$(cut -d ":" -f2- <<< "${out[${#out[@]}-2]}" | xargs)
  echo $service_id deployed
  fldist --node-addr $i --seed 4VzczoMt73wVenKyJhZ6MySsT8jCy1MXKC6kD8vvgDoK -v run_air -p alias.air -d "{\"service\": \"$service_id\", \"alias\": \"aqua-dht\"}"
done
