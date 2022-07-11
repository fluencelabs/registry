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
```
import "@fluencelabs/registry/resources-api.aqua"
import "@fluencelabs/registry/registry-service.aqua"

func my_function(resource_id: string) ->  []Record, *Error:
    on HOST_PEER_ID:
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
    on HOST_PEER_ID:
        id, error <- createResource("sample")
    <- id, error
```

- `createResourceAndRegisterProvider` and `createResourceAndRegisterNodeProvider` are the combination of resource creation and provider registration
- `label` is a unique string for this peer id
- creation is successful if resource id returned
- `*Error` accumulates errors from all affected peers
### How to register Provider

to update provider ...
to remove provider ...
### How to register Node Provider

### How to delete Node Provider

### How to resolve Providers

### How to execute callback on Providers


### Notes
You can redefine [`REPLICATION_FACTOR`](https://github.com/fluencelabs/registry/blob/main/aqua/resources-api.aqua#L10) and [`CONSISTENCY_LEVEL`](https://github.com/fluencelabs/registry/blob/main/aqua/resources-api.aqua#L11).


## Use cases
See [example](./example):
- How to call [`registry`](./example/src/example.ts) function in TS/JS
- Writing an Aqua script using `registry`: [event_example.aqua](./example/src/aqua/event_example.aqua)

## API

API is defined in the [resources-api.aqua](./aqua/resources-api.aqua) module. API Reference soon will be available in the documentation.


## References
- See [Registry documentation](https://fluence.dev/aqua-book/libraries/registry).

### Learn Aqua

* [Aqua Book](https://fluence.dev/aqua-book/)
* [Aqua Playground](https://github.com/fluencelabs/aqua-playground)
* [Aqua repo](https://github.com/fluencelabs/aqua)
