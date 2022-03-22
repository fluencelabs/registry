import { Fluence } from "@fluencelabs/fluence";
import { krasnodar } from "@fluencelabs/fluence-network-environment";
import { createRouteAndRegisterBlocking, resolveRoute } from "./generated/export";

export async function main() {
  // connect to the Fluence network
  await Fluence.start({ connectTo: krasnodar[1] });

  let label = "myLabel";
  let value = "put anything useful here";
  let serviceId = "Foo";
  let ack = 5;

  // create route and register for it
  let relay = Fluence.getStatus().relayPeerId;
  let route_id = await createRouteAndRegisterBlocking(
    label, value, relay, serviceId,
    (s) => console.log(`node ${s} saved the record`),
    ack
  );

  // this will contain peer as route provider
  let providers = await resolveRoute(route_id, ack);
}