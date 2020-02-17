use crate::messages::{MessageTypes, NodeMessageTypes};
use chrono::prelude::Utc;
use futures::channel::mpsc::*;
use schema::transaction::{SignedTransaction, Txn};
use schema::transaction_pool::{TxnPool, TRANSACTION_POOL};
use std::thread;
use std::time::Duration;
use utils::keypair::KeypairType;

pub fn add_txn_to_txn_pool(kp: &KeypairType, txn_sender: &mut Sender<Option<MessageTypes>>) {
    thread::sleep(Duration::from_millis(5000));
    loop {
        thread::sleep(Duration::from_millis(3000));
        let arc_tx_pool = TRANSACTION_POOL.clone();
        let mut txn_pool = arc_tx_pool.lock().unwrap();
        let mut one = SignedTransaction::generate(kp);
        let time_instant = Utc::now().timestamp_nanos();
        one.header
            .insert("timestamp".to_string(), time_instant.to_string());
        txn_pool.insert_op(&time_instant, &one);
        let data = Some(MessageTypes::NodeMsg(
            NodeMessageTypes::SignedTransactionEnum(one),
        ));
        txn_sender.try_send(data);
    }
}
