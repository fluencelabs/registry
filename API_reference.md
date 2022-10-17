# API Reference

- [API Reference](#api-reference)
  - [Data structures](#data-structures)
    - [Key](#key)
    - [RecordMetadata](#recordmetadata)
    - [Record](#record)
    - [Tombstone](#tombstone)
  - [Resources API](#resources-api)
    - [Overview](#overview)
    - [Return types](#return-types)
    - [Methods](#methods)
      - [createResource](#createresource)
      - [getResource](#getresource)
      - [getResourceId](#getresourceid)
      - [registerService](#registerService)
      - [unregisterService](#unregisterservice)
      - [resolveResource](#resolveresource)
      - [executeOnResource](#executeonresource)

## Data structures
### Key
```rust
data Key {
    -- base58-encoded sha256(concat(label, owner_peer_id))
    id: string,
    -- any unique string defined by the owner
    label: string,
    -- peer id in base58
    owner_peer_id: string,
    -- timestamp of creation in seconds
    timestamp_created: u64,
    -- challenge in bytes, will be used for permissions
    challenge: []u8,
    -- challenge type, will be used for permissions
    challenge_type: string,
    -- encoded and hashed previous fields signed by `owner_peer_id`
    signature: []u8,
}
```

This data structure can be created via [`get_key_bytes`](#get_key_bytes) and [`register_key`](#register_key), and replicated via [`republish_key`](#republish_key). For now, there is no way to remove this structure, it can only be automatically garbage-collected via [`clear_expired`](#clear_expired). In the future updates, key tombstones will be implemented and it would be possible to remove key by an owner.

In terms of Resources API Keys are Resources.
### RecordMetadata
```rust
data RecordMetadata {
    -- base58-encoded key id
    key_id: string,
    -- peer id of the issuer in base58
    issued_by: string,
    -- peer_id of hoster
    peer_id: string,
    -- timestamp in seconds
    timestamp_issued: u64,
    -- will be used for permissions
    solution: []u8,
    -- any user-defined string
    value: string,
    -- optional (length is 0 or 1), base58 relay id
    relay_id: []string,
    -- optional (length is 0 or 1), advertising service id
    service_id: []string,
    -- encoded and hashed previous fields signed by `issued_by`
    issuer_signature: []u8,
}
```

Metadata is the main part of the Record created by issuer that contains routing information, such as optional relay id, peer id and optional service id. Key identifier is a deterministic hash of the `label` and the `owner_peer_id`.

### Record
```rust
data Record {
    -- record metadata
    metadata: RecordMetadata,
    -- timestamp in seconds
    timestamp_created: u64,
    -- encoded and hashed previous fields signed by `metadata.peer_id`
    signature: []u8,
}
```

Record is maintained by `metadata.peer_id` via renewing of `timestamp_created` field automatically with scheduled scripts for full-featured peers and manually for other peers (Note: here and below we mean Rust peers as full-featured and JS/TS as others). Record can be removed by issuing a tombstone or become expired and then garbage-collected. Record owner is `metadata.issued_by`.

### Tombstone
```rust
data Tombstone {
    -- base58-encoded key id
    key_id: string,
    -- peer id of the issuer in base58
    issued_by: string,
    -- peer_id of hoster
    peer_id: string,
    -- timestamp in seconds
    timestamp_issued: u64,
    -- will be used for permissions
    solution: []u8,
    -- encoded and hashed previous fields signed by `issued_by`
    issuer_signature: []u8,
}
```

Tombstone is a special type of record that can be issued by record owner which eventually will substitute record with lower `timestamp_issued`. Tombstones replicated alongside with keys and records and live long enough to be sure that certain records will be deleted. Tombstones are garbage-collected automatically.

In Resources API [`unregisterService`](#unregisterservice) method creates Tombstone.

## Resources API
### Overview
Resources API is a high-level API for Registry network protocol. It uses Kademlia for the discovery of resources and service records. Resource and corresponding service Records are identified by Resource ID, and can be found in Registry services on peers in the Kademlia neighborhood of this Resource ID.

### Return types
```
alias ResourceId: string
alias Resource: Key
alias Error: string
```

Notes:
- ResourceId is also an alias for key id in Resources API terminology.
- Every method (except getResourceId) returns a stream of errors from different peers but success of execution is defined by first part of returned values. Optional types should be checked against `nil` to determine success of execution.

### Methods
#### createResource
```rust
func createResource(label: string) -> ?ResourceId, *Error:
```

Creates Resource with `label` and `INIT_PEER_ID` as owner.
#### getResource
```rust
func getResource(resource_id: ResourceId) -> ?Resource, *Error:
```
Returns resource by corresponding `resource_id`.

#### getResourceId
```rust
func getResourceId(label: string, peer_id: string) -> ResourceId:
```

Returns a deterministic hash of the `label` and the `peer_id`.
#### registerService
```rust
func registerService(
    resource_id: ResourceId,
    value: string,
    peer_id: PeerId,
    service_id: ?string
) -> bool, *Error:
```

Registers Record issued by `INIT_PEER_ID` for service on `peer_id`. `value` is any user-defined string.
#### unregisterService
```rust
func unregisterService(resource_id: ResourceId, peer_id: PeerId) -> bool, *Error:
```

Prevents the record issued by `INIT_PEER_ID` from being renewed and eventually removed.
#### resolveResource
```rust
func resolveResource(resource_id: ResourceId, ack: i16) -> ?[]Record, *Error:
```

Returns all records registered by this `resource_id`. `ack` is a minimal number of polled peers.

#### executeOnResource
```rust
func executeOnResource(resource_id: ResourceId, ack: i16, call: Record -> ()) -> bool, *Error:
```

Resolves all records by given `resource_id` and execites in parallel given callback.