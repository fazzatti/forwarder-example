use soroban_sdk::{address_payload::AddressPayload, contracttype, Address, Bytes, BytesN, Env};

#[contracttype]
#[derive(Clone, Debug)]
pub struct Message {
    pub recipient: Address,
    pub destination_caller: Address,
    pub message_body: MessageBody,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct MessageBody {
    pub mint_recipient: Address,
    pub amount: i128,
    pub hook_data: Bytes,
}

/// Parse 32-byte payload into contract Address (C...)
fn contract_address_from_bytes32(env: &Env, bytes: &Bytes) -> Address {
    assert!(bytes.len() == 32, "expected 32 bytes");
    let mut arr = [0u8; 32];
    for i in 0..32 {
        arr[i] = bytes.get(i as u32).unwrap();
    }
    let bytes32 = BytesN::from_array(env, &arr);
    AddressPayload::ContractIdHash(bytes32).to_address(env)
}

/// Parse 32-byte big-endian uint256 into i128
/// CCTP uses uint256 (32 bytes) for amounts, Soroban uses i128 (16 bytes).
/// To simplify for this demo, we just read the lower 128 bits (last 16 bytes).
fn amount_from_bytes32(bytes: &Bytes) -> i128 {
    assert!(bytes.len() == 32, "expected 32 bytes");
    let mut arr = [0u8; 16];
    for i in 0..16 {
        arr[i] = bytes.get((16 + i) as u32).unwrap();
    }
    i128::from_be_bytes(arr)
}

/// Parse raw message bytes into Message struct
/// Header: recipient (32) | destination_caller (32)
/// Body: mint_recipient (32) | amount (32) | hook_data (remaining)
pub fn parse_message(env: &Env, message: &Bytes) -> Message {
    assert!(message.len() >= 128, "message too short");

    let recipient = contract_address_from_bytes32(env, &message.slice(0..32));
    let destination_caller = contract_address_from_bytes32(env, &message.slice(32..64));
    let mint_recipient = contract_address_from_bytes32(env, &message.slice(64..96));
    let amount = amount_from_bytes32(&message.slice(96..128));
    let hook_data = message.slice(128..);

    Message {
        recipient,
        destination_caller,
        message_body: MessageBody {
            mint_recipient,
            amount,
            hook_data,
        },
    }
}
