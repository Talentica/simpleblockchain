extern crate db_service;
extern crate message_handler;
extern crate schema;
extern crate utils;

use super::poa_config::initialize_config;
use super::poa_message_sender::ConsensusMessageSender;
use super::poa_messages::{
    ConsensusMessageTypes, ElectionPing, ElectionPong, LeaderElection, SignedLeaderElection,
};
use db_service::db_fork_ref::SchemaFork;
use db_service::db_layer::{fork_db, patch_db, snapshot_db};
use db_service::db_snapshot_ref::SchemaSnap;
use exonum_merkledb::{Fork, ObjectHash};
use futures::{channel::mpsc::*, executor::*, future, prelude::*, task::*};
use message_handler::message_sender::MessageSender;
use message_handler::messages::MessageTypes;
use schema::block::SignedBlock;
use schema::transaction_pool::{TxnPool, POOL};
use std::collections::{hash_map::DefaultHasher, BTreeMap};
use std::hash::{Hash, Hasher};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use std::time::SystemTime;
use utils::configreader::Configuration;
use utils::keypair::{CryptoKeypair, Keypair, KeypairType, PublicKey, Verify};
use utils::serializer::{deserialize, serialize};

pub struct Consensus {
    keypair: KeypairType,
    pk: String,
    round_number: u64,
    public_keys: Vec<String>,
    force_sealing: bool,
}

pub struct LeaderMap {
    map: BTreeMap<u64, String>,
}

pub struct MetaData {
    active_node: Vec<String>,
    public_keys: Vec<String>,
    kp: KeypairType,
    sender: Sender<Option<MessageTypes>>,
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
            let mut schema = SchemaFork::new(&fork);
            if schema.blockchain_length() == 0 {
                let custom_headers: Vec<u8> = Vec::new();
                let timestamp: u128 = SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap()
                    .as_micros();
                let genesis_signed_block = schema.initialize_db(custom_headers, timestamp);
                info!(
                    "genesis block created!, genesis block hash {:?}",
                    genesis_signed_block.get_hash()
                );
                MessageSender::send_block_msg(sender, genesis_signed_block);
            } else {
                self.round_number = schema.blockchain_length() - 1;
                info!(
                    "started from previous state {} {}",
                    schema.blockchain_length(),
                    schema.state_trie_merkle_hash()
                )
            }
        }
        patch_db(fork);
        let leader_payload: LeaderElection = LeaderElection {
            block_height: self.round_number + 1,
            old_leader: self.pk.clone(),
            new_leader: self.pk.clone(),
        };
        self.round_number = self.round_number + 1;
        let serialized_data: Vec<u8> = match serialize(&leader_payload) {
            Result::Ok(value) => value,
            Result::Err(_) => vec![0],
        };
        let signature = Keypair::sign(&self.keypair, &serialized_data);
        let signed_new_leader: SignedLeaderElection = SignedLeaderElection {
            leader_payload,
            signature,
        };
        ConsensusMessageSender::send_leader_election_msg(sender, signed_new_leader);
        let mut leader_map_locked = leader_map.lock().unwrap();
        leader_map_locked
            .map
            .insert(self.round_number, self.pk.clone());
    }

    fn select_leader(&self, meta_data: Arc<Mutex<MetaData>>) -> SignedLeaderElection {
        thread::sleep(Duration::from_millis(1000));
        let meta_data_locked = meta_data.lock().unwrap();
        let mut iter_vec_pk: usize = 0;
        #[allow(unused_assignments)]
        let mut current_lowest_hash: u64 = 0;
        if meta_data_locked.active_node.len() == 0 {
            warn!("no other node active");
            let leader_payload: LeaderElection = LeaderElection {
                block_height: self.round_number + 1,
                old_leader: self.pk.clone(),
                new_leader: self.pk.clone(),
            };
            let serialized_data: Vec<u8> = match serialize(&leader_payload) {
                Result::Ok(value) => value,
                Result::Err(_) => vec![0],
            };
            let signature = Keypair::sign(&self.keypair, &serialized_data);
            return SignedLeaderElection {
                leader_payload,
                signature,
            };
        }
        for i in iter_vec_pk..meta_data_locked.active_node.len() {
            if self.pk != meta_data_locked.active_node[i].clone() {
                let may_be_leader: LeaderElection = LeaderElection {
                    block_height: self.round_number + 1,
                    old_leader: self.pk.clone(),
                    new_leader: String::from(meta_data_locked.active_node[i].clone()),
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
            new_leader: String::from(meta_data_locked.active_node[iter_vec_pk].clone()),
        };
        let serialized_data: Vec<u8> = match serialize(&leader_payload) {
            Result::Ok(value) => value,
            Result::Err(_) => vec![0],
        };
        let signature = Keypair::sign(&self.keypair, &serialized_data);
        SignedLeaderElection {
            leader_payload,
            signature,
        }
    }

    fn propose_block(&self, fork: Fork) -> (Fork, SignedBlock) {
        let signed_block: SignedBlock;
        {
            let mut schema = SchemaFork::new(&fork);
            // let signed_block = schema.create_block(&self.keypair);
            let custom_headers: Vec<u8> = Vec::new();
            if !self.force_sealing {
                let (fork_instance, signed_block) =
                    schema.forge_new_block(&self.keypair, custom_headers);
                info!(
                    "new block created.. id {},hash {}",
                    signed_block.block.id,
                    signed_block.object_hash()
                );
                return (fork_instance, signed_block);
            }
            signed_block = schema.create_block(&self.keypair, custom_headers);
            info!(
                "new block created.. id {},hash {}",
                signed_block.block.id,
                signed_block.object_hash()
            );
        }
        return (fork, signed_block);
    }

    fn validator(
        &mut self,
        sender: &mut Sender<Option<MessageTypes>>,
        meta_data: Arc<Mutex<MetaData>>,
    ) -> bool {
        // no polling machenism of txn_pool and create block need to implement or modified here
        // if one want to change the create_block and txn priority then change/ implment that part in
        // schema operations and p2p module
        let mut fork = fork_db();
        {
            let (fork_instance, signed_block) = self.propose_block(fork);
            fork = fork_instance;
            POOL.sync_pool(&signed_block.block.txn_pool);
            self.round_number = signed_block.block.id;
            MessageSender::send_block_msg(sender, signed_block);

            let mut meta_data_locked = meta_data.lock().unwrap();
            meta_data_locked.active_node.clear();
            let msg: ElectionPing =
                ElectionPing::create(&meta_data_locked.kp, self.round_number + 1);
            ConsensusMessageSender::send_election_ping_msg(sender, msg);
            info!("pinging for block number {}", self.round_number + 1);
            thread::sleep(Duration::from_micros(1000));
        }
        patch_db(fork);
        let signed_new_leader: SignedLeaderElection = self.select_leader(meta_data);
        self.round_number = self.round_number + 1;
        let flag: bool = signed_new_leader.leader_payload.new_leader.clone()
            == signed_new_leader.leader_payload.old_leader.clone();
        ConsensusMessageSender::send_leader_election_msg(sender, signed_new_leader);

        return flag;
    }

    pub fn consensus_msg_receiver(
        leader_map: Arc<Mutex<LeaderMap>>,
        meta_data: Arc<Mutex<MetaData>>,
        rx: Arc<Mutex<Receiver<Option<Vec<u8>>>>>,
    ) {
        thread::spawn(move || {
            block_on(future::poll_fn(move |cx: &mut Context| {
                loop {
                    match rx.lock().unwrap().poll_next_unpin(cx) {
                        Poll::Ready(Some(msg)) => {
                            match msg {
                                None => info!("Empty msg received !"),
                                Some(msgtype) => {
                                    if let Ok(msgtype) =
                                        deserialize::<ConsensusMessageTypes>(msgtype.as_slice())
                                    {
                                        match msgtype {
                                            ConsensusMessageTypes::LeaderElect(data) => {
                                                let new_leader_obj: SignedLeaderElection = data;
                                                let ser_leader_election: Vec<u8> =
                                                    match serialize(&new_leader_obj.leader_payload)
                                                    {
                                                        Result::Ok(value) => value,
                                                        Result::Err(_) => vec![0],
                                                    };
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
                                                    info!(
                                                        "New Leader for block height {} -> {}",
                                                        new_leader_obj.leader_payload.block_height,
                                                        new_leader_obj.leader_payload.new_leader,
                                                    );
                                                }
                                                // update leader selection process here.
                                            }
                                            ConsensusMessageTypes::ConsensusPing(data) => {
                                                let election_ping: ElectionPing = data;
                                                let mut meta_data_locked =
                                                    meta_data.lock().unwrap();
                                                if meta_data_locked
                                                    .public_keys
                                                    .contains(&election_ping.payload.public_key)
                                                {
                                                    if election_ping.verify() {
                                                        let election_pong: ElectionPong =
                                                            ElectionPong::create(
                                                                &meta_data_locked.kp,
                                                                &election_ping,
                                                            );
                                                        ConsensusMessageSender::send_election_pong_msg(
                                                            &mut meta_data_locked.sender,
                                                            election_pong,
                                                        );
                                                        debug!(
                                                            "Ping message from  {} for height {} -> ",
                                                            election_ping.payload.public_key,
                                                            election_ping.payload.height,
                                                        );
                                                    } else {
                                                        warn!(
                                                            "Election Ping data tempered {}",
                                                            election_ping.payload.height
                                                        );
                                                    }
                                                } else {
                                                    debug!(
                                                        "public_keys {:?} key {:?}",
                                                        meta_data_locked.public_keys,
                                                        election_ping.payload.public_key
                                                    );
                                                    warn!("Election Ping data from malicious node");
                                                }
                                            }
                                            ConsensusMessageTypes::ConsensusPong(data) => {
                                                let election_pong: ElectionPong = data;
                                                let mut meta_data_locked =
                                                    meta_data.lock().unwrap();
                                                if hex::encode(
                                                    meta_data_locked.kp.public().encode(),
                                                ) == election_pong.payload.current_leader
                                                {
                                                    if meta_data_locked.public_keys.contains(
                                                        &election_pong.payload.may_be_leader,
                                                    ) {
                                                        if election_pong.verify() {
                                                            meta_data_locked.active_node.push(
                                                                election_pong
                                                                    .payload
                                                                    .may_be_leader
                                                                    .clone(),
                                                            );
                                                            debug!(
                                                            "Pong message received from  {} for height {} -> ",
                                                            election_pong.payload.may_be_leader,
                                                            election_pong.payload.height,
                                                        );
                                                        } else {
                                                            warn!(
                                                                "Election Ping data tempered {}",
                                                                election_pong.payload.may_be_leader
                                                            );
                                                        }
                                                    } else {
                                                        warn!(
                                                            "Election Pong data tempered {}",
                                                            election_pong.payload.may_be_leader,
                                                        );
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        Poll::Ready(None) => {
                            info!("channel closed !");
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
        meta_data: Arc<Mutex<MetaData>>,
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
                        let fork = fork_db();
                        {
                            let mut schema = SchemaFork::new(&fork);
                            schema.sync_state();
                        }
                        patch_db(fork);
                    } else {
                        // current leader
                        let value = _value.clone();
                        if value != self.pk {
                            // someone else is the current leader.
                            leader_map_locked.map.remove(&key);
                        } else {
                            // i am the leader.
                            info!("I AM THE LEADER NOW!!!");
                            if self.validator(sender, meta_data.clone()) {
                                leader_map_locked
                                    .map
                                    .insert(self.round_number, self.pk.clone());
                            }
                        }
                    }
                } else {
                }
            }
            thread::sleep(Duration::from_millis(2000));
        }
    }

    pub fn init_poa_consensus(
        config: &Configuration,
        consensus_file_path: &str,
        sender: &mut Sender<Option<MessageTypes>>,
        msg_receiver: Arc<Mutex<Receiver<Option<Vec<u8>>>>>,
    ) {
        initialize_config(consensus_file_path);
        let consensus_configuration: &crate::poa_config::Configuration =
            &crate::poa_config::POA_CONFIG;
        let mut consensus_obj = Consensus {
            keypair: config.node.keypair.clone(),
            pk: String::from(""),
            round_number: 0,
            public_keys: consensus_configuration.public_keys.clone(),
            force_sealing: consensus_configuration.force_sealing,
        };
        consensus_obj.pk = hex::encode(consensus_obj.keypair.public().encode());
        let leader_map_obj = LeaderMap {
            map: BTreeMap::new(),
        };
        let leader_map = Arc::new(Mutex::new(leader_map_obj));

        let consensus_meta_data = MetaData {
            active_node: vec![],
            public_keys: consensus_obj.public_keys.clone(),
            kp: config.node.keypair.clone(),
            sender: sender.clone(),
        };
        let meta_data = Arc::new(Mutex::new(consensus_meta_data));
        Consensus::consensus_msg_receiver(leader_map.clone(), meta_data.clone(), msg_receiver);
        thread::sleep(Duration::from_millis(5000));
        if config.node.genesis_block {
            {
                let clone_leader_map = leader_map.clone();
                let mut leader_map_locked = clone_leader_map.lock().unwrap();
                leader_map_locked.map.insert(0, consensus_obj.pk.clone());
            }
            consensus_obj.init_state(&config.db.dbpath, leader_map.clone(), &mut sender.clone());
        } else {
            let fork = fork_db();
            {
                let mut schema = SchemaFork::new(&fork);
                schema.sync_state();
            }
            patch_db(fork);
        }
        consensus_obj.state_machine(leader_map.clone(), meta_data.clone(), &mut sender.clone());
    }
}
