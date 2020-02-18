extern crate db_service;
extern crate p2plib;
extern crate schema;
extern crate utils;

use db_service::db_fork_ref::SchemaFork;
use db_service::db_layer::{fork_db, patch_db, snapshot_db};
use db_service::db_snapshot_ref::SchemaSnap;
use exonum_merkledb::ObjectHash;
use futures::{channel::mpsc::*, executor::*, future, prelude::*, task::*};
use p2plib::messages::{
    ConsensusMessageTypes, LeaderElection, MessageTypes, NodeMessageTypes, SignedLeaderElection,
};
use schema::transaction_pool::{TxnPool, TRANSACTION_POOL};
use std::collections::{hash_map::DefaultHasher, BTreeMap};
use std::hash::{Hash, Hasher};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use utils::configreader::Configuration;
use utils::keypair::{CryptoKeypair, Keypair, KeypairType, PublicKey, Verify};
use utils::serializer::serialize;

pub struct Consensus {
    keypair: KeypairType,
    pk: String,
    round_number: u64,
    public_keys: Vec<String>,
}

pub struct LeaderMap {
    map: BTreeMap<u64, String>,
}

impl Consensus {
    fn init_state(
        &mut self,
        _db_path: &String,
        leader_map: Arc<Mutex<LeaderMap>>,
        sender: &mut Sender<Option<MessageTypes>>,
    ) {
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
        let leader_payload: LeaderElection = LeaderElection {
            block_height: self.round_number + 1,
            old_leader: self.pk.clone(),
            new_leader: self.pk.clone(),
        };
        self.round_number = self.round_number + 1;
        let signature = Keypair::sign(&self.keypair, &serialize(&leader_payload));
        let signed_new_leader: SignedLeaderElection = SignedLeaderElection {
            leader_payload,
            signature,
        };
        let data = Some(MessageTypes::ConsensusMsg(
            ConsensusMessageTypes::LeaderElect(signed_new_leader),
        ));
        let mut leader_map_locked = leader_map.lock().unwrap();
        leader_map_locked.map.insert(1, self.pk.clone());
        sender.try_send(data);
    }

    fn select_leader(&self) -> SignedLeaderElection {
        let mut iter_vec_pk: usize = 0;
        let mut current_lowest_hash: u64 = 0;
        if self.public_keys[iter_vec_pk].clone() != self.pk {
            let may_be_leader: LeaderElection = LeaderElection {
                block_height: self.round_number + 1,
                old_leader: self.pk.clone(),
                new_leader: String::from(self.public_keys[iter_vec_pk].clone()),
            };
            let mut hasher = DefaultHasher::new();
            may_be_leader.hash(&mut hasher);
            current_lowest_hash = hasher.finish();
        } else {
            iter_vec_pk = 1;
            let may_be_leader: LeaderElection = LeaderElection {
                block_height: self.round_number + 1,
                old_leader: self.pk.clone(),
                new_leader: String::from(self.public_keys[iter_vec_pk].clone()),
            };
            let mut hasher = DefaultHasher::new();
            may_be_leader.hash(&mut hasher);
            current_lowest_hash = hasher.finish();
        }
        for i in iter_vec_pk + 1..self.public_keys.len() {
            if self.pk != self.public_keys[i].clone() {
                let may_be_leader: LeaderElection = LeaderElection {
                    block_height: self.round_number + 1,
                    old_leader: self.pk.clone(),
                    new_leader: String::from(self.public_keys[i].clone()),
                };
                let mut hasher = DefaultHasher::new();
                may_be_leader.hash(&mut hasher);
                if current_lowest_hash > hasher.finish() {
                    iter_vec_pk = i;
                    current_lowest_hash = hasher.finish();
                }
            }
        }
        let leader_payload: LeaderElection = LeaderElection {
            block_height: self.round_number + 1,
            old_leader: self.pk.clone(),
            new_leader: String::from(self.public_keys[iter_vec_pk].clone()),
        };
        let signature = Keypair::sign(&self.keypair, &serialize(&leader_payload));
        SignedLeaderElection {
            leader_payload,
            signature,
        }
    }

    fn validator(&mut self, sender: &mut Sender<Option<MessageTypes>>) {
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
            let data = Some(MessageTypes::NodeMsg(NodeMessageTypes::SignedBlockEnum(
                signed_block,
            )));
            sender.try_send(data);
        }
        patch_db(fork);
        let signed_new_leader: SignedLeaderElection = self.select_leader();
        self.round_number = self.round_number + 1;
        let data = Some(MessageTypes::ConsensusMsg(
            ConsensusMessageTypes::LeaderElect(signed_new_leader),
        ));
        sender.try_send(data);
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
                                            let new_leader_obj: SignedLeaderElection = data;
                                            let ser_leader_election =
                                                serialize(&new_leader_obj.leader_payload);
                                            if PublicKey::verify_from_encoded_pk(
                                                &new_leader_obj.leader_payload.old_leader,
                                                &ser_leader_election,
                                                &new_leader_obj.signature,
                                            ) {
                                                let mut leader_map_locked =
                                                    leader_map.lock().unwrap();
                                                leader_map_locked.map.insert(
                                                    new_leader_obj.leader_payload.block_height,
                                                    new_leader_obj
                                                        .leader_payload
                                                        .new_leader
                                                        .clone(),
                                                );
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

    fn state_machine(
        &mut self,
        leader_map: Arc<Mutex<LeaderMap>>,
        sender: &mut Sender<Option<MessageTypes>>,
    ) {
        loop {
            /*
            check leader map so you will get least rn and respective leader
            now check in db the block count.
            accordingly decide either leader map need cleanup or i'm the leader or other's are leader.
            if leader_map_least_num < block_count  ---- cleanup
            else you are the leader or other will be leader
            */
            {
                let snapshot = snapshot_db();
                let db_snapshot = SchemaSnap::new(&snapshot);
                let current_block_chain_length = db_snapshot.get_blockchain_length();
                let mut leader_map_locked = leader_map.lock().unwrap();
                if let Some((_key, _value)) = leader_map_locked.map.iter().next() {
                    let key = _key.clone();
                    if key < current_block_chain_length {
                        // block already added so no need of previous leader
                        leader_map_locked.map.remove(&key);
                    } else if key > current_block_chain_length {
                        // future leader
                        // by pass this for now and wait
                    } else {
                        // current leader
                        let value = _value.clone();
                        if value != self.pk {
                            // someone else is the current leader.
                            leader_map_locked.map.remove(&key);
                        } else {
                            // i am the leader.
                            self.validator(sender);
                        }
                    }
                } else {
                }
                thread::sleep(Duration::from_millis(2000));
            }
        }
    }

    pub fn init_consensus(
        config: &Configuration,
        sender: &mut Sender<Option<MessageTypes>>,
        msg_receiver: Arc<Mutex<Receiver<Option<ConsensusMessageTypes>>>>,
    ) {
        let consensus_configuration: &crate::consensus_config::Configuration =
            &crate::consensus_config::GLOBAL_CONFIG;
        let mut consensus_obj = Consensus {
            keypair: config.node.keypair.clone(),
            pk: String::from(""),
            round_number: 0,
            public_keys: consensus_configuration.public_keys.clone(),
        };
        consensus_obj.pk = hex::encode(consensus_obj.keypair.public().encode());
        let leader_map_obj = LeaderMap {
            map: BTreeMap::new(),
        };
        let leader_map = Arc::new(Mutex::new(leader_map_obj));
        let leader_map_clone = leader_map.clone();
        Consensus::consensus_msg_receiver(leader_map_clone, msg_receiver);
        thread::sleep(Duration::from_millis(5000));
        if config.node.genesis_block {
            {
                let clone_leader_map = leader_map.clone();
                let mut leader_map_locked = clone_leader_map.lock().unwrap();
                leader_map_locked.map.insert(0, consensus_obj.pk.clone());
            }
            consensus_obj.init_state(&config.db.dbpath, leader_map.clone(), &mut sender.clone());
        }
        consensus_obj.state_machine(leader_map.clone(), &mut sender.clone());
    }
}

/*
how to control mining function on the basis of leader function
only genesis created will start mining defult other will create block on leader calling.
do this first
*/
