/**
 * Call forwarder.forward() with a test message
 */

import { Contract, LocalSigner } from "@colibri/core";
import chalk from "chalk";
import { networkConfig } from "./config/env.ts";
import { readFromJsonFile } from "./utils/io.ts";
import { loadWasmFile } from "./utils/load-wasm.ts";
import { buildMessage } from "./utils/build-message.ts";
import { DeploymentData } from "./config/types.ts";
import { Buffer } from "buffer";

const TARGET = "CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC";
const AMOUNT = 12345n;

console.log(chalk.bgBlue.black("\n=== Forward to C-address ===\n"));

const { assetIssuerSk, forwarderId } = await readFromJsonFile<DeploymentData>(
  "deployment"
);

const issuer = LocalSigner.fromSecret(assetIssuerSk);

const message = buildMessage(forwarderId, AMOUNT, TARGET);
const attestation = Buffer.from(new Uint8Array(0)); // Empty attestation for testing

console.log(chalk.gray(`Forwarder: ${chalk.green(forwarderId)}`));
console.log(chalk.gray(`Target: ${chalk.green(TARGET)}`));
console.log(chalk.gray(`Amount: ${chalk.green(AMOUNT.toString())}`));
console.log(chalk.gray(`Message length: ${message.length} bytes`));

const forwarderWasm = await loadWasmFile("forwarder");
const forwarder = new Contract({
  networkConfig,
  contractConfig: {
    wasm: forwarderWasm as any,
    contractId: forwarderId,
  },
});

await forwarder.loadSpecFromWasm();

console.log(chalk.gray("\nCalling forward()..."));

const result = await forwarder.invoke({
  method: "forward",
  methodArgs: {
    message: message,
    attestation: attestation,
  },
  config: {
    source: issuer.publicKey(),
    fee: "10000",
    signers: [issuer],
    timeout: 45,
  },
});

console.log(chalk.bgGreen.black("\n=== Forward Complete ==="));
console.log("tx hash: ", result.hash);
