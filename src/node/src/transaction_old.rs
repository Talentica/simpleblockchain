extern crate utils;
use chrono::prelude::*;
use std::collections::{vec_deque::VecDeque, HashMap};
use utils::keypair::{CryptoKeypair, Keypair, KeypairType, PublicKey, Verify};
use utils::serializer::{serialize, serialize_hash256, Deserialize, Serialize};

pub trait Txn {
    type T;
    type U;
    // generate trait is only for testing purpose
    fn generate(kp: &Self::U) -> Self::T;
    fn validate(&self) -> bool;
    fn sign(&self, kp: &Self::U) -> Vec<u8>;
}

#[derive(Clone, Debug, Serialize, Deserialize)]
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
    nonce: u64,
    from: String,
    to: String,
    fxn_call: String,
    // TODO:: payload is for fxn_call variables
    // update payload type and add/remove in future as per requirement
    payload: HashMap<String, String>,
    amount: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
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

pub trait TxnPool {
    type T;
    type U;
    
    fn new() -> Self::T;
    fn delete_op(&mut self, key: &String);
    fn pop_front(&mut self) -> (String, Self::U);
    fn insert_op(&mut self, value: &Self::U);
    fn length_op(&self) -> (usize, usize);
    fn execute(&mut self) -> Vec<Self::U>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionPool {
    hashmap: HashMap<String, Vec<u8>>,
    queue: VecDeque<String>,
}

impl TxnPool for TransactionPool {
    type T = TransactionPool;
    type U = Vec<u8>;

    fn new() -> TransactionPool {
        TransactionPool {
            hashmap: HashMap::new(),
            queue: VecDeque::new(),
        }
    }

    fn delete_op(&mut self, key: &String) {
        // let (key, value) = m.lock().unwrap().pop_first().unwrap(); // lock the mutex, remove a value, unlock
        if self.hashmap.contains_key(key) {
            self.hashmap.remove_entry(key);
        }
    }

    fn pop_front(&mut self) -> (String, Self::U) {
        while self.queue.len() > 0 {
            let key = self.queue.pop_back().unwrap();
            if self.hashmap.contains_key(&key) {
                let value = self.hashmap.get(&key).unwrap().clone();
                // self.hashmap.remove_entry(&key);
                self.delete_op(&key);
                return (key, value);
            }
        }
        panic!();
    }

    fn insert_op(&mut self, value: &Self::U) {
        let txn_hash = serialize_hash256(value);
        let mut hash_str = String::new();
        hash_str.push_str(&String::from_utf8_lossy(&txn_hash));

        self.hashmap.insert(hash_str.clone(), value.clone());
        self.queue.push_front(hash_str);
    }

    fn length_op(&self) -> (usize, usize) {
        (self.hashmap.len(), self.queue.len())
    }

    fn execute(&mut self) -> Vec<Self::U> {
        let mut temp_vec = Vec::<Vec<u8>>::with_capacity(10);
        while temp_vec.len() < 10 && self.length_op().1 > 0 {
            let txn = self.pop_front().1;
            temp_vec.push(txn);
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
        let txn_hash = serialize_hash256(&one);
        let mut txn_hash_str = String::new();
        txn_hash_str.push_str(&String::from_utf8_lossy(&txn_hash));
        transaction_pool.insert_op(&serialize(&one));
        let txn_hash = serialize_hash256(&one);
        let mut txn_hash_str = String::new();
        txn_hash_str.push_str(&String::from_utf8_lossy(&txn_hash));
        transaction_pool.insert_op(&serialize(&two));

        let exexuted_pool = transaction_pool.execute();
        println!("{:?}", exexuted_pool);
    }
}
