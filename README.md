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
    - [How to execute a callback on Providers](#how-to-execute-a-callback-on-providers)
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

There are many [services](https://doc.fluence.dev/docs/concepts#services) in the network on different peers, and there should be a way to find and resolve these services in runtime without prior knowledge about exact service providers. Such approach gives robustness and flexibility to our solutions in terms of discovery, redundancy and high availability.

In centralized systems, we can have centralized storage and routing, but in p2p decentralized environments, the problem becomes more challenging. Registry is our view on the solution for the problem.
![image](images/registry.png)

## Why is it important?

Scalability, redundancy and high availability are essential parts of a decentralized system, but they are not available out of the box. To enable them, information about services should be bound with peers providing them. Also, such networks are constantly changing, and those changes should be reflected and resolvable in runtime to provide uninterruptible access. So there's a need to have a decentralized protocol to update and resolve information about routing, both global and local.

## What is it?

Registry is available (built-in) on every Fluence node, and it provides service advertisement and discovery. The component allows of creating relationships between unique identifiers and groups of services on various peers. So service providers can either join or disconnect during runtime and be discoverable on the network.

However, Registry is not a plain KV-storage. Instead, it is a composition of the Registry service for each network participant and the scheduled scripts maintaining replication and garbage collection.

So, if you want to discover a group of services on different peers without prior knowledge in runtime, you should register a **Resource**. A resource is a group of services or peers united by some common feature. Please notice that resource lifetime is ~24 hours. However, if the resource has been accessed recently, its lifetime is prolonged, and it will not be garbage-collected for the next 24 hours from the last access.

A combination of `service_id` and `peer_id` represents a service **Provider**.

![image](images/discovery.png)
![image](images/mapping.png)

There are two types of providers depending on a peer a service operates on. **Node Providers** correspond to a full-featured Rust [node](https://doc.fluence.dev/docs/node) and the rest of **Providers** — to a [JS peer/client](https://doc.fluence.dev/docs/fluence-js). And a record for any provider should be renewed every 24 hours to avoid garbage collection.

As for now, every resource is limited by a [number](./service/src/defaults.rs#25) of providers `32` it can hold, disregarding records for the node services. So local services have no limitation for registration in a local registry. Other providers' records are ranked by peer weights in a local [TrustGraph](https://github.com/fluencelabs/trust-graph/blob/master/README.md#what-is-it) instance. Thus every node has a list of the most trusted service providers locally. "Trusted" is a TrustGraph term meaning a service provider complies with requirements defined by a node owner.

There is no permissions management at the moment, but in the coming updates, a resource owner will be able to provide a challenge to check against.

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

Let's register a resource with the label `sample` by `INIT_PEER_ID`:
```rust
func my_resource() -> ?ResourceId, *Error:
   id, error <- createResource("sample")
   <- id, error
```

- `createResourceAndRegisterProvider` and `createResourceAndRegisterNodeProvider` are the combination of resource creation and provider registration
- `label` is a unique string for the peer id
- creation is successful if a resource id is returned
- `*Error` accumulates errors from all the affected peers
### How to register Provider
- `registerProvider(resource_id: ResourceId, value: string, service_id: ?string) -> bool, *Error`
- `createResourceAndRegisterProvider(label: string, value: string, service_id: ?string) -> ?ResourceId, *Error`


Let's register a local service `greeting` and pass a random string `hi` as a value:
```rust
func register_local_service(resource_id: string) -> ?bool, *Error:
   success, error <- registerProvider(resource_id, "hi", ?[greeting])
   <- success, error
```

- `value` is a user-defined string that can be used at the discretion of the user
- to update the provider record, you should register it again to create a record with a newer timestamp
- to remove the provider you should stop updating its record
- you should renew the record every 24 hours to keep the provider available

### How to register Node Provider
- `registerNodeProvider(provider_node_id: PeerId, resource_id: ResourceId, value: string, service_id: ?string) -> bool, *Error`
- `createResourceAndRegisterNodeProvider(provider_node_id: PeerId, label: string, value: string, service_id: ?string) -> ?ResourceId, *Error`

Let's register a service `echo` hosted on `peer_id` and pass a random string like `sample` as a value:
```rust
func register_external_service(resource_id: string, peer_id: string) -> ?bool, *Error:
   success, error <- registerNodeProvider(peer_id, resource_id, "hi", ?[greeting])
   <- success, error
```

- the record will not be garbage-collected from the provider's node, but it is better to update it every 24 hours. In the following updates renewing process will be handled by a node using the scheduled scripts

### How to delete Node Provider
- `removeNodeFromProviders(provider_node_id: PeerId, resource_id: ResourceId)`
Let's remove a node provider's record from a target node:
```rust
func stop_provide_external_service(resource_id: string, peer_id: string):
   removeNodeFromProviders(peer_id, resource_id)
```

- it will be removed from the target node and in 24 hours from the network

### How to resolve Providers
- `resolveProviders(resource_id: ResourceId, ack: i16) -> []Record, *Error`

Let's resolve all the providers of our resource_id:
```rust
func get_my_providers(resource_id: string, consistency_level: i16) -> []Record, *Error:
   providers, error <- resolveProviders(resource_id, consistency_level)
   <- providers, error
```

- `ack` represents a minimal number of peers that requested for known providers

### How to execute a callback on Providers
- `executeOnProviders(resource_id: ResourceId, ack: i16, call: Record -> ()) -> *Error`

```rust
func call_provider(p: Record):
   -- topological move to a provider via relay
   on p.peer_id via p.relay_id:
       -- resolve and call your service on a provider
       ...
       Op.noop()

-- call on every provider
func call_everyone(resource_id: String, ack: i16):
   executeOnProviders(resource_id, ack, call_provider)
```

- it is a combination of `resolveProviders` and a `for` loop through records with the callback execution
- it can be useful in case of broadcasting events on providers

For more detailed example please take a look in the [docs](https://doc.fluence.dev/aqua-book/libraries/registry#call-a-function-on-resource-providers)

### Notes
You can redefine [`REPLICATION_FACTOR`](https://github.com/fluencelabs/registry/blob/main/aqua/resources-api.aqua#L10) and [`CONSISTENCY_LEVEL`](https://github.com/fluencelabs/registry/blob/main/aqua/resources-api.aqua#L11).


## Use cases

### Services discovery
Discover services without prior knowledge about exact peers and service identifiers.

### Service high-availability
A service provided by several peers still will be available for the client in case of disconnections and other providers' failures.

![image](images/availability.png)

### Subnetwork discovery
You can register a group of peers for a resource (without specifying any services). So you "tag" and group the nodes to create a subnetwork.

![image](images/subnetwork.png)

### Load balancer
If you have a list of service providers updated in runtime, you can create a load-balancing service based on your preferred metrics.

## API

API is defined in the [resources-api.aqua](./aqua/resources-api.aqua) module. API Reference will be available in the documentation soon.


## References
- [Registry documentation](https://fluence.dev/aqua-book/libraries/registry).

### Learn Aqua

* [Aqua Book](https://fluence.dev/aqua-book/)
* [Aqua Playground](https://github.com/fluencelabs/aqua-playground)
* [Aqua repo](https://github.com/fluencelabs/aqua)
