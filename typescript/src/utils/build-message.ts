import { Buffer } from "buffer";
import {
  ContractId,
  Ed25519PublicKey,
  MuxedAddress,
  StrKey,
} from "@colibri/core";
import { Address } from "stellar-sdk";

function contractIdToBytes32(contractId: string): Uint8Array {
  return StrKey.decodeContract(contractId);
}

function amountToBytes32(amount: bigint): Uint8Array {
  const buf = new Uint8Array(32);
  const view = new DataView(buf.buffer);
  view.setBigUint64(16, amount >> 64n, false);
  view.setBigUint64(24, amount & 0xffffffffffffffffn, false);
  return buf;
}

function hookDataToXdr(
  hookData: Ed25519PublicKey | ContractId | MuxedAddress
): Uint8Array {
  const address = Address.fromString(hookData);
  const scVal = address.toScVal();
  return scVal.toXDR();
}

/**
 * Build CCTP message bytes
 * Header: recipient (32) | destination_caller (32)
 * Body: mint_recipient (32) | amount (32) | hook_data (variable)
 *
 * @param encodeAsXdr - if true, hook_data is XDR encoded ScVal::Address (supports MuxedAddress)
 *                      if false, hook_data is strkey string (G.../C... 56 chars)
 */
export function buildMessage(
  forwarderId: string,
  amount: bigint,
  hookData: Ed25519PublicKey | ContractId | MuxedAddress,
  encodeAsXdr: boolean = false
): Buffer {
  const recipient = contractIdToBytes32(forwarderId);
  const destinationCaller = contractIdToBytes32(forwarderId);
  const mintRecipient = contractIdToBytes32(forwarderId);
  const amountBytes = amountToBytes32(amount);

  const hookDataBytes = encodeAsXdr
    ? hookDataToXdr(hookData)
    : new TextEncoder().encode(hookData);

  const message = new Uint8Array(32 + 32 + 32 + 32 + hookDataBytes.length);
  message.set(recipient, 0);
  message.set(destinationCaller, 32);
  message.set(mintRecipient, 64);
  message.set(amountBytes, 96);
  message.set(hookDataBytes, 128);

  return Buffer.from(message);
}
