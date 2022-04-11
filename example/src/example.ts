import {Fluence, KeyPair} from "@fluencelabs/fluence";
import { krasnodar, Node } from "@fluencelabs/fluence-network-environment";
import {createResourceAndRegisterNodeProvider, resolveProviders, timestamp_sec} from "./generated/export";

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
    await Fluence.start({ connectTo: krasnodar[0] });
    console.log("%s", await timestamp_sec());
    console.log(
        "📗 created a fluence peer %s with relay %s",
        Fluence.getStatus().peerId,
        Fluence.getStatus().relayPeerId
    );
    let label = "myLabel";
    let value = "myValue";
    console.log("Will create resource with label:", label);
    // create route (if not exists) and register on it
    let [resource_id, error] = await createResourceAndRegisterNodeProvider(krasnodar[0].peerId,
      label, value, "identity"
    );

    if (resource_id !== null) {
        // find other peers on this route
        console.log("let's resolve route for %s", resource_id);
        let [providers, error] = await resolveProviders(resource_id, 5);
        console.log("route providers:", providers);
    } else {
        console.error(error);
    }
}

main().then(() => process.exit(0))
    .catch(error => {
    console.error(error);
    process.exit(1);
  });
