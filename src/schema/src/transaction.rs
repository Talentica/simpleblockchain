extern crate utils;
use exonum_crypto::Hash;
pub use exonum_merkledb::{impl_object_hash_for_binary_value, BinaryValue, ObjectHash};
use failure::Error;
use std::time::SystemTime;
use std::{borrow::Cow, convert::AsRef};

use std::collections::HashMap;
use utils::keypair::{CryptoKeypair, Keypair, KeypairType, PublicKey, Verify};
use utils::serializer::{serialize, Deserialize, Serialize};
pub trait Txn {
    type T;
    type U;
    // generate trait is only for testing purpose
    fn generate(kp: &Self::U) -> Self::T;
    fn validate(&self) -> bool;
    fn sign(&self, kp: &Self::U) -> Vec<u8>;
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum DataTypes {
    String,
    Vec(String),
    Number(u64),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Transaction {
    /* TODO:
    // Priority for a transaction. Additive. Higher is better.
    pub type TransactionPriority = u64;
    // Minimum number of blocks a transaction will remain valid for.
    // `TransactionLongevity::max_value()` means "forever".
    pub type TransactionLongevity = u64;
    // Tag for a transaction. No two transactions with the same tag should be placed on-chain.
    pub type TransactionTag = Vec<u8>;
    */
    pub nonce: u64,
    pub from: String,
    pub to: String,
    pub fxn_call: String,
    // TODO:: payload is for fxn_call variables
    // update payload type and add/remove in future as per requirement
    pub payload: Vec<DataTypes>,
    pub amount: u64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SignedTransaction {
    pub txn: Transaction,
    // TODO::
    // update header type and add/remove in future as per requirement
    pub header: HashMap<String, String>,
    pub signature: Vec<u8>,
}

impl Txn for Transaction {
    type T = Transaction;
    type U = KeypairType;

    fn validate(&self) -> bool {
        unimplemented!();
    }

    fn sign(&self, kp: &KeypairType) -> Vec<u8> {
        let ser_txn = serialize(&self);
        let sign = Keypair::sign(&kp, &ser_txn);
        sign
    }

    fn generate(kp: &KeypairType) -> Transaction {
        let from: String = hex::encode(kp.public().encode());
        let to_add_kp = Keypair::generate();
        let to: String = hex::encode(to_add_kp.public().encode());
        Transaction {
            nonce: 0,
            from,
            to,
            amount: 32,
            fxn_call: String::from("transfer"),
            payload: vec![],
        }
    }
}

impl Txn for SignedTransaction {
    type T = SignedTransaction;
    type U = KeypairType;

    fn validate(&self) -> bool {
        let ser_txn = serialize(&self.txn);
        PublicKey::verify_from_encoded_pk(&self.txn.from, &ser_txn, &self.signature.as_ref())
    }

    fn sign(&self, kp: &KeypairType) -> Vec<u8> {
        let ser_txn = serialize(&self.txn);
        let sign = Keypair::sign(&kp, &ser_txn);
        sign
    }

    fn generate(kp: &KeypairType) -> SignedTransaction {
        let from: String = hex::encode(kp.public().encode());
        let to_add_kp = Keypair::generate();
        let to: String = hex::encode(to_add_kp.public().encode());
        let txn: Transaction = Transaction {
            nonce: 0,
            from,
            to,
            amount: 32,
            fxn_call: String::from("transfer"),
            payload: vec![],
        };
        let txn_sign = txn.sign(&kp);
        let mut header = HashMap::default();
        let time_stamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_micros();
        header.insert("timestamp".to_string(), time_stamp.to_string());
        SignedTransaction {
            txn,
            signature: txn_sign,
            header,
        }
    }
}

impl BinaryValue for SignedTransaction {
    fn to_bytes(&self) -> Vec<u8> {
        bincode::serialize(self).unwrap()
    }

    fn from_bytes(bytes: Cow<'_, [u8]>) -> Result<Self, Error> {
        bincode::deserialize(bytes.as_ref()).map_err(From::from)
    }
}

impl_object_hash_for_binary_value! { SignedTransaction}

#[cfg(test)]
mod tests_transactions {

    #[test]
    pub fn main_transaction() {
        use super::*;
        let kp = Keypair::generate();
        let signed_txn = SignedTransaction::generate(&kp);
        let validate_txn = signed_txn.validate();
        println!("{:?}", validate_txn);
    }
}
