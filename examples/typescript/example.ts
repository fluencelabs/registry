import { initTopicAndSubscribe, findSubscribers } from "@fluencelabs/aqua-dht-ts";
import { createClient } from "@fluencelabs/fluence";
import { krasnodar } from "@fluencelabs/fluence-network-environment";

async function main() {
    // connect to the Fluence network
    const client = await createClient(krasnodar[1]);
    let topic = "myTopic";
    let value = "myValue";
    // create topic (if not exists) and subscribe on it
    await initTopicAndSubscribe(client, client.relayPeerId!, topic, value, client.relayPeerId!, null);
    // find other peers subscribed to that topic
    let subscribers = await findSubscribers(client, client.relayPeerId!, topic);
    console.log("found subscribers:", subscribers);
}

main()
  .then(() => process.exit(0))
  .catch(error => {
    console.error(error);
    process.exit(1);
  });
