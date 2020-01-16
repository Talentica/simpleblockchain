extern crate utils;
use chrono::prelude::*;
use exonum_crypto::Hash;
use exonum_merkledb::{impl_object_hash_for_binary_value, BinaryValue, ObjectHash};
use failure::Error;
use std::{borrow::Cow, convert::AsRef};

use std::collections::{BTreeMap, HashMap};
use std::time::Instant;
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
    pub payload: HashMap<String, String>,
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
            payload: HashMap::default(),
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
            payload: HashMap::default(),
        };
        let txn_sign = txn.sign(&kp);
        let mut header = HashMap::default();
        header.insert("timestamp".to_string(), Local::now().to_string());
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

pub type TxnPoolKeyType = Instant;
pub type TxnPoolValueType = SignedTransaction;

pub trait TxnPool {
    type T;
    type U;
    fn new() -> Self;
    fn delete_op(&mut self, key: &Self::T);
    fn pop_front(&mut self) -> Self::U;
    fn insert_op(&mut self, key: &Self::T, value: &Self::U);
    fn length_op(&self) -> usize;
    fn get(&self, key: &Self::T) -> Option<&Self::U>;
    fn execute(&mut self, acc_data_base: &mut HashMap<String, u64>) -> Vec<Hash>;
}

#[derive(Debug, Clone)]
pub struct TransactionPool {
    pub pool: BTreeMap<TxnPoolKeyType, TxnPoolValueType>,
}

impl TxnPool for TransactionPool {
    type T = TxnPoolKeyType;
    type U = TxnPoolValueType;

    fn new() -> TransactionPool {
        TransactionPool {
            pool: BTreeMap::new(),
        }
    }

    fn delete_op(&mut self, key: &Self::T) {
        // let (key, value) = m.lock().unwrap().pop_first().unwrap(); // lock the mutex, remove a value, unlock
        if self.pool.contains_key(key) {
            self.pool.remove(key);
        }
    }

    fn pop_front(&mut self) -> Self::U {
        let (first_key, first_value) = self.pool.iter().next().unwrap();
        let value = first_value.clone();
        let key = first_key.clone();
        self.delete_op(&key);
        value
    }

    fn insert_op(&mut self, key: &Self::T, value: &Self::U) {
        self.pool.insert(key.clone(), value.clone());
    }

    fn length_op(&self) -> usize {
        self.pool.len()
    }

    fn get(&self, key: &Self::T) -> Option<&Self::U> {
        if self.pool.contains_key(key) {
            return self.pool.get(&key);
        } else {
            return Option::None;
        }
    }

    fn execute(&mut self, acc_data_base: &mut HashMap<String, u64>) -> Vec<Hash> {
        let mut temp_vec = Vec::<Hash>::with_capacity(10);
        while temp_vec.len() < 10 && self.length_op() > 0 {
            let txn: TxnPoolValueType = self.pop_front();
            if txn.validate() {
                if acc_data_base.contains_key(&txn.txn.from) {
                    let from_bal = acc_data_base.get(&txn.txn.from).unwrap().clone();
                    if from_bal > txn.txn.amount {
                        if acc_data_base.contains_key(&txn.txn.to) {
                            let new_bal =
                                txn.txn.amount + acc_data_base.get(&txn.txn.to).unwrap().clone();
                            acc_data_base.insert(txn.txn.to.clone(), new_bal);
                        } else {
                            acc_data_base.insert(txn.txn.to.clone(), txn.txn.amount.clone());
                        }
                        acc_data_base
                            .insert(txn.txn.from.clone(), from_bal - txn.txn.amount.clone());
                        temp_vec.push(txn.object_hash());
                    }
                }
            }
        }
        temp_vec
    }
}

#[cfg(test)]
mod tests_transactions {

    #[test]
    pub fn main_transaction() {
        use super::*;
        let mut transaction_pool = TransactionPool::new();
        let kp = Keypair::generate();
        let one = SignedTransaction::generate(&kp);
        let two = SignedTransaction::generate(&kp);
        let time_instant = Instant::now();
        transaction_pool.insert_op(&time_instant, &one);
        let time_instant = Instant::now();
        transaction_pool.insert_op(&time_instant, &two);

        let exexuted_pool = transaction_pool.execute(&mut HashMap::<String, u64>::new());
        println!("{:?}", exexuted_pool);
    }
}
