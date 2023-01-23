#!/usr/bin/env bash

PEERS=(
  "/ip4/127.0.0.1/tcp/9991/ws/p2p/12D3KooWBM3SdXWqGaawQDGQ6JprtwswEg3FWGvGhmgmMez1vRbR"
  "/ip4/127.0.0.1/tcp/9992/ws/p2p/12D3KooWQdpukY3p2DhDfUfDgphAqsGu5ZUrmQ4mcHSGrRag6gQK"
  "/ip4/127.0.0.1/tcp/9993/ws/p2p/12D3KooWRT8V5awYdEZm6aAV9HWweCEbhWd7df4wehqHZXAB7yMZ"
  "/ip4/127.0.0.1/tcp/9994/ws/p2p/12D3KooWBzLSu9RL7wLP6oUowzCbkCj2AGBSXkHSJKuq4wwTfwof"
  "/ip4/127.0.0.1/tcp/9995/ws/p2p/12D3KooWBf6hFgrnXwHkBnwPGMysP3b1NJe5HGtAWPYfwmQ2MBiU"
  "/ip4/127.0.0.1/tcp/9996/ws/p2p/12D3KooWPisGn7JhooWhggndz25WM7vQ2JmA121EV8jUDQ5xMovJ"
)

cd ${GITHUB_WORKSPACE}/aqua-tests

for PEER_ADDR in ${PEERS[@]}; do
  echo "Checking ${PEER_ADDR}"
  if ! npx aqua remote get_interface --addr ${PEER_ADDR} --id registry | jq -ec 'has("function_signatures")'; then
aqua remote get_interface --addr
    exit 1
  fi
done
