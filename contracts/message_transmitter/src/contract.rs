use crate::message::parse_message;
use soroban_sdk::{contract, contractimpl, contracttype, token, Address, Bytes, Env};

#[contracttype]
pub enum DataKey {
    Asset,
}

#[contract]
pub struct MessageTransmitter;

#[contractimpl]
impl MessageTransmitter {
    pub fn __constructor(env: Env, asset: Address) {
        env.storage().instance().set(&DataKey::Asset, &asset);
    }

    pub fn asset(env: Env) -> Address {
        env.storage().instance().get(&DataKey::Asset).unwrap()
    }

    pub fn receive_message(env: Env, message: Bytes) {
        let msg = parse_message(&env, &message);

        // Ensure the destination caller has authorized this call
        msg.destination_caller.require_auth();

        let asset: Address = env.storage().instance().get(&DataKey::Asset).unwrap();
        token::StellarAssetClient::new(&env, &asset)
            .mint(&msg.message_body.mint_recipient, &msg.message_body.amount);
    }
}
