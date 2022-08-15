import { Fluence, KeyPair } from "@fluencelabs/fluence"
import { krasnodar } from "@fluencelabs/fluence-network-environment"
import { registerEchoService } from "./generated/export"
import { registerProvider } from "./generated/export"

const sk = "Iz3HUmNIB78lkNNVmMkDKrju0nCivtkJNyObrFAr774=";

async function main() {
    const keypair = await KeyPair.fromEd25519SK(Buffer.from(sk, 'base64'));
    // connect to the Fluence network
    await Fluence.start({ connectTo: krasnodar[5], KeyPair: keypair });
    console.log(
        "📗 created a fluence peer %s with relay %s",
        Fluence.getStatus().peerId,
        Fluence.getStatus().relayPeerId
    );

    let peerId =  Fluence.getStatus().peerId!;
    let relayId = Fluence.getStatus().relayPeerId!;
    let serviceId = "echo";

    // register local service with service id "echo"
    await registerEchoService(serviceId, {echo: (msg) => {
        console.log("Received message:", msg);
        return peerId + ": " + msg
    }});
    console.log("Copy this code to call this service:");
    console.log(`fluence run -f 'echoJS("${peerId}", "${relayId}", "${serviceId}", "msg")'`);

    // don't register if resource id isn't passed
    if (process.argv.length == 3) {
        let [success, error] = await registerProvider(process.argv[2], "echo", serviceId);
        console.log("registration result: ", success);
    }

    // this code keeps fluence client running till any key pressed
    process.stdin.setRawMode(true);
    process.stdin.resume();
    process.stdin.on('data', async () => {
        await Fluence.stop();
        process.exit(0);
    });
}

main().catch(error => {
    console.error(error);
    process.exit(1);
});
