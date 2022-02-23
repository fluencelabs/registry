import {Fluence, KeyPair} from "@fluencelabs/fluence";
import { krasnodar, Node } from "@fluencelabs/fluence-network-environment";
import {initTopicAndSubscribeBlocking, findSubscribers, add_alias, timestamp_sec} from "./generated/export";


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

    await Fluence.start({ connectTo: local[0] });
    console.log("%s", await timestamp_sec());
    console.log(
        "📗 created a fluence peer %s with relay %s",
        Fluence.getStatus().peerId,
        Fluence.getStatus().relayPeerId
    );
    // await add_alias("registry", "53a9a930-fe8c-47e9-b06d-2ce052d72f38");
    let topic = "myTopic";
    let value = "myValue";
    console.log("Will create topic", topic);
    // create topic (if not exists) and subscribe on it
    let relay = Fluence.getStatus().relayPeerId;
    let route_id = await initTopicAndSubscribeBlocking(
      topic, value, relay, null,
      (s) => console.log(`node ${s} saved the record`)
    );

    process.stdin.setRawMode(true);
    process.stdin.resume();
    process.stdin.on('data', async () => {
        // find other peers subscribed to that topic
        console.log("let's find subscribers for %s", route_id);
        let subscribers = await findSubscribers(route_id);
        console.log("found subscribers:", subscribers);
        await Fluence.stop();
        process.exit(0);
    });

}

main()
  .catch(error => {
    console.error(error);
    process.exit(1);
  });
