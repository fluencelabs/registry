# Registry

[![npm](https://img.shields.io/npm/v/@fluencelabs/registry)](https://www.npmjs.com/package/@fluencelabs/registry)


## Overview

Registry is an essential part of the [Fluence network](https://fluence.network) protocol. It provides a Resources API that can be used for service advertisement and discovery. Registry is available (built-in) on every Fluence node, and it provides service advertisement and discovery. The component allows creating relationships between unique identifiers and groups of services on various peers, so that service providers can either join or disconnect anytime and be discoverable on the network.

There are many [services](https://doc.fluence.dev/docs/concepts#services) in the network on different peers, and there should be a way to find and resolve these services without prior knowledge about exact identifiers. Such an approach brings robustness and flexibility to our solutions in terms of discovery, redundancy and high availability.

In centralized systems, one can have centralized storage and routing, but in p2p decentralized environments, the problem becomes more challenging. Our solution for the problem is **Registry**, a purpose-driven distributed hash table (DHT), an inherent part of the [Fluence](https://fluence.dev) protocol.

![image](images/registry.png)

However, Registry is not a plain key/value storage. Instead, it is a composition of the Registry service for each network participant and scheduled scripts maintaining replication and garbage collection. Thus, if you want to discover a group of services on different peers without prior knowledge, you should create a **Resource**. A resource is a group of services or peers united by some common feature. Any service is represented by a combination of `service_id` and `peer_id`, it is called a **Record**.

**Why is Registry important?**

Scalability, redundancy and high availability are essential parts of a decentralized system, but they are not available out of the box. To enable them, information about services should be bound with peers providing them. Also, such networks are constantly changing, and those changes should be reflected and resolvable to provide uninterruptible access. So there's a need to have a decentralized protocol to update and resolve information about routing, both global and local.


## Installation and Usage

A complete workflow covering installation of Registry, creating Resources, registering services etc. can be found [here](INSTALL.md).


## Documentation

Comprehensive documentation on Fluence can be found [here](https://fluence.dev). In particular, it includes [Aqua Book](https://fluence.dev/docs/aqua-book/getting-started/). Also, check our [YouTube channel](https://www.youtube.com/@fluencelabs). [This presentation](https://www.youtube.com/watch?v=Md0_Ny_5_1o&t=770s) at one of our community calls was especially dedicated to Registry.

Resources API is defined in the [resources-api](./aqua/resources-api.aqua) module. Service API is defined in the [registry-service](./aqua/registry-service.aqua) module. For the details, check the [API Reference](./API_reference.md).


## Repository Structure

- [**aqua-tests**](./aqua-tests) contains tests for the Aqua API and use
  it in e2e tests
- [**aqua**](./aqua) is the Aqua API registry
- [**builtin-package**](./builtin-package) contains a build script and
  config files used for building a standard distro for the Rust peer
  builtins
- [**service**](./service) is the Rust code for the Registry service


## Support

Please, file an [issue](https://github.com/fluencelabs/registry/issues) if you find a bug. You can also contact us at [Discord](https://discord.com/invite/5qSnPZKh7u) or [Telegram](https://t.me/fluence_project).  We will do our best to resolve the issue ASAP.


## Contributing

Any interested person is welcome to contribute to the project. Please, make sure you read and follow some basic [rules](./CONTRIBUTING.md).


## License

All software code is copyright (c) Fluence Labs, Inc. under the [Apache-2.0](./LICENSE) license.

