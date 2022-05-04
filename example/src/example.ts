import {Fluence, KeyPair} from "@fluencelabs/fluence";
import { krasnodar, Node,stage, testNet  } from "@fluencelabs/fluence-network-environment";
import { allowServiceFn, and, or } from "@fluencelabs/fluence/dist/internal/builtins/Sig";
import {createResourceAndRegisterProvider,test, registerNodeProvider, createResourceAndRegisterNodeProvider, createResource, registerProvider, resolveProviders, timestamp_sec} from "./generated/export";
import assert from "assert";

let local: Node[] = [
    {
        peerId: "12D3KooWHBG9oaVx4i3vi6c1rSBUm7MLBmyGmmbHoZ23pmjDCnvK",
        multiaddr:
            "/ip4/127.0.0.1/tcp/9990/ws/p2p/12D3KooWHBG9oaVx4i3vi6c1rSBUm7MLBmyGmmbHoZ23pmjDCnvK",
    },
    {
        peerId: "12D3KooWRABanQHUn28dxavN9ZS1zZghqoZVAYtFpoN7FdtoGTFv",
        multiaddr:
            "/ip4/127.0.0.1/tcp/9991/ws/p2p/12D3KooWRABanQHUn28dxavN9ZS1zZghqoZVAYtFpoN7FdtoGTFv",
    },
    {
        peerId: "12D3KooWFpQ7LHxcC9FEBUh3k4nSCC12jBhijJv3gJbi7wsNYzJ5",
        multiaddr:
            "/ip4/127.0.0.1/tcp/9992/ws/p2p/12D3KooWFpQ7LHxcC9FEBUh3k4nSCC12jBhijJv3gJbi7wsNYzJ5",
    },
];

async function main() {
    // connect to the Fluence network
    await Fluence.start({ connectTo: krasnodar[1] });
    console.log(
        "📗 created a fluence peer %s with relay %s",
        Fluence.getStatus().peerId,
        Fluence.getStatus().relayPeerId
    );

    let label = "myLabel";
    let value = "myValue";
    console.log("Will create resource with label:", label);
    let [resource_id, create_error] = await createResource(label);

    assert(resource_id !== null, create_error.toString());
    console.log("resource %s created successfully", resource_id);
    let node_provider = krasnodar[3].peerId;
    let [node_success, reg_node_error] = await registerNodeProvider(node_provider, resource_id, value, "identity");
    assert(node_success, reg_node_error.toString());
    console.log("node %s registered as provider successfully", node_provider);

    let [success, reg_error] = await registerProvider(resource_id, value, "identity");
    console.log("peer %s registered as provider successfully", Fluence.getStatus().peerId);
    assert(success, reg_error.toString());

    let [providers, error] = await resolveProviders(resource_id, 2);
    console.log("route providers:", providers);
}

main().then(() => process.exit(0))
    .catch(error => {
    console.error(error);
    process.exit(1);
  });
