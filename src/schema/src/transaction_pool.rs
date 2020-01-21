extern crate utils;

use exonum_crypto::Hash;
use exonum_merkledb::ObjectHash;

use crate::transaction::{SignedTransaction, Txn};
use std::collections::{BTreeMap, HashMap};

pub type TxnPoolKeyType = i64;
pub type TxnPoolValueType = SignedTransaction;

pub trait TxnPool {
    type H;
    type T;
    type U;
    fn new() -> Self;
    fn delete_txn_hash(&mut self, key: &Self::H);
    fn delete_txn_order(&mut self, key: &Self::T);
    fn pop_front(&mut self) -> Self::U;
    fn insert_op(&mut self, key: &Self::T, value: &Self::U);
    fn length_order_pool(&self) -> usize;
    fn length_hash_pool(&self) -> usize;
    fn get(&self, key: &Self::H) -> Option<&Self::U>;
    fn sync_pool(&mut self, txn_hash_vec: &Vec<Self::H>);
    fn sync_order_pool(&mut self, txn_hash_vec: &Vec<Self::H>);
    fn execute(&mut self, acc_data_base: &mut HashMap<String, u64>) -> Vec<Self::H>;
}

#[derive(Debug, Clone)]
pub struct TransactionPool {
    hash_pool: BTreeMap<Hash, TxnPoolValueType>,
    order_pool: BTreeMap<TxnPoolKeyType, TxnPoolValueType>,
}

impl TxnPool for TransactionPool {
    type H = Hash;
    type T = TxnPoolKeyType;
    type U = TxnPoolValueType;

    fn new() -> TransactionPool {
        TransactionPool {
            hash_pool: BTreeMap::new(),
            order_pool: BTreeMap::new(),
        }
    }

    fn delete_txn_hash(&mut self, key: &Self::H) {
        // let (key, value) = m.lock().unwrap().pop_first().unwrap(); // lock the mutex, remove a value, unlock
        if self.hash_pool.contains_key(key) {
            self.hash_pool.remove(key);
        }
    }

    fn delete_txn_order(&mut self, key: &Self::T) {
        // let (key, value) = m.lock().unwrap().pop_first().unwrap(); // lock the mutex, remove a value, unlock
        if self.order_pool.contains_key(key) {
            self.order_pool.remove(key);
        }
    }

    fn pop_front(&mut self) -> Self::U {
        let (first_key, first_value) = self.order_pool.iter().next().unwrap();
        let value = first_value.clone();
        let key = first_key.clone();
        self.order_pool.remove(&key);
        value
    }

    fn insert_op(&mut self, key: &Self::T, value: &Self::U) {
        self.hash_pool.insert(value.object_hash(), value.clone());
        self.order_pool.insert(key.clone(), value.clone());
    }

    fn length_order_pool(&self) -> usize {
        self.order_pool.len()
    }

    fn length_hash_pool(&self) -> usize {
        self.hash_pool.len()
    }

    fn get(&self, key: &Self::H) -> Option<&Self::U> {
        if self.hash_pool.contains_key(key) {
            return self.hash_pool.get(&key);
        } else {
            return Option::None;
        }
    }

    fn sync_pool(&mut self, txn_hash_vec: &Vec<Hash>) {
        for each_hash in txn_hash_vec.iter() {
            let txn: &SignedTransaction = self.get(each_hash).unwrap();
            let timestamp = txn
                .header
                .get(&String::from("timestamp"))
                .unwrap()
                .parse::<i64>()
                .unwrap();
            self.delete_txn_order(&timestamp);
            self.delete_txn_hash(each_hash);
        }
    }

    fn sync_order_pool(&mut self, txn_hash_vec: &Vec<Hash>) {
        // TODO: readd all txns which are deleted at the time of block proposal
        for each_hash in txn_hash_vec.iter() {
            let txn: SignedTransaction = self.get(each_hash).unwrap().clone();
            let timestamp = txn
                .header
                .get(&String::from("timestamp"))
                .unwrap()
                .parse::<i64>()
                .unwrap();
            self.order_pool.insert(timestamp, txn);
        }
    }

    fn execute(&mut self, acc_data_base: &mut HashMap<String, u64>) -> Vec<Hash> {
        let mut temp_vec = Vec::<Hash>::with_capacity(10);
        while temp_vec.len() < 10 && self.length_order_pool() > 0 {
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
        use chrono::prelude::Utc;
        use utils::keypair::{CryptoKeypair, Keypair};

        let mut transaction_pool = TransactionPool::new();
        let kp = Keypair::generate();
        let one = SignedTransaction::generate(&kp);
        let two = SignedTransaction::generate(&kp);
        let time_instant = Utc::now().timestamp_nanos();
        transaction_pool.insert_op(&time_instant, &one);
        let time_instant = Utc::now().timestamp_nanos();
        transaction_pool.insert_op(&time_instant, &two);

        let exexuted_pool = transaction_pool.execute(&mut HashMap::<String, u64>::new());
        println!("{:?}", exexuted_pool);
    }
}
