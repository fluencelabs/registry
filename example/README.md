# Services advertisement and discovery

## Overview

The example shows one of the important Registry use-cases — services advertisement and discovery. So it is about advertisement, discovery and usage by clients with the same Aqua code transparently without any knowledge about particular peer and service ids.

In the beginning, we will deploy a Rust echo service and call it with the Fluence CLI.
Then we will start a JS/TS client with echo service and run it.

Secondly, we will use Registry to call different types of services with exactly one piece of code.
And finally, we will show how to remove service records.

## Requirements

```markdown
node: >= 16
rust: rustc 1.63.0-nightly
@fluencelabs/cli: 0.2.13
```

# Set up environment

1. Install `fluence` cli:

    ```bash
    npm i -g @fluencelabs/cli@0.2.13
    ```

2. Initialize Fluence project:

    ```bash
    fluence init .
    ```
    Output:
    ```bash
    Successfully initialized Fluence project template at <your-path-to-registry-repo>/example
    ```

3. You can use [VSCode with Aqua extension](https://marketplace.visualstudio.com/items?itemName=FluenceLabs.aqua) for syntax highlighting  and better developer experience.

## Add echo service in Rust

0. If you have **Apple Silicon** you should install cargo manually, or you can ignore this step otherwise:

    ```bash
    # for m1:
    cargo install marine --version '0.12.1' --root ~/.fluence/cargo
    ```

1. Add echo service:

    ```bash
    fluence service add echoService
    ```

    output:

    ```bash
    $ fluence service add echoService
    Added echoService to fluence.yaml
    ```

2. Following [code](echoService/modules/echoService/src/main.rs) returns a peer id of the host and an initial message:

    ```rust
    #[marine]
    pub fn echo(msg: String) -> String {
        format!("{}: {}", marine_rs_sdk::get_call_parameters().host_id, msg)
    }
    ```

### Test echo service locally with REPL
0. If you have **Apple Silicon** you should install mrepl manually, or you can ignore this step otherwise:

    ```bash
    # for m1:
    cargo install mrepl --version '0.18.6' --root ~/.fluence/cargo
    ```

1. Run the following command to start REPL:
    ```bash
    fluence service repl echoService
    ```
    output:
    ```bash
    $ fluence service repl echoService
    Making sure service and modules are downloaded and built... done
    Welcome to the Marine REPL (version 0.18.0)
    Minimal supported versions
    sdk: 0.6.0
    interface-types: 0.20.0

    app service was created with service id = 070fd906-a9e4-44af-963d-81a75526379c
    elapsed time 55.397166ms

    1>
    ```
2. Call `echo` function of `echoService` module and pass `msg` to it:
   ```bash
   call echoService echo ["msg"]
   ```
   output:
   ```
    1> call echoService echo ["msg"]
    result: String(": msg")
    elapsed time: 9.207542ms
   ```
   Peer id is empty because that is default for Marine REPL.

You can always test your services before deployment with REPL. Check out [documentation](https://doc.fluence.dev/marine-book/marine-tooling-reference/marine-repl) for more details.

### Deploy service and test it on remote peer

1. Deploy service:

    ```bash
    fluence deploy
    ```

    Output, please note that peer ids and service ids may differ:

    ```bash
    $ fluence deploy
    Making sure all services are downloaded... done
    Making sure all modules are downloaded and built... done

    Going to deploy services described in ~/Documents/dev/fluencelabs/registry/example/fluence.yaml:

    echoService:
    get: echoService
    deploy:
        - deployId: default


    ? Do you want to deploy all of these services? Yes
    Deploying:
    service: echoService
    deployId: default
    on: 12D3KooWCMr9mU894i8JXAFqpgoFtx6qnV1LFPSfVc3Y34N4h4LS
    ... done
    Compiling ~/Documents/dev/fluencelabs/registry/example/.fluence/aqua/deployed.app.aqua... done

    Currently deployed services listed in ~/Documents/dev/fluencelabs/registry/example/.fluence/app.yaml:

    echoService:
    default:
        - blueprintId: fd21793bae96d1519599346591bfe7ea75614d9fe7f7c824b279e5a2dc841ee0
        serviceId: 785487e4-de08-4fa4-bdfd-df981ec66da7
        peerId: 12D3KooWCMr9mU894i8JXAFqpgoFtx6qnV1LFPSfVc3Y34N4h4LS
    ```


2.  The following [code](src/aqua/main.aqua) declares a module, imports App and EchoService, and defines a method to call `echo`:

    ```rust
    module Main

    import App from "deployed.app.aqua"
    import EchoService from "services/echoService.aqua"
    export App, echo

    func echo(msg: string) -> string:
        services <- App.services()

        on services.echoService.default[0].peerId:
            EchoService services.echoService.default[0].serviceId
            res <- EchoService.echo(msg)
        <- res
    ```

3. Let’s test our echo service:

    ```bash
     fluence run -f 'echo("hi")'
    ```

    Output:

    ```bash
    $ fluence run -f 'echo("hi")'
    Running:
      function: echo("hi")
      relay: /dns4/kras-08.fluence.dev/tcp/19001/wss/p2p/12D3KooWFtf3rfCDAfWwt6oLZYZbDfn9Vn7bv7g6QjjQxUUEFVBt
    ... done

    Result:

    "12D3KooWCMr9mU894i8JXAFqpgoFtx6qnV1LFPSfVc3Y34N4h4LS: hi"
    ```

We've successfully built and deployed a service written in Rust, and called it from our Aqua code using the Fluence CLI.

## Run echo service in JS/TS:

1. Install dependencies:

    ```bash
    npm i
    ```

2. Check out [src/aqua/export.aqua](src/aqua/export.aqua) with an export of EchoService API to JS/TS:

    ```bash
    module Export
    import "services/echoService.aqua"
    export EchoService
    ```

3. The following [code](src/echo.ts#L23) registers a local EchoService:

    ```tsx
    // register local service with service id "echo"
    await registerEchoService(serviceId, {echo: (msg) => {
        console.log("Received message:", msg);
        return peerId + ": " + msg
    }});
    ```

4. The following [code](src/aqua/main.aqua) is calling our local JS/TS peer with `EchoService`:

    ```rust
    func echoJS(peer: string, relay: string, serviceId: string, msg: string) -> string:
        on peer via relay:
            EchoService serviceId
            res <- EchoService.echo(msg)
        <- res
    ```
    <a id="running-service"></a>
5. Start the client with our service: `npm run start`
    ```bash
    ...
    > example@1.0.0 start
    > node src/echo.js

    📗 created a fluence peer 12D3KooWCmnhnGvKTqEXpVLzdrYu3TkQ3HcLyArGJpLPooJQ69dN with relay 12D3KooWFEwNWcHqi9rtsmDhsYcDbRUCDXH84RC4FW6UfsFWaoHi
    Copy this code:
    fluence run -f 'echoJS("12D3KooWCmnhnGvKTqEXpVLzdrYu3TkQ3HcLyArGJpLPooJQ69dN", "12D3KooWFEwNWcHqi9rtsmDhsYcDbRUCDXH84RC4FW6UfsFWaoHi", "echo", "msg")'
    ```

6. Open a new terminal in the same registry example directory, and execute the following command to check echo service:

    ```bash
    $ fluence run -f 'echoJS("12D3KooWCmnhnGvKTqEXpVLzdrYu3TkQ3HcLyArGJpLPooJQ69dN", "12D3KooWFEwNWcHqi9rtsmDhsYcDbRUCDXH84RC4FW6UfsFWaoHi", "echo", "msg")'
    Running:
      function: echoJS("12D3KooWCmnhnGvKTqEXpVLzdrYu3TkQ3HcLyArGJpLPooJQ69dN", "12D3KooWFEwNWcHqi9rtsmDhsYcDbRUCDXH84RC4FW6UfsFWaoHi", "echo", "msg")
      relay: /dns4/kras-09.fluence.dev/tcp/19001/wss/p2p/12D3KooWD7CvsYcpF9HE9CCV9aY3SJ317tkXVykjtZnht2EbzDPm
    ... done

    Result:

    "12D3KooWCmnhnGvKTqEXpVLzdrYu3TkQ3HcLyArGJpLPooJQ69dN: msg"
    ```

We've successfully started JS/TS peer with an EchoService and tested it with Fluence CLI.
## Register both services in Registry

As service providers we would like all our echo services to be discoverable without specifying particular peer and service ids, in order to achieve that we should use Registry to **advertise** our services.

Firstly, we need to create a **resource.** A resource represents a group of services and has a corresponding resource id for its discovery. Secondly, we need to register the service records on this resource id. So the echo services can be discovered and resolved by the resource id only.

1. The following [code](src/aqua/main.aqua#L24) registers resource with label `echo`:

    ```rust
    func registerResource() -> ?string:
        resource_id, error <- createResource("echo")
        <- resource_id
    ```

2. Let’s register resource:
    <a id="resource-id"></a>
    ```bash
    fluence run -f 'registerResource()'
    ```

    output:

    ```bash
    $ fluence run -f 'registerResource()'
    Running:
      function: registerResource()
      relay: /dns4/kras-01.fluence.dev/tcp/19001/wss/p2p/12D3KooWKnEqMfYo9zvfHmqTLpLdiHXPe4SVqUWcWHDJdFGrSmcA
    ... done

    Result:

    [
      "5pYpWB3ozi6fi1EjNs9X5kE156aA6iLECxTuVdJgUaLB"
    ]
    ```

   So the resource id is `5pYpWB3ozi6fi1EjNs9X5kE156aA6iLECxTuVdJgUaLB`. Please note that the resource id might be different in your case.

    Using the resource id we can access any registered service. There can be more registered echo services on different peers that we can use transparently.

3. This [code](src/aqua/main.aqua#L28) registers deployed service by given `resource_id`:

    ```rust
    func registerService(resource_id: string) -> *bool:
        results: *bool
        services <- App.services()
        for srv <- services.echoService.default:
            results <- registerServiceRecord(resource_id, "" ,srv.peerId, ?[srv.serviceId])
        <- results
    ```

4. Register service for resource_id from [the step 2](#resource-id):

    ```rust
    fluence run -f 'registerService("5pYpWB3ozi6fi1EjNs9X5kE156aA6iLECxTuVdJgUaLB")'
    ```

    output:

    ```bash
    $ fluence run -f 'registerService("5pYpWB3ozi6fi1EjNs9X5kE156aA6iLECxTuVdJgUaLB")'
    Running:
      function: registerService("5pYpWB3ozi6fi1EjNs9X5kE156aA6iLECxTuVdJgUaLB")
      relay: /dns4/kras-05.fluence.dev/tcp/19001/wss/p2p/12D3KooWCMr9mU894i8JXAFqpgoFtx6qnV1LFPSfVc3Y34N4h4LS
    ... done

    Result:

    [
      [
        true
      ]
    ]
    ```

5. Next, we need to register JS service. For that we have Aqua imports and exports in [src/aqua/export.aqua](src/aqua/export.aqua):

    ```
    import registerServiceRecord from "@fluencelabs/registry/resources-api.aqua"
    export registerServiceRecord
    ```

6. In [src/echo.ts](src/echo.ts#L32) we should pass resource id as cmd argument:

    ```tsx
    let [success, error] = await registerServiceRecord(process.argv[2], "echo", peerId, serviceId);
    console.log("registration result: ", success);
    ```

7. So, stop the previous JS client from [the step 5](#running-service) and run
`npm run start 5pYpWB3ozi6fi1EjNs9X5kE156aA6iLECxTuVdJgUaLB` (Note: resource_id is from [the step 2](#resource-id))
output:

    ```rust
    ...
    Copy this code to call this service:
    fluence run -f 'echoJS("12D3KooWCmnhnGvKTqEXpVLzdrYu3TkQ3HcLyArGJpLPooJQ69dN", "12D3KooWFEwNWcHqi9rtsmDhsYcDbRUCDXH84RC4FW6UfsFWaoHi", "echo", "msg")'
    registration result:  true
    ```

We've successfully registered both services, JS/TS and Rust, in Registry and now we can access them only with `resource_id` without knowledge of particular peer and service ids.

## Call services with Registry (using resource_id)

1. In [src/aqua/main.aqua](src/aqua/main.aqua#L35) defined `echoAll` to resolve services and call sequentially:

    ```rust
    func echoAll(resource_id: string, msg: string) -> *string:
        -- 2 is the min number of peers we want to ask
        records <- resolveResource(resource_id, 2)
        results: *string
        for r <- records:
            on r.metadata.peer_id via r.metadata.relay_id:
                EchoService r.metadata.service_id!
                results <- EchoService.echo(msg)
        <- results
    ```

2. Let’s run all registered echo services with only resource id:

    ```bash
    fluence run -f 'echoAll("5pYpWB3ozi6fi1EjNs9X5kE156aA6iLECxTuVdJgUaLB", "hi")'
    ```

    output:
    ```
    $ fluence run -f 'echoAll("5pYpWB3ozi6fi1EjNs9X5kE156aA6iLECxTuVdJgUaLB", "hi")'
    Running:
    function: echoAll("5pYpWB3ozi6fi1EjNs9X5kE156aA6iLECxTuVdJgUaLB", "hi")
    relay: /dns4/kras-06.fluence.dev/tcp/19001/wss/p2p/12D3KooWDUszU2NeWyUVjCXhGEt1MoZrhvdmaQQwtZUriuGN1jTr
    ... done

    Result:

    [
        [
            "12D3KooWFEwNWcHqi9rtsmDhsYcDbRUCDXH84RC4FW6UfsFWaoHi: hi",
            "12D3KooWCmnhnGvKTqEXpVLzdrYu3TkQ3HcLyArGJpLPooJQ69dN: hi"
        ]
    ]
    ```

## Remove service record

If we want to remove a service record from resource, we should use `unregisterService` method from Resources API:

```rust
func unregisterEchoService(resource_id: string) -> *bool:
    results: *bool
    services <- App.services()
    for srv <- services.echoService.default:
        results <- unregisterService(resource_id, srv.peerId)
    <- results
```

1. Let's unregister our deployed EchoService:

    ```bash
    fluence run -f 'unregisterEchoService("5pYpWB3ozi6fi1EjNs9X5kE156aA6iLECxTuVdJgUaLB")'
    ```
    output:
    ```
    $ fluence run -f 'unregisterEchoService("5pYpWB3ozi6fi1EjNs9X5kE156aA6iLECxTuVdJgUaLB")'
    Running:
        function: unregisterEchoService("5pYpWB3ozi6fi1EjNs9X5kE156aA6iLECxTuVdJgUaLB")
        relay: /dns4/kras-05.fluence.dev/tcp/19001/wss/p2p/12D3KooWCMr9mU894i8JXAFqpgoFtx6qnV1LFPSfVc3Y34N4h4LS
    ... done

    Result:

    [
        [
            true
        ]
    ]
    ```

2. Let’s run again `echoAll` method:

    ```bash
    fluence run -f 'echoAll("5pYpWB3ozi6fi1EjNs9X5kE156aA6iLECxTuVdJgUaLB", "hi")'
    ```

    output:
    ```
    $ fluence run -f 'echoAll("5pYpWB3ozi6fi1EjNs9X5kE156aA6iLECxTuVdJgUaLB", "hi")'
    Running:
    function: echoAll("5pYpWB3ozi6fi1EjNs9X5kE156aA6iLECxTuVdJgUaLB", "hi")
    relay: /dns4/kras-06.fluence.dev/tcp/19001/wss/p2p/12D3KooWDUszU2NeWyUVjCXhGEt1MoZrhvdmaQQwtZUriuGN1jTr
    ... done

    Result:

    [
        [
            "12D3KooWCmnhnGvKTqEXpVLzdrYu3TkQ3HcLyArGJpLPooJQ69dN: hi"
        ]
    ]
    ```

So now we know how to add and remove service records for Resource and use it for advertising and discovering services in runtime.
