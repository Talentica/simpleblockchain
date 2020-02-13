extern crate db;
extern crate db_service;
extern crate p2plib;
extern crate schema;
extern crate utils;

use db::db_layer::{fork_db, patch_db};
use db_service::db_fork_ref::SchemaFork;
use exonum_merkledb::ObjectHash;
use futures::{channel::mpsc::*, executor::*, future, prelude::*, task::*};
use p2plib::messages::{ConsensusMessageTypes, MessageTypes};
use schema::block::SignedBlock;
use schema::transaction_pool::{TxnPool, TRANSACTION_POOL};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use utils::configreader::{Configuration, NODETYPE};
use utils::keypair::KeypairType;

pub struct Consensus {
    keypair: KeypairType,
}

pub struct Blocks {
    pending_blocks: std::collections::VecDeque<SignedBlock>,
}

impl Consensus {
    fn init_state(
        &self,
        genesis_block: bool,
        _db_path: &String,
        sender: &mut Sender<Option<MessageTypes>>,
    ) {
        if genesis_block {
            let fork = fork_db();
            {
                let schema = SchemaFork::new(&fork);
                let genesis_signed_block: SignedBlock = schema.initialize_db(&self.keypair);
                let data = Some(MessageTypes::ConsensusMsg(
                    ConsensusMessageTypes::BlockVote(genesis_signed_block),
                ));
                sender.try_send(data);
            }
            patch_db(fork);
        }
    }

    fn validator(&mut self, sender: &mut Sender<Option<MessageTypes>>) {
        loop {
            thread::sleep(Duration::from_millis(10000));
            // no polling machenism of txn_pool and create block need to implement or modified here
            // if one want to change the create_block and txn priority then change/ implment that part in
            // schema operations and p2p module
            let arc_txn_pool = TRANSACTION_POOL.clone();
            let fork = fork_db();
            {
                let schema = SchemaFork::new(&fork);
                let mut txn_pool = arc_txn_pool.lock().unwrap();
                let signed_block = schema.create_block(&self.keypair, &mut txn_pool);
                println!("new block created.. hash {}", signed_block.object_hash());
                txn_pool.sync_pool(&signed_block.block.txn_pool);
                let data = Some(MessageTypes::ConsensusMsg(
                    ConsensusMessageTypes::BlockVote(signed_block),
                ));
                sender.try_send(data);
            }
            patch_db(fork);
        }
    }

    fn full_node(&mut self, pending_blocks: Arc<Mutex<Blocks>>) {
        loop {
            thread::sleep(Duration::from_millis(4000));
            // no polling machenism of txn_pool and create block need to implement or modified here
            // if one want to change the create_block and txn priority then change/ implment that part in
            // schema operations and p2p module
            let mut block_queue = pending_blocks.lock().unwrap();
            if block_queue.pending_blocks.len() > 0 {
                let arc_txn_pool = TRANSACTION_POOL.clone();
                let fork = fork_db();
                let mut flag = true;
                {
                    let schema = SchemaFork::new(&fork);
                    let mut txn_pool = arc_txn_pool.lock().unwrap();
                    let block: &SignedBlock = block_queue.pending_blocks.get_mut(0).unwrap();
                    if schema.update_block(block, &mut txn_pool) {
                        txn_pool.sync_pool(&block.block.txn_pool);
                    } else {
                        println!("block couldn't verified");
                        flag = false;
                    }
                }
                if flag {
                    patch_db(fork);
                    block_queue.pending_blocks.pop_front();
                    println!("block updated in db");
                }
            }
        }
    }

    pub fn start_receiver(
        pending_blocks: Arc<Mutex<Blocks>>,
        rx: Arc<Mutex<Receiver<Option<ConsensusMessageTypes>>>>,
    ) {
        thread::spawn(move || {
            block_on(future::poll_fn(move |cx: &mut Context| {
                loop {
                    match rx.lock().unwrap().poll_next_unpin(cx) {
                        Poll::Ready(Some(msg)) => {
                            match msg {
                                None => println!("Empty msg received !"),
                                Some(msgtype) => {
                                    match msgtype {
                                        ConsensusMessageTypes::LeaderElect(data) => {
                                            println!(
                                            "Leader Elect msg in ConsensusMsgProcessor with data {:?}",
                                            data
                                        );
                                            //TODO
                                            //Write msg processing code
                                        }
                                        ConsensusMessageTypes::BlockVote(data) => {
                                            let signed_block: SignedBlock = data;
                                            println!("Signed Block msg in ConsensusMsgProcessor with Hash {:?}", signed_block.object_hash());
                                            let mut block_queue = pending_blocks.lock().unwrap();
                                            block_queue.pending_blocks.push_back(signed_block);
                                            println!(
                                                "block queue length {}",
                                                block_queue.pending_blocks.len()
                                            );
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
        });
    }

    pub fn init_consensus(
        config: &Configuration,
        sender: &mut Sender<Option<MessageTypes>>,
        msg_receiver: Option<Arc<Mutex<Receiver<Option<ConsensusMessageTypes>>>>>,
    ) {
        let mut consensus_obj = Consensus {
            keypair: config.node.keypair.clone(),
        };
        let mut pending_blocks_obj = Blocks {
            pending_blocks: std::collections::VecDeque::new(),
        };
        let pending_blocks = Arc::new(Mutex::new(pending_blocks_obj));
        match msg_receiver {
            Some(receiver) => Consensus::start_receiver(pending_blocks.clone(), receiver),
            None => println!("Validator Node"),
        }
        thread::sleep(Duration::from_millis(5000));
        consensus_obj.init_state(config.node.genesis_block, &config.db.dbpath, sender);

        match config.node.node_type {
            NODETYPE::Validator => consensus_obj.validator(sender),
            NODETYPE::FullNode => consensus_obj.full_node(pending_blocks.clone()),
        }
    }
}
