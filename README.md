# Forwarder Example

Prototype demonstrating a dummy forwarder contract that receives minted tokens and forwards them to a final recipient specified in hook_data.

## Hook Data

The forwarder contract and scripts expect `hook_data` to contain a Stellar address as string bytes (G/C/M). The parsing helper converts the string into a `MuxedAddress`, and the contract forwards using that `MuxedAddress`.

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

Each script supports the strkey string flow via Deno tasks:

- **C address (contract):**
  - strkey: `deno task forward:c:str`
- **G address (account):**
  - strkey: `deno task forward:g:str`
- **M address (muxed):**
  - strkey: `deno task forward:m:str`

### Example

```bash
# Forward to a muxed address (as a muxed string)
deno task forward:m:str

# Forward to a contract address using strkey encoding
deno task forward:c:str
```

## Message Format

```
| recipient (32 bytes) | destination_caller (32 bytes) | mint_recipient (32 bytes) | amount (32 bytes) | hook_data (variable) |
|-------- Header ------|----------------------------- Body -------------------------------------------------|
```

## Notes

- `parse_hook_data` converts strkey/muxed strings into `MuxedAddress`, and the contract uses that for the transfer.
- The amount assertion is performed before the transfer.
- See the scripts in `typescript/src/` for usage details and CLI flags.
