extern crate utils;
use exonum_crypto::Hash;
use std::convert::AsRef;

use utils::serializer::{Deserialize, Serialize};

#[derive(
    Clone, Copy, Debug, PartialEq, Serialize, Deserialize, Default, BinaryValue, ObjectHash,
)]
#[binary_value(codec = "bincode")]
pub struct State {
    nonce: u64,
    balance: u64,
    storage_root: Hash,
    code_hash: Hash,
}

impl State {
    pub fn new() -> State {
        State {
            nonce: 0,
            balance: 0,
            storage_root: Hash::zero(),
            code_hash: Hash::zero(),
        }
    }

    pub fn get_nonce(&self) -> u64 {
        self.nonce
    }

    pub fn get_balance(&self) -> u64 {
        self.balance
    }

    pub fn get_storage_root(&self) -> Hash {
        self.storage_root
    }

    pub fn get_code_hash(&self) -> Hash {
        self.code_hash
    }

    pub fn increase_nonce(&mut self) {
        self.nonce = self.nonce + 1;
    }

    pub fn deduct_balance(&mut self, amount: u64) {
        self.balance = self.balance - amount;
    }

    pub fn add_balance(&mut self, amount: u64) {
        if self.balance > self.balance + amount {
            panic!("do balance check before fxn calling");
        } else {
            self.balance = self.balance + amount;
        }
    }

    pub fn set_storage_root(&mut self, new_storage_root: Hash) {
        self.storage_root = new_storage_root;
    }

    pub fn set_code_hash(&mut self, new_code_hash: Hash) {
        self.code_hash = new_code_hash;
    }
}

#[cfg(test)]
mod test_state {

    #[test]
    pub fn test_states() {
        use super::*;
        let mut state = State::new();
        debug!("{:?}", state);
        state.balance = 10;
        debug!("{:?}", state);
    }
}
