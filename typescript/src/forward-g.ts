/**
 * Call forwarder.forward() to a G address
 * 1. Create new receiver account
 * 2. Fund via friendbot
 * 3. Add trustline for the asset
 * 4. Trigger forward
 */

import {
  Contract,
  initializeWithFriendbot,
  LocalSigner,
  PIPE_ClassicTransaction,
} from "@colibri/core";
import { Asset, Operation } from "stellar-sdk";
import chalk from "chalk";
import { networkConfig } from "./config/env.ts";
import { readFromJsonFile } from "./utils/io.ts";
import { loadWasmFile } from "./utils/load-wasm.ts";
import { buildMessage } from "./utils/build-message.ts";
import { DeploymentData } from "./config/types.ts";
import { Buffer } from "buffer";

const AMOUNT = 12345n;

console.log(chalk.bgBlue.black("\n=== Forward to G-address ===\n"));

const { assetIssuerSk, forwarderId, assetCode } =
  await readFromJsonFile<DeploymentData>("deployment");

const issuer = LocalSigner.fromSecret(assetIssuerSk);

const receiver = LocalSigner.generateRandom();
console.log(chalk.gray(`Receiver: ${chalk.green(receiver.publicKey())}`));

console.log(chalk.gray("Funding receiver via friendbot..."));
await initializeWithFriendbot(
  networkConfig.friendbotUrl as string,
  receiver.publicKey()
);

console.log(chalk.gray("Adding trustline..."));

const classicPipe = PIPE_ClassicTransaction.create({ networkConfig });

const asset = new Asset(assetCode, issuer.publicKey());

const trustlineOp = Operation.changeTrust({
  asset: asset,
});

const res = await classicPipe.run({
  operations: [trustlineOp],
  config: {
    source: receiver.publicKey(),
    fee: "10000",
    signers: [receiver],
    timeout: 45,
  },
});

console.log(chalk.gray("Trustline added", res.hash));

const message = buildMessage(forwarderId, AMOUNT, receiver.publicKey());
const attestation = Buffer.from(new Uint8Array(0));

console.log(chalk.gray(`Forwarder: ${chalk.green(forwarderId)}`));
console.log(chalk.gray(`Target: ${chalk.green(receiver.publicKey())}`));
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
