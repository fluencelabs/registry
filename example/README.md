# Services advertisement and discovery

## Overview

This example shows how to use Registry to discover and call fluence services without having their exact peer and service ids

## Table of contents:

1. [Set up the environment](#set-up-the-environment)
2. [Deploy echo service written in Rust](#deploy-echo-service-written-in-rust)
3. [Run echo service written in JS/TS](#run-echo-service-written-in-jsts)
4. [Register both services in Registry](#register-both-services-in-registry)
5. [Call both services using resourceId](#call-both-services-using-resourceid)
6. [Remove service record](#remove-service-record)

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
4. You can use VSCode with [Aqua extension](https://marketplace.visualstudio.com/items?itemName=FluenceLabs.aqua) for [Aqua language](https://fluence.dev/docs/aqua-book/getting-started/) syntax highlighting and better developer experience.

## Deploy echo service written in Rust

To deploy Fluence application execute 
```sh
fluence deploy
```
Press Enter when prompted `? Do you want to deploy all of these services? (Y/n)`

This Fluence application, described in [fluence.yaml](fluence.yaml), consists of just one [echo service](./echoService) which has only one [module](./echoService/modules/echoService/) written in Rust. [The module code](echoService/modules/echoService/src/main.rs) has only one function [echo](echoService/modules/echoService/src/main.rs#L9), which simply returns your `msg` back along with peerId of the host:

`peerId`, that was randomly selected, and `serviceId` which was assigned to the deployed service will be both stored in [app.yaml](./.fluence/app.yaml) after deployment is complete

To call [echo](src/aqua/main.aqua#L8) aqua function execute:
```sh
fluence run -f 'echo("hi")'
```
The function uses IDs of the deployed service, which were stored in `./.fluence/app.yaml` when you deployed Fluence application

You should see output similar to this:
```
"12D3KooWFEwNWcHqi9rtsmDhsYcDbRUCDXH84RC4FW6UfsFWaoHi: hi"
```

This means our echo service is successfully deployed and anyone can call it, if they have peerId and serviceId

## Run echo service written in JS/TS

Execute
```sh
npm run start
```

First, aqua code in [src/aqua/export.aqua](src/aqua/export.aqua) will be compiled to typescript and you will see it in [src/generated/export.ts](src/generated/export.ts).

Then [src/echo.ts](src/echo.ts) will be executed. It registers local js service with serviceId "echo" so anyone who has `relayId`, `peerId` and `serviceId` ("echo") will be able to call it. Leave this fluence js peer running for now

Copy the command from the terminal where you ran JS/TS echo service. This command executes [echoJS](src/aqua/main.aqua#L16) aqua function with arguments: relayId, peerId, serviceId and msg

Open another terminal in the same directory, paste the command and run it

You should see output similar to this:
```
"12D3KooWCmnhnGvKTqEXpVLzdrYu3TkQ3HcLyArGJpLPooJQ69dN: hi"
```

This means anyone can call our `echo` service written in TS/JS, if they have relay id, peerId and serviceId
## Register both services in Registry

We can register our services in Registry if we want anyone to be able to call our services without specifying the exact relay, peer and service ids

First we need to create resource. Resource represents a group of services and has a corresponding `resourceId` which is used for service discovery.

To call [createRes](src/aqua/main.aqua#L22) aqua function, execute
```sh
fluence run -f 'createRes()'
```
It uses `createResource` function from Resources API to register resource with label `echo`.
You should see output similar to this:

```
[
  "5pYpWB3ozi6fi1EjNs9X5kE156aA6iLECxTuVdJgUaLB"
]
```

This is `resourceId`, which we will use to register our services and then other people will be able to use the same `resourceId` to discover and call our services

To register echo service written in Rust, replace `RESOURCE_ID` and execute
```sh
fluence run -f 'registerService("RESOURCE_ID")'
```
This command calls [registerService](src/aqua/main.aqua#L26) aqua function, which uses `registerServiceRecord` function from Resources API to register the rust service on this `resourceId`

You should see this output:
```
[
  [
    true
  ]
]
```
This means the service is registered in Registry and should be accessible by anyone who has only resource id of this service

Then you need to stop fluence js peer in the terminal that you previously ran.

To register echo service written in JS/TS on the resource, replace `RESOURCE_ID` and execute
```sh
npm run start -- 'RESOURCE_ID'
```

## Call both services using resourceId
Go to a different terminal in the same directory, replace `RESOURCE_ID` and execute
```sh
fluence run -f 'echoAll("RESOURCE_ID", "hi")'
```
to call [echoAll](src/aqua/main.aqua#L33) aqua function, which uses `resourceId` to resolve minimum 2 records with peer and service ids and then uses them to call our services

You should see output similar to this:
```
[
  [
    "12D3KooWFEwNWcHqi9rtsmDhsYcDbRUCDXH84RC4FW6UfsFWaoHi: hi",
    "12D3KooWCmnhnGvKTqEXpVLzdrYu3TkQ3HcLyArGJpLPooJQ69dN: hi"
  ]
]
```
This means we successfully registered our services using Registry and now anyone who has `resourceId` can call these services

## Remove service record
Replace `RESOURCE_ID` and execute
```sh
fluence run -f 'unregisterEchoService("RESOURCE_ID")'
```
to call [unregisterEchoService](src/aqua/main.aqua#L43) function that uses `unregisterService` function from Resources API to unregister only our echo services written in Rust

Output should look like this:
```
[
  [
    true
  ]
]
```
Let’s make sure it is removed. Once again, replace `RESOURCE_ID` and  execute
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
You can notice that only one result is left instead of two. This means we successfully removed service record from our Resource
