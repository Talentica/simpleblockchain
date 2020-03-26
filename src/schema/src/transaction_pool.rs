extern crate utils;

use crate::transaction::{SignedTransaction, State};
use exonum_crypto::Hash;
use exonum_merkledb::{
    access::{Access, RawAccessMut},
    ObjectHash, ProofMapIndex,
};
use generic_traits::traits::{PoolTrait, StateTraits, TransactionTrait};
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};
pub type TxnPoolKeyType = u128;
pub type TxnPoolValueType = SignedTransaction;

trait TransactionPoolTraits {
    fn new() -> Self;
    fn delete_txn_hash(&mut self, key: &Hash);
    fn delete_txn_order(&mut self, key: &TxnPoolKeyType);
    fn pop_front(&mut self) -> TxnPoolValueType;
    fn insert_op(&mut self, key: &TxnPoolKeyType, value: &TxnPoolValueType);
    fn length_order_pool(&self) -> usize;
    fn length_hash_pool(&self) -> usize;
    fn get(&self, key: &Hash) -> Option<TxnPoolValueType>;
    fn sync_pool(&mut self, txn_hash_vec: &Vec<Hash>);
    fn sync_order_pool(&mut self, txn_hash_vec: &Vec<Hash>);
}

pub trait TxnPool {
    fn new() -> Self;
    fn delete_txn_hash(&self, key: &Hash);
    fn delete_txn_order(&self, key: &TxnPoolKeyType);
    fn pop_front(&self) -> TxnPoolValueType;
    fn insert_op(&self, key: &TxnPoolKeyType, value: &TxnPoolValueType);
    fn length_order_pool(&self) -> usize;
    fn length_hash_pool(&self) -> usize;
    fn get(&self, key: &Hash) -> Option<TxnPoolValueType>;
    fn sync_pool(&self, txn_hash_vec: &Vec<Hash>);
    fn sync_order_pool(&self, txn_hash_vec: &Vec<Hash>);
}
/**
 * BTreeMap is used here for in-order push-pop values and at the same time, search operation also supported.
*/
/// TransactionPool object to maintain in-coming txn and txn-order.
#[derive(Debug, Clone)]
pub struct TransactionPool {
    hash_pool: BTreeMap<Hash, TxnPoolValueType>,
    order_pool: BTreeMap<TxnPoolKeyType, TxnPoolValueType>,
}

pub struct Pool {
    pub pool: Arc<std::sync::Mutex<TransactionPool>>,
}

impl TransactionPoolTraits for TransactionPool {
    /// this function will create a new instance of transcation pool object
    fn new() -> TransactionPool {
        TransactionPool {
            hash_pool: BTreeMap::new(),
            order_pool: BTreeMap::new(),
        }
    }

    /// this function will delete txn using hash if present, from hash_pool
    fn delete_txn_hash(&mut self, key: &Hash) {
        if self.hash_pool.contains_key(key) {
            self.hash_pool.remove(key);
        }
    }

    /// this function will delete txn using order_value if present, from order_pool
    fn delete_txn_order(&mut self, key: &TxnPoolKeyType) {
        if self.order_pool.contains_key(key) {
            self.order_pool.remove(key);
        }
    }

    /// this function will pop value in fifo order from order_pool
    fn pop_front(&mut self) -> TxnPoolValueType {
        let (first_key, first_value) = self.order_pool.iter().next().unwrap();
        let value = first_value.clone();
        let key = first_key.clone();
        self.order_pool.remove(&key);
        value
    }

    /// this function will push value in both (hash & order) pool
    fn insert_op(&mut self, key: &TxnPoolKeyType, value: &TxnPoolValueType) {
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
    fn get(&self, key: &Hash) -> Option<TxnPoolValueType> {
        if self.hash_pool.contains_key(key) {
            return Some(self.hash_pool.get(&key).unwrap().clone());
        } else {
            return Option::None;
        }
    }

    /// sync both (hash & order ) pool when block committed is created by the other node
    fn sync_pool(&mut self, txn_hash_vec: &Vec<Hash>) {
        for each_hash in txn_hash_vec.iter() {
            let txn: SignedTransaction = self.get(each_hash).unwrap();
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

impl TxnPool for Pool {
    /// this function will create a new instance of transcation pool object
    fn new() -> Pool {
        Pool {
            pool: Arc::new(Mutex::new(TransactionPool::new())),
        }
    }

    /// this function will delete txn using hash if present, from hash_pool
    fn delete_txn_hash(&self, key: &Hash) {
        let mut txn_pool = self.pool.lock().unwrap();
        txn_pool.delete_txn_hash(key);
    }

    /// this function will delete txn using order_value if present, from order_pool
    fn delete_txn_order(&self, key: &TxnPoolKeyType) {
        let mut txn_pool = self.pool.lock().unwrap();
        txn_pool.delete_txn_order(key);
    }

    /// this function will pop value in fifo order from order_pool
    fn pop_front(&self) -> TxnPoolValueType {
        let mut txn_pool = self.pool.lock().unwrap();
        txn_pool.pop_front()
    }

    /// this function will push value in both (hash & order) pool
    fn insert_op(&self, key: &TxnPoolKeyType, value: &TxnPoolValueType) {
        let mut txn_pool = self.pool.lock().unwrap();
        txn_pool.insert_op(key, value);
    }

    /// length of order_pool
    fn length_order_pool(&self) -> usize {
        let txn_pool = self.pool.lock().unwrap();
        txn_pool.length_order_pool()
    }

    /// length of hash_pool
    fn length_hash_pool(&self) -> usize {
        let txn_pool = self.pool.lock().unwrap();
        txn_pool.length_hash_pool()
    }

    /// get transaction usinng hash from hash_pool
    fn get(&self, key: &Hash) -> Option<TxnPoolValueType> {
        let txn_pool = self.pool.lock().unwrap();
        txn_pool.get(key)
    }

    /// sync both (hash & order ) pool when block committed is created by the other node
    fn sync_pool(&self, txn_hash_vec: &Vec<Hash>) {
        let mut txn_pool = self.pool.lock().unwrap();
        txn_pool.sync_pool(txn_hash_vec);
    }

    /// aim of this fxn is revert all changes happened because of block proposal which didn't accepted by the consensus.
    fn sync_order_pool(&self, txn_hash_vec: &Vec<Hash>) {
        let mut txn_pool = self.pool.lock().unwrap();
        txn_pool.sync_order_pool(txn_hash_vec);
    }
}

impl<T: Access> PoolTrait<T, State, SignedTransaction> for TransactionPool
where
    T::Base: RawAccessMut,
{
    fn execute_transactions(
        &self,
        state_trie: &mut ProofMapIndex<T::Base, String, State>,
        txn_trie: &mut ProofMapIndex<T::Base, Hash, SignedTransaction>,
    ) -> Vec<Hash> {
        let mut temp_vec: Vec<Hash> = Vec::with_capacity(15);
        // compute until order_pool exhusted or transaction limit crossed
        // let txn_pool = self.pool.lock().unwrap();
        for (_key, value) in self.order_pool.iter() {
            if temp_vec.len() < 15 {
                let sign_txn = value as &dyn StateTraits<T, State, SignedTransaction>;
                if sign_txn.execute(state_trie, txn_trie) {
                    temp_vec.push(value.get_hash());
                }
            } else {
                break;
            }
        }
        temp_vec
    }

    fn update_transactions(
        &self,
        state_trie: &mut ProofMapIndex<T::Base, String, State>,
        txn_trie: &mut ProofMapIndex<T::Base, Hash, SignedTransaction>,
        hash_vec: &Vec<Hash>,
    ) -> bool {
        // compute until order_pool exhusted or transaction limit crossed
        // let txn_pool = self.pool.lock().unwrap();
        for each in hash_vec.iter() {
            let signed_txn = self.get(each);
            if let Some(txn) = signed_txn {
                let sign_txn = &txn as &dyn StateTraits<T, State, SignedTransaction>;
                if !sign_txn.execute(state_trie, txn_trie) {
                    eprintln!(
                        "transaction execution error (either signature or business logic error)"
                    );
                    return false;
                }
            } else {
                eprintln!("transaction couldn't find for block execution");
                return false;
            }
        }
        true
    }
}

lazy_static! {
    pub static ref POOL: Pool = Pool::new();
}

#[cfg(test)]
mod tests_transactions {

    #[test]
    pub fn main_transaction() {
        use super::*;
        use generic_traits::traits::TransactionTrait;
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
