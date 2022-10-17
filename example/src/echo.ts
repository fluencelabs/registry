/**
 * Copyright 2022 Fluence Labs Limited
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */
import { Fluence, KeyPair, setLogLevel } from "@fluencelabs/fluence";
import { stage } from "@fluencelabs/fluence-network-environment";
import { registerEchoService, registerService } from "./generated/export";
import assert from "node:assert";

// don't store your secret key in the code. This is just for the example
const secretKey = "Iz3HUmNIB78lkNNVmMkDKrju0nCivtkJNyObrFAr774=";

async function main() {
  const keypair = await KeyPair.fromEd25519SK(Buffer.from(secretKey, "base64"));
  const connectTo = stage[0];
  assert(connectTo !== undefined);

  // connect to the Fluence network
  await Fluence.start({ connectTo, KeyPair: keypair });
  setLogLevel("SILENT");

  const peerId = Fluence.getStatus().peerId;
  const relayId = Fluence.getStatus().relayPeerId;
  assert(peerId !== null && relayId !== null);
  console.log(`📗 created a fluence peer ${peerId} with relay ${relayId}`);

  const serviceId = "echo";

  // register local service with serviceId "echo"
  await registerEchoService(serviceId, {
    echo(msg) {
      console.log(`Received message: ${msg}`);
      return `${peerId}: ${msg}`;
    },
  });

  const resourceId = process.argv[2];

  // don't register if resource id isn't passed
  if (resourceId === undefined) {
    console.log(
      `
  Copy this code to call this service:

  fluence run -f 'echoJS("${peerId}", "${relayId}", "${serviceId}", "hi")'`
    );
  } else {
    const [success, error] = await registerService(
      resourceId,
      "echo",
      peerId,
      serviceId
    );
    console.log(`Registration result: ${success || error}`);
  }

  console.log("\nPress any key to stop fluence js peer");

  // this code keeps fluence client running till any key pressed
  process.stdin.setRawMode(true);
  process.stdin.resume();
  process.stdin.on("data", async () => {
    await Fluence.stop();
    process.exit(0);
  });
}

main().catch((error) => {
  console.error(error);
  process.exit(1);
});
