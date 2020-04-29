extern crate utils;
use super::state::CryptoState;
pub use crate::user_messages::CryptoTransaction;
use exonum_crypto::Hash;
use exonum_merkledb::ObjectHash;
pub use sdk::signed_transaction::SignedTransaction;
use sdk::state::State;
use sdk::traits::{AppHandler, StateContext};
use std::collections::HashMap;
use std::convert::AsRef;
use std::time::SystemTime;
use utils::keypair::{CryptoKeypair, Keypair, KeypairType, PublicKey, Verify};
use utils::logger::*;
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
        state_context.put_txn(&self.get_hash(), self.clone());
        flag
    }
}

pub trait ModuleTraits {
    fn transfer(&self, state_context: &mut dyn StateContext) -> bool;
    fn mint(&self, state_context: &mut dyn StateContext) -> bool;
}

impl ModuleTraits for CryptoTransaction {
    fn transfer(&self, state_context: &mut dyn StateContext) -> bool {
        if self.validate() {
            if self.from == self.to {
                info!("self transfer transaction not allowed");
                return false;
            }
            let mut from_state: State = match state_context.get(&self.from) {
                Some(state) => state,
                None => return false,
            };
            let mut from_wallet: CryptoState = deserialize(from_state.get_data().as_slice());
            if self.nonce != from_wallet.get_nonce() + 1 {
                info!(
                    "transfer txn nonce mismatched {:?} {:?}",
                    self.nonce,
                    from_wallet.get_nonce() + 1
                );
                return false;
            }
            if from_wallet.get_balance() >= self.amount {
                let mut to_state: State = match state_context.get(&self.to) {
                    Some(state) => state,
                    None => {
                        let crypto_state: CryptoState = CryptoState::new();
                        let mut state: State = State::new();
                        state.set_data(&serialize(&crypto_state));
                        state
                    }
                };
                let mut to_wallet: CryptoState = deserialize(to_state.get_data().as_slice());
                if !to_wallet.add_balance(self.amount) {
                    return false;
                }
                to_state.set_data(&serialize(&to_wallet));

                from_wallet.deduct_balance(self.amount);
                from_wallet.increase_nonce();
                from_state.set_data(&serialize(&from_wallet));

                state_context.put(&self.to.clone(), to_state);
                state_context.put(&self.from.clone(), from_state);
                return true;
            }
        }
        false
    }

    fn mint(&self, state_context: &mut dyn StateContext) -> bool {
        if self.validate() {
            let mut state: State = match state_context.get(&self.from) {
                Some(state) => state,
                None => {
                    let crypto_state: CryptoState = CryptoState::new();
                    let mut state: State = State::new();
                    state.set_data(&serialize(&crypto_state));
                    state
                }
            };
            let mut wallet: CryptoState = deserialize(state.get_data().as_slice());
            if self.nonce != wallet.get_nonce() + 1 {
                info!(
                    "transfer txn nonce mismatched {:?} {:?}",
                    self.nonce,
                    wallet.get_nonce() + 1
                );
                return false;
            }
            if !wallet.add_balance(self.amount) {
                return false;
            }
            wallet.increase_nonce();
            state.set_data(&serialize(&wallet));
            state_context.put(&self.from.clone(), state);
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
        file_logger_init_from_yml(&String::from("log.yml"));
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
