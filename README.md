# aqua-dht

Distributed Hash Table (DHT) implementation for the Fluence network.

## Getting started

- Install dependencies

```
rustup toolchain install nightly-2021-03-24-x86_64-unknown-linux-gnu
rustup default nightly-2021-03-24-x86_64-unknown-linux-gnu
rustup target add wasm32-wasi
cargo install +nightly marine
```

- To compile .wasm and generate aqua file

```
./build.sh
```

- To run tests

```
cargo test --release
```

- To deploy service
```
./deploy.sh
```
## Project structure

- Aqua source file is located in `aqua`directory.

- .wasm files are stored  in `artifacts` directory.

## API

Note: all timestamps should be passed as result of `("op" "timestamp_sec")` builtin call.
### Key methods

#### `register_key(key: string, current_timestamp: u64, weight: u32)`

- key is unique
- key owner is `%init_peer_id%`
- return `"key already exists with different peer_id"` if key is already registered by another peer

####`get_key_metadata(key: string, current_timestamp: u64)`

- return `"not found"` if key not exists
- update `timestamp_accessed`

#### `republish_key(key: Key, current_timestamp: u64) `

- register key if not exists
- pick older one in case of conflicts
- `pinned` field is ignored

### Value methods

#### `put_value(key: string, value: string, current_timestamp: u64, relay_id: []string, service_id: []string, weight: u32)`

- key should already be registered
- value's peer_id is `%init_peer_id%`
- each peer can have only one value per key
- `relay_id` and `service_id` should have one element or be empty
- there are hardcoded limit for values per key (20)
- values are prioritized by weight, if the limit is exceeded, the lightest value will be replaced by the heavier one

#### `put_value_relay(key: string, value: string, current_timestamp: u64, relay_id: string, weight: u32)`

- same as `put_value` but `relay_id` is required and `service_id` is omitted

#### `put_host_value(key: string, value: string, current_timestamp: u64, relay_id: []string, service_id: []string, weight: u32)`

- key should already be registered
- value's peer_id is `host_id`
- each peer can have only one value per key
- there are no limits for host values

#### `put_host_value_relay(key: string, value: string, current_timestamp: u64, relay_id: string, weight: u32)`

- same as `put_host_value` but `relay_id` is required and `service_id` is omitted

#### `renew_host_value(key: string, current_timestamp: u64)`

- update `timestamp_created` and `timestamp_accessed` for host value given by `%init_peer_id`

#### `clear_host_value(key: string, current_timestamp: u64)`

- remove host value given by `%init_peer_id`

#### `get_values(key: string, current_timestamp: u64)`

- return list of values for given key

#### `republish_values(key: string, records: []Record, current_timestamp: u64)`

- host values are ignored
- merge with current values by last-write-wins strategy


### Other

#### `evict_stale(current_timestamp: u64)`

- return stale keys and records
- remove stale non-host records, unpinned keys that have no host values

#### `clear_expired(current_timestamp: u64)`

- remove expired keys and values
- an expired key is ignored if it has an actual host value

```
merge(records: [][]Record)
merge_two(a: []Record, b: []Record)
merge_hack_get_values(records: []GetValuesResult)
```

- this methods merge values and return the most recent
