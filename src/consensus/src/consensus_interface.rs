extern crate db_service;
extern crate p2plib;
extern crate schema;
extern crate utils;

use db_service::db_fork_ref::SchemaFork;
use db_service::db_layer::{fork_db, patch_db};
use exonum_merkledb::ObjectHash;
use futures::{channel::mpsc::*, executor::*, future, prelude::*, task::*};
use p2plib::messages::{
    ConsensusMessageTypes, MessageTypes, NodeMessageTypes, SignedLeaderElection,
};
use schema::transaction_pool::{TxnPool, TRANSACTION_POOL};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use utils::configreader::{Configuration};
use utils::keypair::{KeypairType, PublicKey, Verify};
use utils::serializer::serialize;

pub struct Consensus {
    keypair: KeypairType,
    public_keys: Vec<String>,
}

pub struct LeaderMap {
    map: HashMap<u64, String>,
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
                let (genesis_signed_block, txn_vec) =
                    schema.initialize_db(&self.keypair, &self.public_keys);
                for each in txn_vec.iter() {
                    let data = Some(MessageTypes::NodeMsg(
                        NodeMessageTypes::SignedTransactionEnum(each.clone()),
                    ));
                    sender.try_send(data);
                }
                let data = Some(MessageTypes::NodeMsg(NodeMessageTypes::SignedBlockEnum(
                    genesis_signed_block,
                )));
                sender.try_send(data);
            }
            patch_db(fork);
        }
    }

    fn validator(&self, sender: &mut Sender<Option<MessageTypes>>) {
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
            }
            patch_db(fork);
        }
    }

    pub fn consensus_msg_receiver(
        leader_map: Arc<Mutex<LeaderMap>>,
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
                                            let next_leader_obj: SignedLeaderElection = data;
                                            let ser_leader_election =
                                                serialize(&next_leader_obj.leader_payload);
                                            let mut leader_map_locked = leader_map.lock().unwrap();
                                            let current_leader = leader_map_locked.map.get(
                                                &(next_leader_obj.leader_payload.block_height - 1),
                                            );
                                            match current_leader {
                                                Some(pk) => {
                                                    if PublicKey::verify_from_encoded_pk(
                                                        pk,
                                                        &ser_leader_election,
                                                        &next_leader_obj.signature,
                                                    ) {
                                                        leader_map_locked.map.insert(
                                                            next_leader_obj
                                                                .leader_payload
                                                                .block_height,
                                                            next_leader_obj
                                                                .leader_payload
                                                                .public_key
                                                                .clone(),
                                                        );
                                                    }
                                                }
                                                None => println!("current leader is not verified"),
                                            }

                                            // update leader selection process here.
                                        }
                                        ConsensusMessageTypes::BlockVote(_data) => {
                                            // TODO: this enum this is not required
                                            // -----
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
        msg_receiver: Arc<Mutex<Receiver<Option<ConsensusMessageTypes>>>>,
    ) {
        let xyz: &crate::consensus_config::Configuration = &crate::consensus_config::GLOBAL_CONFIG;
        let consensus_obj = Consensus {
            keypair: config.node.keypair.clone(),
            public_keys: xyz.public_keys.clone(),
        };
        println!("{:?}", consensus_obj.public_keys);
        let map = LeaderMap {
            map: HashMap::new(),
        };
        let leader_map = Arc::new(Mutex::new(map));
        Consensus::consensus_msg_receiver(leader_map.clone(), msg_receiver);
        thread::sleep(Duration::from_millis(5000));
        consensus_obj.init_state(config.node.genesis_block, &config.db.dbpath, sender);
        consensus_obj.validator(sender);
    }
}
