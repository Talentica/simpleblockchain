extern crate utils;
use exonum_crypto::Hash;
use exonum_merkledb::{impl_object_hash_for_binary_value, BinaryValue, ObjectHash};
use failure::Error;
use std::{borrow::Cow, convert::AsRef};

use utils::serializer::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct Wallet {
    nonce: u64,
    balance: u64,
    storage_root: Hash,
    code_hash: Hash,
}

impl Wallet {
    pub fn new() -> Wallet {
        Wallet {
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

impl BinaryValue for Wallet {
    fn to_bytes(&self) -> Vec<u8> {
        bincode::serialize(self).unwrap()
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Result<Self, Error> {
        bincode::deserialize(bytes.as_ref()).map_err(From::from)
    }
}
impl_object_hash_for_binary_value! { Wallet }

#[cfg(test)]
mod test_wallet {

    #[test]
    pub fn test_wallets() {
        use super::*;
        let mut wallet = Wallet::new();
        println!("{:?}", wallet);
        wallet.balance = 10;
        println!("{:?}", wallet);
    }
}
