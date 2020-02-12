use futures::{channel::mpsc::*, executor::*, future, prelude::*, task::*};
use p2plib::messages::*;

use std::sync::Arc;
use std::sync::Mutex;
use schema::transaction_pool::{TransactionPool, TxnPool, TRANSACTION_POOL};
use schema::transaction::{SignedTransaction, ObjectHash};
#[derive(Debug)]
pub struct NodeMsgProcessor {
    // pub _tx: Sender<Option<NodeMessageTypes>>,
    pub _rx: Arc<Mutex<Receiver<Option<NodeMessageTypes>>>>,
}

impl NodeMsgProcessor {
    pub fn new(rx: Arc<Mutex<Receiver<Option<NodeMessageTypes>>>>) -> Self {
        // let (mut tx, mut rx) = channel::<Option<NodeMessageTypes>>(1024);
        // NodeMsgProcessor { _tx: tx, _rx: rx }
        NodeMsgProcessor { _rx: rx }
    }
    pub fn start(&mut self) {
        //, rx: &'static mut Receiver<Option<NodeMessageTypes>>) {
        // let thread_handle = thread::spawn(move || {
        block_on(future::poll_fn(move |cx: &mut Context| {
            loop {
                match self._rx.lock().unwrap().poll_next_unpin(cx) {
                    Poll::Ready(Some(msg)) => {
                        println!("msg received {:?}", msg);
                        match msg {
                            None => println!("Empty msg received !"),
                            Some(msgtype) => {
                                match msgtype {
                                    NodeMessageTypes::SignedBlockEnum(data) => {
                                        println!(
                                            "Signed Block msg in NodeMsgProcessor with data {:?}",
                                            data
                                        );
                                        //TODO
                                        //Write msg processing code
                                    }
                                    NodeMessageTypes::SignedTransactionEnum(data) => {
                                        let arc_tx_pool = TRANSACTION_POOL.clone();
                                        let mut txn_pool = arc_tx_pool.lock().unwrap();
                                        let txn: SignedTransaction = data;
                                        println!("Signed Transaction msg in NodeMsgProcessor with Hash {:?}", txn.object_hash());
                                        let timestamp = txn
                                            .header
                                            .get(&String::from("timestamp"))
                                            .unwrap()
                                            .parse::<i64>()
                                            .unwrap();
                                        txn_pool.insert_op(&timestamp, &txn);
                                    }
                                }
                            }
                        }
                    }
                    Poll::Ready(None) => {
                        println!("channel closed !");
                        return Poll::Ready(1);
                    }
                    Poll::Pending => break,
                }
            }
            Poll::Pending
        }));
    }
}
