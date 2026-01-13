use crate::message::{parse_hook_data, parse_message};
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

    pub fn forward(env: Env, message: Bytes, attestation: Bytes) {
        let asset = Self::asset(env.clone());
        let transmitter_addr = Self::transmitter(env.clone());
        let token_client = token::Client::new(&env, &asset);

        let this_address = env.current_contract_address();
        let balance_before = token_client.balance(&this_address);

        call_transmitter(&env, &transmitter_addr, &message, &attestation);

        let balance_after = token_client.balance(&this_address);
        let amount_minted = balance_after - balance_before;

        transfer_to_recipient(&env, &asset, &this_address, &message, amount_minted);
    }
}

fn call_transmitter(env: &Env, transmitter: &Address, message: &Bytes, attestation: &Bytes) {
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

    MessageTransmitterClient::new(env, transmitter).receive_message(message, attestation);
}

fn transfer_to_recipient(
    env: &Env,
    asset: &Address,
    this: &Address,
    message: &Bytes,
    amount: i128,
) {
    let parsed = parse_message(env, message);
    let final_recipient = parse_hook_data(&parsed.message_body.hook_data);

    assert!(amount == parsed.message_body.amount, "amount mismatch");

    token::Client::new(env, asset).transfer(this, &final_recipient, &amount);
}
