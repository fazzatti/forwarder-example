# Forwarder Example

Prototype demonstrating a dummy forwarder contract that receives minted tokens and forwards them to a final recipient specified in hook_data.

## Hook Data Encoding Modes

The forwarder contract and scripts support two encoding modes for the recipient in `hook_data`:

- **strkey**: Standard Stellar G.../C.../M... address as string bytes
- **XDR**: Serialized `ScVal::Address` (supports MuxedAddress, i.e. M...)

The encoding mode is selected via the `--xdr` flag in the TypeScript scripts, or by using the appropriate Deno task variant (see below).

### Muxed Address Support and behavior

- Both parsing helpers now return a `MuxedAddress` (so strkey strings that encode a G/C/M address are converted into a `MuxedAddress`).
- The contract always forwards using a `MuxedAddress` instance (no duplicate transfer logic).
- To forward to a muxed (M...) address, prefer the XDR encoding mode (`forward:m:xdr`).
- The strkey mode for `forward:m` remains unsupported in the example and will exit with a warning when attempted (`forward:m:str`).

## Project Structure

```
├── contracts/
│   ├── forwarder/           # Receives minted tokens and forwards to final recipient
│   └── message_transmitter/ # Mock MessageTransmitter (mints tokens)
│
└── typescript/              # Test scripts (setup, forward:c, forward:g)
```

- **recipient**: Contract that receives the message (unused in this demo)
- **destination_caller**: Contract authorized to call receive_message (forwarder)
- **mint_recipient**: Address that receives minted tokens (forwarder)
- **amount**: Token amount (uint256, we read lower 128 bits)
- **hook_data**: Final recipient as 56-byte strkey (G.../C...)

## Prerequisites

- [Deno](https://deno.land/) runtime installed
- [Stellar CLI](https://developers.stellar.org/docs/tools/developer-tools/cli/stellar-cli) installed

## Build Contracts

```bash
stellar contract build
```

## TypeScript Scripts & Deno Tasks

### Setup (deploy contracts)

```bash
deno task setup
```

### Forwarding Tasks

Each script supports both strkey and XDR encoding via Deno tasks:

- **C address (contract):**
  - strkey: `deno task forward:c:str`
  - XDR: `deno task forward:c:xdr`
- **G address (account):**
  - strkey: `deno task forward:g:str`
  - XDR: `deno task forward:g:xdr`
- **M address (muxed):**
  - XDR: `deno task forward:m:xdr`
  - strkey: `deno task forward:m:str` (not supported, will warn and exit)

### Example

```bash
# Forward to a muxed address using XDR encoding
deno task forward:m:xdr

# Forward to a contract address using strkey encoding
deno task forward:c:str
```

## Message Format

```
| recipient (32 bytes) | destination_caller (32 bytes) | mint_recipient (32 bytes) | amount (32 bytes) | hook_data (variable) |
|-------- Header ------|----------------------------- Body -------------------------------------------------|
```

## Notes

- Both `parse_hook_data` (strkey) and `parse_hook_data_xdr` (XDR) now yield a `MuxedAddress`, and the contract uses that for the transfer.
- The amount assertion is performed before the transfer.
- See the scripts in `typescript/src/` for usage details and CLI flags.
