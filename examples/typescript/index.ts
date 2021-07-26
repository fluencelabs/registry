import { initTopicAndSubscribe, findSubscribers } from "@fluencelabs/aqua-dht-ts";
import { createClient } from "@fluencelabs/fluence";
import { krasnodar } from "@fluencelabs/fluence-network-environment";

async function main() {
    const client = await createClient(krasnodar[1]);
    let topic = "myTopic";
    let value = "myValue";
    await initTopicAndSubscribe(client, client.relayPeerId!, topic, value, client.relayPeerId!, null);
    let subscribers = await findSubscribers(client, client.relayPeerId!, topic);
    console.log("found subscribers:", subscribers);
}

main()
  .then(() => process.exit(0))
  .catch(error => {
    console.error(error);
    process.exit(1);
  });
