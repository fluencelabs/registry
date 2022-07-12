# Registry

- [Registry](#registry)
  - [Overview](#overview)
  - [Why is it important?](#why-is-it-important)
  - [What is it?](#what-is-it)
  - [How to Use it in Aqua](#how-to-use-it-in-aqua)
    - [How to import](#how-to-import)
    - [How to create Resource](#how-to-create-resource)
    - [How to register Provider](#how-to-register-provider)
    - [How to register Node Provider](#how-to-register-node-provider)
    - [How to delete Node Provider](#how-to-delete-node-provider)
    - [How to resolve Providers](#how-to-resolve-providers)
    - [How to execute callback on Providers](#how-to-execute-callback-on-providers)
    - [Notes](#notes)
  - [Use cases](#use-cases)
    - [Services discovery](#services-discovery)
    - [Service high-availability](#service-high-availability)
    - [Subnetwork discovery](#subnetwork-discovery)
    - [Load balancer](#load-balancer)
  - [API](#api)
  - [References](#references)
    - [Learn Aqua](#learn-aqua)

## Overview

There is a lot of services in the network on different peers and should be a mechanism to find and resolve them in runtime without prior knowledge about exact providers. This mechanism gives a flexibility to our solutions in terms of discovery, redundancy and high availability.

In centralized systems we can have some centralized storage and routing but in p2p decentralized environments this problem becomes more challenging. Registry is our view on the solution for this problem.

## Why is it important?
Why it's important in our context? If there's prior art, why not use it? Can we live without it? Why not to use a key/value store.

- if you have multiple replicas of one service on different peers you should have the ability to discover services in runtime without predefined knowledge which peer is serving this service if some peer joined/disconnected

Scalability, redundancy, high-availability, etc are essential parts of decentralized system but they are not available out-of-the-box. To enable these, informantion about services should be binded with peers providing them. Also these networks are frequently changing and this information should be resolvable in runtime to provide unstoppable access. So you should have some decentralized protocol to update and resolve information about global and local routing.

## What is it?

Registry is a builtin service which provides service advertisement and discovery. This component creates relationships between unique identifiers and groups of services on various peers. So service providers can join or disconnect during runtime and can be discoverable in the network. **picture with resolving group of services by peer_id/service_id and by resource_id**

Registry is not a plain KV-storage. It is a composition of the Registry service for each network participant and the scheduled scripts which maintain replication, garbage collection, and sustainability.

If you wanna discover in runtime the group of services on different peers without prior knowledge you should register a **Resource.**

A resource should be understood as a group of services or a group of peers united by some common feature.

A combination of service_id and peer_id should be understood as a service **Provider**. **Picture with resource and provider**

There are two types of providers depends on which peer this service operates. If this is full-featured Rust node record lifecycle controlled by node (with scheduled scripts), if this is JS peer/client record lifecycle management should be additionally implemented (it should renew the record every 24 hours).
However if resource and records have been accessed recently it will not be garbage-collected for the next 24 hours from the last access.

For now every resource limited by [number](./service/src/defaults.rs#25) of providers `32` it can hold, not considering record for services providing by this node. So local services have no limitation for registration in local registry. Other providers records are ranked by the weight of peers in the local [TrustGraph](https://github.com/fluencelabs/trust-graph/blob/master/README.md#what-is-it) instance.

For now there are no permissions checking but later an owner of the resource will provide a challenge to check.

## How to Use it in Aqua

### How to import
```rust
import "@fluencelabs/registry/resources-api.aqua"
import "@fluencelabs/registry/registry-service.aqua"

func my_function(resource_id: string) ->  []Record, *Error:
    result, error <- resolveProviders(resource_id)
    <- result, error
```

### How to create Resource
- `createResource(label: string) -> ?ResourceId, *Error`
- `createResourceAndRegisterProvider(label: string, value: string, service_id: ?string) -> ?ResourceId, *Error`
- `createResourceAndRegisterNodeProvider(provider_node_id: PeerId, label: string, value: string, service_id: ?string) -> ?ResourceId, *Error`

Let's register a resource with label `sample` by `INIT_PEER_ID`:
```rust
func my_resource() -> ?ResourceId, *Error:
    id, error <- createResource("sample")
    <- id, error
```

- `createResourceAndRegisterProvider` and `createResourceAndRegisterNodeProvider` are the combination of resource creation and provider registration
- `label` is a unique string for this peer id
- creation is successful if resource id returned
- `*Error` accumulates errors from all affected peers
### How to register Provider
- `registerProvider(resource_id: ResourceId, value: string, service_id: ?string) -> bool, *Error`
- `createResourceAndRegisterProvider(label: string, value: string, service_id: ?string) -> ?ResourceId, *Error`


Let's register local service `greeting` and pass some random string like `hi` as value:
```rust
func register_local_service(resource_id: string) -> ?bool, *Error:
    success, error <- registerProvider(resource_id, "hi", ?[greeting])
    <- success, error
```

- `value` is a user-defined string which can be used at the discretion of the user
- to update provider record you should register it one more time to create record with newer timestamp
- to remove provider you should stop updating it
- you should renew record every 24 hours to keep provider available

### How to register Node Provider
- `registerNodeProvider(provider_node_id: PeerId, resource_id: ResourceId, value: string, service_id: ?string) -> bool, *Error`
- `createResourceAndRegisterNodeProvider(provider_node_id: PeerId, label: string, value: string, service_id: ?string) -> ?ResourceId, *Error`

Let's register service `echo` hosted on `peer_id` and pass some random string like `sample` as value:
```rust
func register_external_service(resource_id: string, peer_id: string) -> ?bool, *Error:
    success, error <- registerNodeProvider(peer_id, resource_id, "hi", ?[greeting])
    <- success, error
```

- record will not be garbage-collected from provider's node but it better to update it every 24 hours, in the following updates renewing process will be handled by the node with scheduled scripts

### How to delete Node Provider
- `removeNodeFromProviders(provider_node_id: PeerId, resource_id: ResourceId)`
Let's remove node provider's record from target node:
```rust
func stop_provide_external_service(resource_id: string, peer_id: string):
    removeNodeFromProviders(peer_id, resource_id)
```

- it will be removed from target node and maximum in 24 hours from the network
### How to resolve Providers
- `resolveProviders(resource_id: ResourceId, ack: i16) -> []Record, *Error`

Let's resolve all providers of our resource_id:
```rust
func get_my_providers(resource_id: string, consistency_level: i16) -> []Record, *Error:
    providers, error <- resolveProviders(resource_id, consistency_level)
    <- providers, error
```

- `ack` is a characteristics which represents min number of peers which asked for known providers

### How to execute callback on Providers
- `executeOnProviders(resource_id: ResourceId, ack: i16, call: Record -> ()) -> *Error`


```rust
func call_provider(p: Record):
    -- topological move to provider via relay
    on p.peer_id via p.relay_id:
        -- resolve and call your service on a provider
        ...
        Op.noop()

-- call on every provider
func call_everyone(resource_id: String, ack: i16):
    executeOnProviders(resource_id, ack, call_provider)
```

- it is just a combination of `resolveProviders` and `for` loop through records with callback execution
- it can be useful in case of broadcasting events on providers
- look in the [docs](https://doc.fluence.dev/aqua-book/libraries/registry#call-a-function-on-resource-providers) for more detailed example
### Notes
You can redefine [`REPLICATION_FACTOR`](https://github.com/fluencelabs/registry/blob/main/aqua/resources-api.aqua#L10) and [`CONSISTENCY_LEVEL`](https://github.com/fluencelabs/registry/blob/main/aqua/resources-api.aqua#L11).


## Use cases
### Services discovery
Discover services without prior knowledge about exact peers and service identifiers.

### Service high-availability
Service provided by several peers still will be available for the client in case of disconnections and other providers' failures.


### Subnetwork discovery
You can register a group of peers for a resource (without specifying exact services). So you "tagging" and grouping the nodes to create a subnetwork.

### Load balancer
If you have a list of service providers which updated in runtime you can create load balancing service based on your preferred metrics.

## API

API is defined in the [resources-api.aqua](./aqua/resources-api.aqua) module. API Reference soon will be available in the documentation.


## References
- See [Registry documentation](https://fluence.dev/aqua-book/libraries/registry).

### Learn Aqua

* [Aqua Book](https://fluence.dev/aqua-book/)
* [Aqua Playground](https://github.com/fluencelabs/aqua-playground)
* [Aqua repo](https://github.com/fluencelabs/aqua)
