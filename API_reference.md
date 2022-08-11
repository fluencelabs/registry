# API Reference

- [API Reference](#api-reference)
  - [Service API](#service-api)
    - [Overview](#overview)
    - [Methods](#methods)
      - [get_key_id](#get_key_id)
      - [get_key_bytes](#get_key_bytes)
      - [register_key](#register_key)
      - [get_key_metadata](#get_key_metadata)
      - [republish_key](#republish_key)
      - [merge_keys](#merge_keys)
      - [get_record_bytes](#get_record_bytes)
      - [put_record](#put_record)
      - [get_host_record_bytes](#get_host_record_bytes)
      - [put_host_record](#put_host_record)
      - [clear_host_record](#clear_host_record)
      - [propagate_host_record](#propagate_host_record)
      - [get_records](#get_records)
      - [evict_stale](#evict_stale)
      - [republish_records](#republish_records)
      - [clear_expired](#clear_expired)
      - [merge, merge_two](#merge-merge_two)
      - [set_stale_timeout](#set_stale_timeout)
      - [set_expired_timeout](#set_expired_timeout)
    - [Data structures](#data-structures)
      - [Key](#key)
      - [Record](#record)
  - [Resources API](#resources-api)
    - [Overview](#overview-1)
    - [Methods](#methods-1)
      - [createResource](#createresource)
      - [createResourceAndRegisterProvider](#createresourceandregisterprovider)
      - [createResourceAndRegisterNodeProvider](#createresourceandregisternodeprovider)
      - [registerProvider](#registerprovider)
      - [registerNodeProvider](#registernodeprovider)
      - [resolveProviders](#resolveproviders)
      - [executeOnProviders](#executeonproviders)

## Service API
### Overview
### Methods
#### get_key_id
```rust
get_key_id(label: string, peer_id: string) -> string
```

#### get_key_bytes
```rust
get_key_bytes(
    label: string,
    owner_peer_id: []string,
    timestamp_created: u64,
    challenge: []u8,
    challenge_type: string
) -> []u8
```

#### register_key
```rust
register_key(
    label: string,
    owner_peer_id: []string,
    timestamp_created: u64,
    challenge: []u8,
    challenge_type: string,
    signature: []u8,
    weight: WeightResult,
    current_timestamp_sec: u64
) -> RegisterKeyResult
```
<!-- omit in toc -->
##### RegisterKeyResult
```rust
data RegisterKeyResult:
  success: bool
  error: string
  key_id: string
```

#### get_key_metadata
```rust
get_key_metadata(key_id: string, current_timestamp_sec: u64) -> GetKeyMetadataResult
```
<!-- omit in toc -->
##### GetKeyMetadataResult
```rust
data GetKeyMetadataResult:
  success: bool
  error: string
  key: Key
```

#### republish_key
```rust
republish_key(key: Key, weight: WeightResult, current_timestamp_sec: u64) -> RegistryResult
```
<!-- omit in toc -->
##### RegistryResult
```rust
data RegistryResult:
  success: bool
  error: string
```

#### merge_keys
```rust
merge_keys(keys: []Key) -> MergeKeysResult
```
<!-- omit in toc -->
##### MergeKeysResult
```rust
data MergeKeysResult:
  success: bool
  error: string
  key: Key
```

#### get_record_bytes
```rust
get_record_bytes(
    key_id: string,
    value: string,
    relay_id: []string,
    service_id: []string,
    timestamp_created: u64,
    solution: []u8
) -> []u8
```

#### put_record
```rust
put_record(
    key_id: string,
    value: string,
    relay_id: []string,
    service_id: []string,
    timestamp_created: u64,
    solution: []u8,
    signature: []u8,
    weight: WeightResult,
    current_timestamp_sec: u64
) -> RegistryResult
```
See [RegistryResult](#registryresult)

#### get_host_record_bytes
```rust
get_host_record_bytes(
    key_id: string,
    value: string,
    relay_id: []string,
    service_id: []stri  ng,
    timestamp_created: u64,
    solution: []u8
) -> []u8
```

#### put_host_record
```rust
put_host_record(
    key_id: string,
    value: string,
    relay_id: []string,
    service_id: []string,
    timestamp_created: u64,
    solution: []u8,
    signature: []u8,
    weight: WeightResult,
    current_timestamp_sec: u64
) -> PutHostRecordResult
```
<!-- omit in toc -->
##### PutHostRecordResult
```rust
data PutHostRecordResult:
  success: bool
  error: string
  record: []Record
```

#### clear_host_record
```rust
clear_host_record(key_id: string, current_timestamp_sec: u64) -> RegistryResult
```
See [RegistryResult](#registryresult)


#### propagate_host_record
```rust
propagate_host_record(
    set_host_value: PutHostRecordResult, current_timestamp_sec: u64,
    weight: WeightResult
) -> RegistryResult
```
See [RegistryResult](#registryresult)

#### get_records
```rust
get_records(key_id: string, current_timestamp_sec: u64) -> GetRecordsResult
```
<!-- omit in toc -->
##### GetRecordsResult
```rust
data GetRecordsResult:
  success: bool
  error: string
  result: []Record
```

#### evict_stale
```rust
evict_stale(current_timestamp_sec: u64) -> EvictStaleResult
```
<!-- omit in toc -->
##### EvictStaleResult
```rust
data EvictStaleItem:
  key: Key
  records: []Record

data EvictStaleResult:
  success: bool
  error: string
  results: []EvictStaleItem
```

#### republish_records
```rust
republish_records(records: []Record, weights: []WeightResult, current_timestamp_sec: u64) -> RepublishRecordsResult
```

<!-- omit in toc -->
##### RepublishRecordsResult
```rust
data RepublishRecordsResult:
  success: bool
  error: string
  updated: u64
```
#### clear_expired
```rust
clear_expired(current_timestamp_sec: u64) -> ClearExpiredResult
```
<!-- omit in toc -->
##### ClearExpiredResult
```rust
data ClearExpiredResult:
  success: bool
  error: string
  count_keys: u64
  count_records: u64
```
#### merge, merge_two
```rust
merge(records: [][]Record) -> MergeResult
merge_two(a: []Record, b: []Record) -> MergeResult
```

<!-- omit in toc -->
##### MergeResult
```rust
data MergeResult:
  success: bool
  error: string
  result: []Record
```

#### set_stale_timeout
```rust
set_stale_timeout(timeout_sec: u64)
```
#### set_expired_timeout
```rust
set_expired_timeout(timeout_sec: u64)
```

### Data structures
#### Key
```rust
data Key:
  id: string
  label: string
  owner_peer_id: string
  timestamp_created: u64
  challenge: []u8
  challenge_type: string
  signature: []u8
```

#### Record
```rust
data Record:
  key_id: string
  value: string
  peer_id: string
  set_by: string
  relay_id: []string
  service_id: []string
  timestamp_created: u64
  solution: []u8
  signature: []u8
```
## Resources API
### Overview
### Methods
#### createResource
```rust
func createResource(label: string) -> ?ResourceId, *Error:
```

#### createResourceAndRegisterProvider
```rust
func createResourceAndRegisterProvider(
    label: string,
    value: string,
    service_id:
    ?string
) -> ?ResourceId, *Error:
```

#### createResourceAndRegisterNodeProvider
```rust
func createResourceAndRegisterNodeProvider(
    provider_node_id: PeerId,
    label: string,
    value: string,
    service_id: ?string
) -> ?ResourceId, *Error:
```

#### registerProvider
```rust
func registerProvider(resource_id: ResourceId, value: string, service_id: ?string) -> bool, *Error:
```

#### registerNodeProvider
```rust
func registerNodeProvider(
    provider_node_id: PeerId,
    resource_id: ResourceId,
    value: string,
    service_id: ?string
) -> bool, *Error:
```


#### resolveProviders
```rust
func resolveProviders(resource_id: ResourceId, ack: i16) -> []Record, *Error:
```

#### executeOnProviders
```rust
func executeOnProviders(resource_id: ResourceId, ack: i16, call: Record -> ()) -> *Error:
```