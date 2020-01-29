use chrono::prelude::Utc;
use schema::transaction::{SignedTransaction, Txn};
use schema::transaction_pool::{TransactionPool, TxnPool};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use utils::keypair::{CryptoKeypair, Keypair, KeypairType};

pub fn add_txn_to_txn_pool(
    kp: &KeypairType,
    txn_pool_clone: Arc<std::sync::Mutex<schema::transaction_pool::TransactionPool>>,
) {
    loop {
        thread::sleep(Duration::from_millis(500));
        let mut txn_pool = txn_pool_clone.lock().unwrap();
        let mut one = SignedTransaction::generate(kp);
        let time_instant = Utc::now().timestamp_nanos();
        one.header
            .insert("timestamp".to_string(), time_instant.to_string());
        txn_pool.insert_op(&time_instant, &one);
    }
}
