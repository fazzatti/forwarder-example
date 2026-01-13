import { ContractId, Ed25519SecretKey } from "@colibri/core";

export type DeploymentData = {
  assetCode: string;
  assetIssuerSk: Ed25519SecretKey;
  transmitterId: ContractId;
  forwarderId: ContractId;
};
