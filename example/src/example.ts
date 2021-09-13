import { FluencePeer } from "@fluencelabs/fluence";
import { krasnodar } from "@fluencelabs/fluence-network-environment";
import { initTopicAndSubscribe, findSubscribers } from "./generated/export";

async function main() {
    // connect to the Fluence network
    const peer = FluencePeer.default;
    await peer.init({ connectTo: krasnodar[1] });
    let topic = "myTopic";
    let value = "myValue";
    // create topic (if not exists) and subscribe on it
    let relay = peer.connectionInfo.connectedRelay!;
    await initTopicAndSubscribe(peer, topic, value, relay, null);
    // find other peers subscribed to that topic
    let subscribers = await findSubscribers(peer, topic);
    console.log("found subscribers:", subscribers);
}

main()
  .then(() => process.exit(0))
  .catch(error => {
    console.error(error);
    process.exit(1);
  });
