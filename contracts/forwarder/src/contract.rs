use crate::message::{parse_hook_data, parse_hook_data_xdr, parse_message};
use soroban_sdk::{
    auth::{ContractContext, InvokerContractAuthEntry, SubContractInvocation},
    contract, contractclient, contractimpl, contracttype, token, vec, Address, Bytes, Env, IntoVal,
    Symbol,
};

#[contracttype]
#[derive(Clone, Debug)]
pub enum DataKey {
    Asset,
    Transmitter,
}

#[allow(dead_code)]
#[contractclient(name = "MessageTransmitterClient")]
trait MessageTransmitter {
    fn receive_message(env: Env, message: Bytes, attestation: Bytes);
}

#[contract]
pub struct Forwarder;

#[contractimpl]
impl Forwarder {
    pub fn __constructor(env: Env, asset: Address, transmitter: Address) {
        env.storage().instance().set(&DataKey::Asset, &asset);
        env.storage()
            .instance()
            .set(&DataKey::Transmitter, &transmitter);
    }

    pub fn asset(env: Env) -> Address {
        env.storage().instance().get(&DataKey::Asset).unwrap()
    }

    pub fn transmitter(env: Env) -> Address {
        env.storage().instance().get(&DataKey::Transmitter).unwrap()
    }

    /// Forward tokens to final recipient from hook_data
    /// xdr_hook_data: if true, hook_data is XDR serialized ScVal::Address (supports MuxedAddress)
    ///                if false, hook_data is strkey string (G.../C... 56 chars)
    pub fn forward(env: Env, message: Bytes, xdr_hook_data: bool) {
        let asset = Self::asset(env.clone());
        let transmitter_addr = Self::transmitter(env.clone());
        let token_client = token::Client::new(&env, &asset);

        let this_address = env.current_contract_address();
        let balance_before = token_client.balance(&this_address);

        call_transmitter(&env, &transmitter_addr, &message);

        let balance_after = token_client.balance(&this_address);
        let amount_minted = balance_after - balance_before;

        let parsed = parse_message(&env, &message);
        // Determine recipient type (Address or MuxedAddress)
        let recipient = if xdr_hook_data {
            Recipient::Muxed(parse_hook_data_xdr(&env, &parsed.message_body.hook_data))
        } else {
            Recipient::Address(parse_hook_data(&parsed.message_body.hook_data))
        };
        assert!(
            amount_minted == parsed.message_body.amount,
            "amount mismatch"
        );

        transfer_to_recipient(&env, &asset, &recipient, amount_minted);
    }
}

fn call_transmitter(env: &Env, transmitter: &Address, message: &Bytes) {
    let attestation = Bytes::new(env);
    env.authorize_as_current_contract(vec![
        env,
        InvokerContractAuthEntry::Contract(SubContractInvocation {
            context: ContractContext {
                contract: transmitter.clone(),
                fn_name: Symbol::new(env, "receive_message"),
                args: (message.clone(), attestation.clone()).into_val(env),
            },
            sub_invocations: vec![env],
        }),
    ]);

    MessageTransmitterClient::new(env, transmitter).receive_message(message, &attestation);
}

#[derive(Debug)]
enum Recipient {
    Address(Address),
    Muxed(soroban_sdk::MuxedAddress),
}

fn transfer_to_recipient(env: &Env, asset: &Address, recipient: &Recipient, amount_minted: i128) {
    let client = token::Client::new(env, asset);
    match recipient {
        Recipient::Address(addr) => {
            client.transfer(&env.current_contract_address(), addr, &amount_minted);
        }
        Recipient::Muxed(muxed) => {
            client.transfer(&env.current_contract_address(), muxed, &amount_minted);
        }
    }
}
