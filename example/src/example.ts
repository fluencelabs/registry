import { Fluence } from "@fluencelabs/fluence";
import { krasnodar } from "@fluencelabs/fluence-network-environment";
import { initTopicAndSubscribeBlocking, findSubscribers } from "./generated/export";

async function main() {
    // connect to the Fluence network
    await Fluence.start({ connectTo: krasnodar[1] });
    let topic = "myTopic" + new Date().valueOf();
    let value = "myValue";
    console.log("Will create topic", topic);
    // create topic (if not exists) and subscribe on it
    let relay = Fluence.getStatus().relayPeerId;
    await initTopicAndSubscribeBlocking(
      topic, value, relay, null, 
      (s) => console.log(`node ${s} saved the record`)
    );
    // find other peers subscribed to that topic
    let subscribers = await findSubscribers(topic);
    console.log("found subscribers:", subscribers);
}

main()
  .then(() => process.exit(0))
  .catch(error => {
    console.error(error);
    process.exit(1);
  });
