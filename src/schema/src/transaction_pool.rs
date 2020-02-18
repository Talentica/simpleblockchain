extern crate utils;

use exonum_crypto::Hash;
use exonum_merkledb::ObjectHash;

use crate::transaction::SignedTransaction;
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};
pub type TxnPoolKeyType = u128;
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
}

/**
 * BTreeMap is used here for in-order push-pop values and at the same time, search operation also supported.
*/
/// TransactionPool object to maintain in-coming txn and txn-order.
#[derive(Debug, Clone)]
pub struct TransactionPool {
    hash_pool: BTreeMap<Hash, TxnPoolValueType>,
    pub order_pool: BTreeMap<TxnPoolKeyType, TxnPoolValueType>,
}

impl TxnPool for TransactionPool {
    type H = Hash;
    type T = TxnPoolKeyType;
    type U = TxnPoolValueType;

    /// this function will create a new instance of transcation pool object
    fn new() -> TransactionPool {
        TransactionPool {
            hash_pool: BTreeMap::new(),
            order_pool: BTreeMap::new(),
        }
    }

    /// this function will delete txn using hash if present, from hash_pool
    fn delete_txn_hash(&mut self, key: &Self::H) {
        if self.hash_pool.contains_key(key) {
            self.hash_pool.remove(key);
        }
    }

    /// this function will delete txn using order_value if present, from order_pool
    fn delete_txn_order(&mut self, key: &Self::T) {
        if self.order_pool.contains_key(key) {
            self.order_pool.remove(key);
        }
    }

    /// this function will pop value in fifo order from order_pool
    fn pop_front(&mut self) -> Self::U {
        let (first_key, first_value) = self.order_pool.iter().next().unwrap();
        let value = first_value.clone();
        let key = first_key.clone();
        self.order_pool.remove(&key);
        value
    }

    /// this function will push value in both (hash & order) pool
    fn insert_op(&mut self, key: &Self::T, value: &Self::U) {
        self.hash_pool.insert(value.object_hash(), value.clone());
        self.order_pool.insert(key.clone(), value.clone());
    }

    /// length of order_pool
    fn length_order_pool(&self) -> usize {
        self.order_pool.len()
    }

    /// length of hash_pool
    fn length_hash_pool(&self) -> usize {
        self.hash_pool.len()
    }

    /// get transaction usinng hash from hash_pool
    fn get(&self, key: &Self::H) -> Option<&Self::U> {
        if self.hash_pool.contains_key(key) {
            return self.hash_pool.get(&key);
        } else {
            return Option::None;
        }
    }

    /// sync both (hash & order ) pool when block committed is created by the other node
    fn sync_pool(&mut self, txn_hash_vec: &Vec<Hash>) {
        for each_hash in txn_hash_vec.iter() {
            let txn: &SignedTransaction = self.get(each_hash).unwrap();
            let timestamp = txn
                .header
                .get(&String::from("timestamp"))
                .unwrap()
                .parse::<TxnPoolKeyType>()
                .unwrap();
            self.delete_txn_order(&timestamp);
            self.delete_txn_hash(each_hash);
        }
    }

    /// aim of this fxn is revert all changes happened because of block proposal which didn't accepted by the consensus.
    fn sync_order_pool(&mut self, txn_hash_vec: &Vec<Hash>) {
        for each_hash in txn_hash_vec.iter() {
            let txn: SignedTransaction = self.get(each_hash).unwrap().clone();
            let timestamp = txn
                .header
                .get(&String::from("timestamp"))
                .unwrap()
                .parse::<TxnPoolKeyType>()
                .unwrap();
            self.order_pool.insert(timestamp, txn);
        }
    }
}

lazy_static! {
    pub static ref TRANSACTION_POOL: Arc<std::sync::Mutex<TransactionPool>> =
        Arc::new(Mutex::new(TransactionPool::new()));
}

#[cfg(test)]
mod tests_transactions {

    #[test]
    pub fn main_transaction() {
        use super::*;
        use crate::transaction::Txn;
        use std::time::SystemTime;
        use utils::keypair::{CryptoKeypair, Keypair};

        let mut transaction_pool = TransactionPool::new();
        let kp = Keypair::generate();
        let one = SignedTransaction::generate(&kp);
        let two = SignedTransaction::generate(&kp);
        let time_instant = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_micros();
        transaction_pool.insert_op(&time_instant, &one);
        let time_instant = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_micros();
        transaction_pool.insert_op(&time_instant, &two);
    }
}
