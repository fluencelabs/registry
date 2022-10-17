# Services advertisement and discovery

## Overview

This example shows how to use Registry to discover and call fluence services without having their exact peer and service ids.

## Table of contents:

- [Services advertisement and discovery](#services-advertisement-and-discovery)
  - [Overview](#overview)
  - [Table of contents:](#table-of-contents)
  - [Set up the environment](#set-up-the-environment)
  - [Deploy echo service written in Rust](#deploy-echo-service-written-in-rust)
  - [Run echo service written in JS/TS](#run-echo-service-written-in-jsts)
  - [Register both services using Registry](#register-both-services-using-registry)
  - [Call both services using resourceId](#call-both-services-using-resourceid)
  - [Remove service record](#remove-service-record)

## Set up the environment

1. Install the latest version of Fluence CLI:
    ```sh
    npm i -g @fluencelabs/cli
    ```
2. Install Fluence project dependencies. It may take a while:
    ```sh
    fluence dependency i
    ```
3. Install JS dependencies:
    ```sh
    npm i
    ```
You can also use VSCode with [Aqua extension](https://marketplace.visualstudio.com/items?itemName=FluenceLabs.aqua) for [Aqua language](https://fluence.dev/docs/aqua-book/getting-started/) syntax highlighting and better developer experience.

## Deploy echo service written in Rust

To deploy the Fluence application execute
```sh
fluence deploy
```
Press Enter when prompted `? Do you want to deploy all of these services? (Y/n)`

This Fluence application, described in [fluence.yaml](fluence.yaml), consists of just one [echo service](./echo_service) which has only one [module](./echo_service/modules/echo_service/) written in Rust. [The module code](echo_service/modules/echo_service/src/main.rs) has only one function [echo](echo_service/modules/echo_service/src/main.rs#L9), which returns your `msg` along with peerId of the host:

To call [echo](src/aqua/main.aqua#L8) aqua function execute:
```sh
fluence run -f 'echo("hi")'
```
The function uses `peerId` and `serviceId`, which Fluence CLI stored in `./.fluence/app.yaml` when you deployed the Fluence application in the previous step.

You should see output similar to this:
```
"12D3KooWFEwNWcHqi9rtsmDhsYcDbRUCDXH84RC4FW6UfsFWaoHi: hi"
```

It means we successfully deployed our echo service, and anyone can call it if they have `peerId` and `serviceId`

## Run echo service written in JS/TS

Execute
```sh
npm run start
```

First, aqua code in [src/aqua/export.aqua](src/aqua/export.aqua) will be compiled to typescript and you will see it in [src/generated/export.ts](src/generated/export.ts).

Then you possibly will have to confirm ts-node installation and [src/echo.ts](src/echo.ts) will be executed. It registers local js service with serviceId "echo", so anyone who has `relayId`, `peerId` and `serviceId` ("echo") will be able to call it. Copy the command from the terminal, which will look similar to this:
```sh
fluence run -f 'echoJS("12D3KooWCmnhnGvKTqEXpVLzdrYu3TkQ3HcLyArGJpLPooJQ69dN", "12D3KooWSD5PToNiLQwKDXsu8JSysCwUt8BVUJEqCHcDe7P5h45e", "echo", "hi")'
```
This command executes [echoJS](src/aqua/main.aqua#L16) aqua function with arguments: relayId, peerId, serviceId and msg

Open another terminal in the same directory, paste the command and run it.

You should see output similar to this:
```
"12D3KooWCmnhnGvKTqEXpVLzdrYu3TkQ3HcLyArGJpLPooJQ69dN: hi"
```

It means anyone can call our `echo` service, written in TS/JS, if they have `relayId`, `peerId` and `serviceId`.
## Register both services using Registry

We can register our services in Registry if we want anyone to be able to call our services without specifying the exact relay, peer, and service IDs.

First, we need to create the Resource. The Resource represents a group of services and has a corresponding `resourceId` which we can use for service discovery.

To call [createRes](src/aqua/main.aqua#L22) aqua function, execute
```sh
fluence run -f 'createRes()'
```
It uses `createResource` function from Resources API to register the Resource with the label `echo`.
You should see output similar to this:

```
[
  "5pYpWB3ozi6fi1EjNs9X5kE156aA6iLECxTuVdJgUaLB"
]
```

It is `resourceId`, which we will use to register our services, and then we will be able to use the same `resourceId` to discover and call our services

To register the `echo` service written in Rust, replace `RESOURCE_ID` and execute
```sh
fluence run -f 'registerEchoService("RESOURCE_ID")'
```
This command calls [registerEchoService](src/aqua/main.aqua#L26) aqua function, which uses `registerService` function from Resources API to register the rust service on this `resourceId`

You should see this output:
```
[
  [
    true
  ]
]
```
It means the service is registered in Registry and should be accessible by anyone who only has the `resourceId` of this service.

Then please stop fluence js peer in the previous terminal that you ran.

To register echo service written in JS/TS on the Resource, replace `RESOURCE_ID` and execute
```sh
npm run start -- 'RESOURCE_ID'
```
## Call both services using resourceId
Go to a different terminal in the same directory, replace `RESOURCE_ID` and execute this command to call [echoAll](src/aqua/main.aqua#L33) aqua function
```sh
fluence run -f 'echoAll("RESOURCE_ID", "hi")'
```
It uses `resourceId` to resolve a minimum of two records with peer and service ids and then uses them to call our services

You should see output similar to this:
```
[
  [
    "12D3KooWFEwNWcHqi9rtsmDhsYcDbRUCDXH84RC4FW6UfsFWaoHi: hi",
    "12D3KooWCmnhnGvKTqEXpVLzdrYu3TkQ3HcLyArGJpLPooJQ69dN: hi"
  ]
]
```
It means we successfully registered our services using Registry, and now anyone can call these services using only `resourceId`.

## Remove service record
Replace `RESOURCE_ID` and execute
```sh
fluence run -f 'unregisterEchoService("RESOURCE_ID")'
```
to call [unregisterEchoService](src/aqua/main.aqua#L43) function that uses `unregisterService` function from Resources API to unregister only our echo services written in Rust

The output should look like this:
```
[
  [
    true
  ]
]
```
Let's make sure we've removed the service record. Once again, replace `RESOURCE_ID` and  execute
```sh
fluence run -f 'echoAll("RESOURCE_ID", "hi")'
```

You should see output similar to this:
```
[
  [
    "12D3KooWCmnhnGvKTqEXpVLzdrYu3TkQ3HcLyArGJpLPooJQ69dN: hi"
  ]
]
```
You can notice that only one result is left instead of two. It means we successfully removed the service record from our Resource
