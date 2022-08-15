"use strict";
var __awaiter = (this && this.__awaiter) || function (thisArg, _arguments, P, generator) {
    function adopt(value) { return value instanceof P ? value : new P(function (resolve) { resolve(value); }); }
    return new (P || (P = Promise))(function (resolve, reject) {
        function fulfilled(value) { try { step(generator.next(value)); } catch (e) { reject(e); } }
        function rejected(value) { try { step(generator["throw"](value)); } catch (e) { reject(e); } }
        function step(result) { result.done ? resolve(result.value) : adopt(result.value).then(fulfilled, rejected); }
        step((generator = generator.apply(thisArg, _arguments || [])).next());
    });
};
Object.defineProperty(exports, "__esModule", { value: true });
const fluence_1 = require("@fluencelabs/fluence");
const fluence_network_environment_1 = require("@fluencelabs/fluence-network-environment");
const export_1 = require("./generated/export");
const export_2 = require("./generated/export");
const sk = "Iz3HUmNIB78lkNNVmMkDKrju0nCivtkJNyObrFAr774=";
function main() {
    return __awaiter(this, void 0, void 0, function* () {
        const keypair = yield fluence_1.KeyPair.fromEd25519SK(Buffer.from(sk, 'base64'));
        // connect to the Fluence network
        yield fluence_1.Fluence.start({ connectTo: fluence_network_environment_1.krasnodar[5], KeyPair: keypair });
        console.log("📗 created a fluence peer %s with relay %s", fluence_1.Fluence.getStatus().peerId, fluence_1.Fluence.getStatus().relayPeerId);
        let peerId = fluence_1.Fluence.getStatus().peerId;
        let relayId = fluence_1.Fluence.getStatus().relayPeerId;
        let serviceId = "echo";
        // register local service with service id "echo"
        yield (0, export_1.registerEchoService)(serviceId, { echo: (msg) => {
                console.log("Received message:", msg);
                return peerId + ": " + msg;
            } });
        console.log("Copy this code to call this service:");
        console.log(`fluence run -f 'echoJS("${peerId}", "${relayId}", "${serviceId}", "msg")'`);
        if (process.argv.length == 3) {
            let [success, error] = yield (0, export_2.registerProvider)(process.argv[2], "echo", serviceId);
            console.log("registration result: ", success);
        }
        // this code keeps fluence client running till any key pressed
        process.stdin.setRawMode(true);
        process.stdin.resume();
        process.stdin.on('data', () => __awaiter(this, void 0, void 0, function* () {
            yield fluence_1.Fluence.stop();
            process.exit(0);
        }));
    });
}
main().catch(error => {
    console.error(error);
    process.exit(1);
});
