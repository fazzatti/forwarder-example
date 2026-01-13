/**
 * Setup script for CCTP Forwarder prototype
 *
 * 1. Fund issuer account via friendbot
 * 2. Create custom asset and wrap as SAC
 * 3. Deploy message_transmitter contract (with asset)
 * 4. Set transmitter as SAC admin (so it can mint)
 * 5. Deploy forwarder contract (with asset + transmitter)
 * 6. Save deployment data
 */

import {
  Contract,
  initializeWithFriendbot,
  LocalSigner,
  StellarAssetContract,
  TransactionConfig,
} from "@colibri/core";
import chalk from "chalk";
import { networkConfig } from "./config/env.ts";
import { saveToJsonFile } from "./utils/io.ts";
import { loadWasmFile } from "./utils/load-wasm.ts";
import { DeploymentData } from "./config/types.ts";

const issuer = LocalSigner.generateRandom();
const ASSET_CODE = "DEMO";

console.log(chalk.bgBlue.black("\n=== Forwarder Example Setup ===\n"));

console.log(chalk.gray(`Funding issuer: ${chalk.green(issuer.publicKey())}`));
await initializeWithFriendbot(
  networkConfig.friendbotUrl as string,
  issuer.publicKey()
);

const txConfig: TransactionConfig = {
  source: issuer.publicKey(),
  fee: "10000",
  signers: [issuer],
  timeout: 45,
};

const asset = new StellarAssetContract({
  networkConfig,
  code: ASSET_CODE,
  issuer: issuer.publicKey(),
});

console.log(
  chalk.gray(`Asset: ${chalk.green(asset.code)}:${issuer.publicKey()}`)
);
console.log(chalk.gray(`SAC Contract ID: ${chalk.green(asset.contractId)}`));
console.log(chalk.gray("Deploying SAC..."));

await asset.deploy(txConfig);
console.log(chalk.gray("SAC deployed"));

console.log(chalk.gray("Loading message_transmitter WASM..."));

const transmitterWasm = await loadWasmFile("message_transmitter");

const transmitter = new Contract({
  networkConfig,
  contractConfig: { wasm: transmitterWasm as any },
});

await transmitter.loadSpecFromWasm();

console.log(chalk.gray("Uploading transmitter WASM..."));
await transmitter.uploadWasm(txConfig);

console.log(chalk.gray("Deploying transmitter..."));
await transmitter.deploy({
  config: txConfig,
  constructorArgs: { asset: asset.contractId },
});

console.log(`Transmitter ID: ${chalk.green(transmitter.getContractId())}`);

console.log(chalk.gray("Setting transmitter as SAC admin..."));
await asset.setAdmin({
  newAdmin: transmitter.getContractId(),
  config: txConfig,
});

console.log(chalk.gray("Transmitter set as SAC admin"));

console.log(chalk.gray("Loading forwarder WASM..."));
const forwarderWasm = await loadWasmFile("forwarder");

const forwarderContract = new Contract({
  networkConfig,
  contractConfig: { wasm: forwarderWasm as any },
});

await forwarderContract.loadSpecFromWasm();

console.log(chalk.gray("Uploading forwarder WASM..."));
await forwarderContract.uploadWasm(txConfig);

console.log(chalk.gray("Deploying forwarder..."));
await forwarderContract.deploy({
  config: txConfig,
  constructorArgs: {
    asset: asset.contractId,
    transmitter: transmitter.getContractId(),
  },
});

console.log(`Forwarder ID: ${chalk.green(forwarderContract.getContractId())}`);

// 6. Save deployment data
await saveToJsonFile<DeploymentData>(
  {
    assetCode: asset.code,
    assetIssuerSk: issuer.secretKey(),
    transmitterId: transmitter.getContractId(),
    forwarderId: forwarderContract.getContractId(),
  },
  "deployment"
);

console.log(chalk.blue(" Setup Complete"));
