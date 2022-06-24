import { Fluence } from "@fluencelabs/fluence";
import { Node, stage } from "@fluencelabs/fluence-network-environment";
import { registerNodeProvider, createResource, registerProvider, resolveProviders } from "./generated/export";
import assert from "assert";


async function main() {
    // connect to the Fluence network
    await Fluence.start({ connectTo: stage[0] });
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
    let node_provider = stage[5].peerId;

    // this call should have bigger ttl
    let [node_success, reg_node_error] = await registerNodeProvider(node_provider, resource_id, value, "identity", {ttl: 20000});
    assert(node_success, reg_node_error.toString());
    console.log("node %s registered as provider successfully", node_provider);

    let [success, reg_error] = await registerProvider(resource_id, value, "identity");
    assert(success, reg_error.toString());
    console.log("peer %s registered as provider successfully", Fluence.getStatus().peerId);

    let [providers, error] = await resolveProviders(resource_id, 2);
    console.log("route providers:", providers);
    assert(providers.length == 2);
}

main().then(() => process.exit(0))
    .catch(error => {
        console.error(error);
        process.exit(1);
    });
