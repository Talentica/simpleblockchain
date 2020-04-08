extern crate db_service;
use futures::{channel::mpsc::*, executor::*, future, prelude::*, task::*};
use p2plib::messages::*;

use db_service::db_fork_ref::SchemaFork;
use db_service::db_layer::{fork_db, patch_db};
use schema::block::SignedBlock;
use schema::transaction::{ObjectHash, SignedTransaction};
use schema::transaction_pool::{TxnPool, TxnPoolKeyType, POOL};
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;

#[derive(Debug)]
pub struct NodeMsgProcessor {
    pub _rx: Arc<Mutex<Receiver<Option<NodeMessageTypes>>>>,
}

pub struct Blocks {
    pending_blocks: std::collections::VecDeque<SignedBlock>,
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
        let pending_blocks_obj = Blocks {
            pending_blocks: std::collections::VecDeque::new(),
        };
        let arc_pending_blocks = Arc::new(Mutex::new(pending_blocks_obj));
        NodeMsgProcessor::pending_block_processing_thread(arc_pending_blocks.clone());
        let pending_blocks = arc_pending_blocks.clone();
        block_on(future::poll_fn(move |cx: &mut Context| {
            loop {
                match self._rx.lock().unwrap().poll_next_unpin(cx) {
                    Poll::Ready(Some(msg)) => {
                        // debug!("msg received {:?}", msg);
                        match msg {
                            None => debug!("Empty msg received !"),
                            Some(msgtype) => {
                                match msgtype {
                                    NodeMessageTypes::SignedBlockEnum(data) => {
                                        debug!(
                                            "Signed Block msg in NodeMsgProcessor with data {:?}",
                                            data.object_hash()
                                        );
                                        let signed_block: SignedBlock = data;
                                        let mut block_queue = pending_blocks.lock().unwrap();
                                        block_queue.pending_blocks.push_back(signed_block);
                                        debug!(
                                            "block queue length {}",
                                            block_queue.pending_blocks.len()
                                        );
                                    }
                                    NodeMessageTypes::SignedTransactionEnum(data) => {
                                        let txn: SignedTransaction = data;
                                        debug!("Signed Transaction msg in NodeMsgProcessor with Hash {:?}", txn.object_hash());
                                        let timestamp = txn
                                            .header
                                            .get(&String::from("timestamp"))
                                            .unwrap()
                                            .parse::<TxnPoolKeyType>()
                                            .unwrap();
                                        POOL.insert_op(&timestamp, &txn);
                                    }
                                }
                            }
                        }
                    }
                    Poll::Ready(None) => {
                        debug!("channel closed !");
                        return Poll::Ready(1);
                    }
                    Poll::Pending => break,
                }
            }
            Poll::Pending
        }));
    }

    fn pending_block_processing_thread(pending_blocks: Arc<Mutex<Blocks>>) {
        thread::spawn(move || {
            loop {
                thread::sleep_ms(2000);
                // no polling machenism of txn_pool and create block need to implement or modified here
                // if one want to change the create_block and txn priority then change/ implment that part in
                // schema operations and p2p module
                let mut block_queue = pending_blocks.lock().unwrap();
                if block_queue.pending_blocks.len() > 0 {
                    let fork = fork_db();
                    let mut flag = false;
                    {
                        let mut schema = SchemaFork::new(&fork);
                        let block: &SignedBlock = block_queue.pending_blocks.get_mut(0).unwrap();
                        if schema.update_block(block) {
                            POOL.sync_pool(&block.block.txn_pool);
                            debug!(
                                "block height {}, block hash {}",
                                block.block.id,
                                block.object_hash()
                            );
                            flag = true;
                        } else {
                            debug!("block couldn't verified");
                            if schema.blockchain_length() > block.block.id {
                                block_queue.pending_blocks.pop_front();
                            } else {
                                flag = true;
                                schema.sync_state();
                            }
                        }
                    }
                    if flag {
                        patch_db(fork);
                        block_queue.pending_blocks.pop_front();
                        debug!("block updated in db");
                    }
                }
            }
        });
    }
}
