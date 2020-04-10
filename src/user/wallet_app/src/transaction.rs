extern crate utils;
use super::state::CryptoState;
pub use crate::user_messages::CryptoTransaction;
use exonum_crypto::Hash;
use exonum_merkledb::ObjectHash;
use generic_traits::signed_transaction::SignedTransaction;
use generic_traits::state::State;
use generic_traits::traits::{AppHandler, StateContext};
use std::collections::HashMap;
use std::convert::AsRef;
use std::time::SystemTime;
use utils::keypair::{CryptoKeypair, Keypair, KeypairType, PublicKey, Verify};
use utils::serializer::{deserialize, serialize};

const APPNAME: &str = "Cryptocurrency";

trait StateTraits {
    fn execute(&self, state_context: &mut dyn StateContext) -> bool;
}

pub trait TransactionTrait<T> {
    // generate trait is only for testing purpose
    fn generate(kp: &KeypairType) -> T;
    fn validate(&self) -> bool;
    fn sign(&self, kp: &KeypairType) -> Vec<u8>;
    fn get_hash(&self) -> exonum_crypto::Hash;
}

impl TransactionTrait<CryptoTransaction> for CryptoTransaction {
    fn validate(&self) -> bool {
        // add application validation logic if needed
        true
    }

    fn sign(&self, kp: &KeypairType) -> Vec<u8> {
        let ser_txn = serialize(&self);
        let sign = Keypair::sign(&kp, &ser_txn);
        sign
    }

    fn generate(kp: &KeypairType) -> CryptoTransaction {
        let from: String = hex::encode(kp.public().encode());
        let to_add_kp = Keypair::generate();
        let to: String = hex::encode(to_add_kp.public().encode());
        CryptoTransaction {
            nonce: 0,
            from,
            to,
            amount: 32,
            fxn_call: String::from("transfer"),
        }
    }

    fn get_hash(&self) -> Hash {
        self.object_hash()
    }
}

impl TransactionTrait<SignedTransaction> for SignedTransaction {
    fn validate(&self) -> bool {
        let txn = deserialize::<CryptoTransaction>(&self.txn);
        PublicKey::verify_from_encoded_pk(&txn.from, &self.txn, &self.signature.as_ref())
    }

    fn sign(&self, kp: &KeypairType) -> Vec<u8> {
        Keypair::sign(&kp, &self.txn)
    }

    fn generate(kp: &KeypairType) -> SignedTransaction {
        let from: String = hex::encode(kp.public().encode());
        let to_add_kp = Keypair::generate();
        let to: String = hex::encode(to_add_kp.public().encode());
        let txn: CryptoTransaction = CryptoTransaction {
            nonce: 0,
            from,
            to,
            amount: 32,
            fxn_call: String::from("transfer"),
        };
        let txn_sign = txn.sign(&kp);
        let mut header = HashMap::default();
        let time_stamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_micros();
        header.insert("timestamp".to_string(), time_stamp.to_string());
        SignedTransaction {
            txn: serialize(&txn),
            app_name: String::from(APPNAME),
            signature: txn_sign,
            header,
        }
    }

    fn get_hash(&self) -> Hash {
        self.object_hash()
    }
}

impl StateTraits for SignedTransaction {
    fn execute(&self, state_context: &mut dyn StateContext) -> bool {
        let mut flag: bool = false;
        if self.validate() {
            let txn = deserialize::<CryptoTransaction>(&self.txn);
            let crypto_txn = &txn.clone() as &dyn ModuleTraits;
            if txn.fxn_call == String::from("transfer") {
                flag = crypto_txn.transfer(state_context);
            } else if txn.fxn_call == String::from("mint") {
                flag = crypto_txn.mint(state_context);
            } else {
            }
        }
        if flag {
            state_context.put_txn(&self.get_hash(), self.clone());
            flag
        } else {
            false
        }
    }
}

pub trait ModuleTraits {
    fn transfer(&self, state_context: &mut dyn StateContext) -> bool;
    fn mint(&self, state_context: &mut dyn StateContext) -> bool;
}

impl ModuleTraits for CryptoTransaction {
    fn transfer(&self, state_context: &mut dyn StateContext) -> bool {
        if self.validate() {
            if state_context.contains(&self.from) {
                let mut from_wallet: CryptoState =
                    deserialize::<CryptoState>(state_context.get(&self.from).unwrap().get_data());
                if from_wallet.get_balance() > self.amount {
                    if state_context.contains(&self.to) {
                        let mut to_wallet: CryptoState = deserialize::<CryptoState>(
                            state_context.get(&self.to).unwrap().get_data(),
                        );
                        to_wallet.add_balance(self.amount);
                        let mut new_state = State::new();
                        new_state.set_data(&serialize(&to_wallet));
                        state_context.put(&self.to.clone(), new_state);
                    } else {
                        let mut to_wallet = CryptoState::new();
                        to_wallet.add_balance(self.amount);
                        let mut new_state = State::new();
                        new_state.set_data(&serialize(&to_wallet));
                        state_context.put(&self.to.clone(), new_state);
                    }
                    from_wallet.deduct_balance(self.amount);
                    from_wallet.increase_nonce();
                    let mut new_state = State::new();
                    new_state.set_data(&serialize(&from_wallet));
                    state_context.put(&self.from.clone(), new_state);
                    return true;
                }
            }
        }
        false
    }

    fn mint(&self, state_context: &mut dyn StateContext) -> bool {
        if self.validate() {
            if state_context.contains(&self.to) {
                let mut new_state = state_context.get(&self.to).unwrap();
                let mut to_wallet: CryptoState = deserialize::<CryptoState>(new_state.get_data());
                to_wallet.add_balance(self.amount);
                new_state.set_data(&serialize(&to_wallet));
                state_context.put(&self.to.clone(), new_state);
            } else {
                let mut to_wallet = CryptoState::new();
                to_wallet.add_balance(self.amount);
                let mut new_state = State::new();
                new_state.set_data(&serialize(&to_wallet));
                state_context.put(&self.to.clone(), new_state);
            }
            return true;
        }
        false
    }
}

pub struct CryptoApp {
    name: String,
}

impl CryptoApp {
    pub fn new(s: &String) -> CryptoApp {
        CryptoApp { name: s.clone() }
    }
}

impl AppHandler for CryptoApp {
    fn execute(&self, txn: &SignedTransaction, state_context: &mut dyn StateContext) -> bool {
        let st = txn as &dyn StateTraits;
        st.execute(state_context)
    }

    fn name(&self) -> String {
        self.name.clone()
    }
}

#[no_mangle]
pub fn register_app() -> Box<dyn AppHandler + Send> {
    Box::new(CryptoApp::new(&String::from(APPNAME)))
}
