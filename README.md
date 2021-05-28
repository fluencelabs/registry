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

```~~~~
register_key(key: string, current_timestamp: u64)
get_key_metadata(key: string, current_timestamp: u64)

# used for replication
republish_key(key: Key, current_timestamp: u64) 
```

### Value methods
```
# key should already be registered, each peer_id has its own value for the key
put_value(key: string, value: string, current_timestamp: u64, relay_id: []string, service_id: []string)
put_value_relay(key: string, value: string, current_timestamp: u64, relay_id: string)

# return list of values for given key
get_values(key: string, current_timestamp: u64)

# used for replication
republish_values(key: string, records: []Record, current_timestamp: u64)
```

### Other
```
# clear locally and return keys and values older than 1 hour for republishing
evict_stale(current_timestamp: u64)

# clear values and keys older than 24 hours
clear_expired(current_timestamp: u64)
```


```
# this methods merge values and return the most recent
merge(records: [][]Record)
merge_two(a: []Record, b: []Record)
merge_wrapped(records: [][][]Record)
merge_hack(records: [][]Record, hack: string)
merge_hack_get_values(records: []GetValuesResult)
merge_hack_struct(records: RecordsStruct)
```
