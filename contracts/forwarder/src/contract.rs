use crate::message::{parse_hook_data, parse_message};
use soroban_sdk::{
    auth::{ContractContext, InvokerContractAuthEntry, SubContractInvocation},
    contract, contractclient, contractimpl, contracttype, token, vec, Address, Bytes, Env, IntoVal,
    MuxedAddress, Symbol,
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
    fn receive_message(env: Env, message: Bytes);
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

    /// Forward tokens to final recipient from hook_data
    /// hook_data is expected to be a strkey string representing a C.../G.../M... address.
    pub fn forward(env: Env, message: Bytes) {
        let asset = asset(env.clone());
        let token_client = token::Client::new(&env, &asset);
        let this_address = env.current_contract_address();

        let balance_before = token_client.balance(&this_address);

        call_transmitter(&env, &message);

        let balance_after = token_client.balance(&this_address);
        let amount_minted = balance_after - balance_before;

        let parsed = parse_message(&env, &message);
        let recipient: MuxedAddress = parse_hook_data(&parsed.message_body.hook_data);
        assert!(
            amount_minted == parsed.message_body.amount,
            "amount mismatch"
        );
        transfer_to_recipient(&env, &recipient, amount_minted);
    }
}

pub fn asset(env: Env) -> Address {
    env.storage().instance().get(&DataKey::Asset).unwrap()
}

pub fn transmitter(env: Env) -> Address {
    env.storage().instance().get(&DataKey::Transmitter).unwrap()
}

fn call_transmitter(env: &Env, message: &Bytes) {
    let transmitter_addr = transmitter(env.clone());
    env.authorize_as_current_contract(vec![
        env,
        InvokerContractAuthEntry::Contract(SubContractInvocation {
            context: ContractContext {
                contract: transmitter_addr.clone(),
                fn_name: Symbol::new(env, "receive_message"),
                args: (message.clone(),).into_val(env),
            },
            sub_invocations: vec![env],
        }),
    ]);

    MessageTransmitterClient::new(env, &transmitter_addr).receive_message(message);
}

fn transfer_to_recipient(env: &Env, recipient: &MuxedAddress, amount_minted: i128) {
    let asset = asset(env.clone());
    let client = token::Client::new(env, &asset);
    client.transfer(&env.current_contract_address(), recipient, &amount_minted);
}
