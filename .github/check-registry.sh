#!/usr/bin/env bash
set -x

PORT=9991
PEERS=(
  "12D3KooWBM3SdXWqGaawQDGQ6JprtwswEg3FWGvGhmgmMez1vRbR"
  "12D3KooWQdpukY3p2DhDfUfDgphAqsGu5ZUrmQ4mcHSGrRag6gQK"
  "12D3KooWRT8V5awYdEZm6aAV9HWweCEbhWd7df4wehqHZXAB7yMZ"
  "12D3KooWBzLSu9RL7wLP6oUowzCbkCj2AGBSXkHSJKuq4wwTfwof"
  "12D3KooWBf6hFgrnXwHkBnwPGMysP3b1NJe5HGtAWPYfwmQ2MBiU"
  "12D3KooWPisGn7JhooWhggndz25WM7vQ2JmA121EV8jUDQ5xMovJ"
)

cd ${GITHUB_WORKSPACE}/aqua-tests
pwd

for PEER_ID in ${PEERS[@]}; do
  echo "Checking peer ${PEER_ID}"
  if npx aqua remote get_interface --addr /ip4/127.0.0.1/tcp/${PORT}/ws/p2p/${PEER_ID} --id registry | jq -ec 'has("function_signatures")'; then
    exit 1
  else
    ((PORT++))
  fi
done
