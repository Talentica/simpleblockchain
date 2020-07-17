extern crate db_service;
extern crate message_handler;
extern crate schema;
extern crate utils;

use super::aura_message_sender::AuraMessageSender;
use super::aura_messages::{AuraMessageTypes, AuthorBlock, BlockAcceptance, RoundOwner};
use super::config::initialize_config;
use db_service::db_fork_ref::SchemaFork;
use db_service::db_layer::{fork_db, patch_db, snapshot_db};
use db_service::db_snapshot_ref::SchemaSnap;
use exonum_merkledb::{ObjectHash, Snapshot};
use futures::{channel::mpsc::*, executor::*, future, prelude::*, task::*};

use message_handler::messages::MessageTypes;
use schema::block::SignedBlock;
use schema::transaction_pool::{TxnPool, POOL};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use std::time::SystemTime;
use utils::configreader::Configuration;
use utils::keypair::KeypairType;
use utils::serializer::{deserialize, serialize, Deserialize, Serialize};

pub struct Aura {
    // peer identity
    keypair: KeypairType,
    // peer public key
    pk: String,
    // validator's details
    validator_mapping: HashMap<String, u64>,
    // 10th part of step_time (in millis)
    leader_epoch: u64,
    // empty block acceptance
    force_sealing: bool,
    // last start time of consensus (in seconds)
    start_time: u64,
}

/// WaitingBLocksQueue will store waiting block queue and
/// and ongoing author block details
/// WaitingBLocksQueue will help to create new block and verify new upcoming blocks.
/// WaitingBLocksQueue will be pushed to permanent state after "b" blocks get majority.
pub struct WaitingBLocksQueue {
    // waiting blocks queue
    pub queue: Vec<SignedBlock>,
    // ongoing block proposal acceptance
    pub last_block_acceptance: HashSet<String>,
    // ongoing block proposal hash
    pub last_block_hash: String,
}

impl WaitingBLocksQueue {
    pub fn new() -> WaitingBLocksQueue {
        WaitingBLocksQueue {
            queue: Vec::new(),
            last_block_acceptance: HashSet::new(),
            last_block_hash: String::from("temp_hash"),
        }
    }
}

/// AURA consensus key-details
pub struct MetaData {
    // validator's pool size
    validator_pool_size: u64,
    // validator's public_key mapping with auther_ordering number
    validator_mapping: HashMap<String, u64>,
    // peer keypair
    kp: KeypairType,
    // data sender in P2P system
    sender: Sender<Option<MessageTypes>>,
    // last start time of consensus (in seconds)
    start_time: u64,
    // round number at the time of last restart
    round_number: u64,
    // step time of round (in seconds)
    step_time: u64,
    // waiting blocks queue size
    block_queue_size: usize,
    // peer public key
    public_key: String,
}

// AURA consensus custom headers for the signed block
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CustomHeaders {
    timestamp: u64,
    round_number: u64,
}

impl Aura {
    // init_state will create genesis block if predefined storage is empty
    // or if storage is not empty it will start from previous state
    // read genesis block details from config file (future work)
    fn init_state(&mut self, _db_path: &String, _sender: &mut Sender<Option<MessageTypes>>) {
        let fork = fork_db();
        {
            let mut schema = SchemaFork::new(&fork);
            if schema.blockchain_length() == 0 {
                let custom_headers: CustomHeaders = CustomHeaders {
                    timestamp: self.start_time,
                    round_number: 0,
                };
                let custom_headers: Vec<u8> = match serialize(&custom_headers) {
                    Ok(value) => value,
                    Err(_) => Vec::new(),
                };
                let genesis_signed_block =
                    schema.initialize_db(custom_headers, self.start_time as u128);
                info!(
                    "genesis block created with hash {:?}",
                    genesis_signed_block.get_hash()
                );
            } else {
                info!(
                    "started from previous state {} {}",
                    schema.blockchain_length(),
                    schema.state_trie_merkle_hash()
                )
            }
        }
        patch_db(fork);
    }

    // fn will compute what is the round number at present time
    fn calculate_round_number(meta_data: &MetaData) -> u64 {
        let current_epoch: u64 = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs()
            - meta_data.start_time;
        let round_count: u64 = (current_epoch / meta_data.step_time) + meta_data.round_number;
        round_count
    }

    // fn will compute what is the round leader at present time
    fn primary_leader(meta_data: &MetaData) -> String {
        let round_count = Aura::calculate_round_number(meta_data);
        let leader_id: u64 = round_count
            - (round_count / meta_data.validator_pool_size) * meta_data.validator_pool_size;
        for (pk, id) in meta_data.validator_mapping.iter() {
            if id.clone() == leader_id {
                return pk.clone();
            }
        }
        panic!("Should match current identity with anyone validator identity");
    }

    // fn will process incoming AuthorBlockEnum data
    fn handle_author_block_enum(
        author_block: AuthorBlock,
        waiting_blocks_queue: &mut WaitingBLocksQueue,
        meta_data_obj: &mut MetaData,
    ) {
        let current_leader: String = Aura::primary_leader(&meta_data_obj);
        // only rightful auther should propose block
        if current_leader != author_block.block.block.peer_id.clone() {
            info!("malicious author proposing block!");
            return;
        }

        // validate block signature
        if !author_block.verify() {
            warn!(
                "malicious block proposed by author {:?}!",
                author_block.block.block.peer_id
            );
            return;
        }
        // validate block height & previous block hash
        match waiting_blocks_queue.queue.last() {
            Some(last_waiting_block) => {
                let last_waiting_block: &SignedBlock = last_waiting_block;
                if last_waiting_block.block.id + 1 != author_block.block.block.id {
                    warn!(
                        "malicious block proposed by author {:?}!",
                        author_block.block.block.peer_id
                    );
                    warn!("malicious block proposed, invalid height compare to waiting block!");
                    warn!(
                        "block should proposed on height {:?}, but got block on height {:?}",
                        last_waiting_block.block.id + 1,
                        author_block.block.block.id
                    );
                    return;
                }
                if last_waiting_block.get_hash() != author_block.block.block.prev_hash {
                    warn!(
                        "malicious block proposed by author {:?}!",
                        author_block.block.block.peer_id
                    );
                    warn!("malicious block proposed, invalid previous block hash compare to waiting block!");
                    warn!(
                        "previous_hash shuold be {:?}, but previous hash is {:?}",
                        last_waiting_block.get_hash(),
                        author_block.block.get_hash()
                    );
                    return;
                }
                let custom_header: CustomHeaders =
                    match deserialize(&author_block.block.auth_headers) {
                        Ok(value) => value,
                        Err(_) => {
                            warn!("block custom headers couldn't deserialized");
                            return;
                        }
                    };
                let last_custom_header: CustomHeaders =
                    match deserialize(&last_waiting_block.auth_headers) {
                        Ok(value) => value,
                        Err(_) => {
                            if last_waiting_block.block.id == 0 {
                                CustomHeaders {
                                    timestamp: 0,
                                    round_number: 0,
                                }
                            } else {
                                return;
                            }
                        }
                    };
                if custom_header.round_number <= last_custom_header.round_number {
                    warn!(
                        "malicious block proposed by author {:?}!",
                        author_block.block.block.peer_id
                    );
                    warn!(
                        "malicious block proposed, invalid round number compare to waiting block!"
                    );
                    warn!(
                        "block should proposed higher round number then {:?}, but got block on round number {:?}",
                        last_custom_header.round_number,
                        custom_header.round_number
                    );
                    return;
                }
                if custom_header.timestamp <= last_custom_header.timestamp {
                    warn!(
                        "malicious block proposed by author {:?}!",
                        author_block.block.block.peer_id
                    );
                    warn!("malicious block proposed, invalid timestamp compare to waiting block!");
                    warn!(
                        "block should proposed higher timestamp then {:?}, but got block on timestamp {:?}",
                        last_custom_header.timestamp,
                        custom_header.timestamp
                    );
                    return;
                }
            }
            None => {
                let snapshot: Box<dyn Snapshot> = snapshot_db();
                {
                    let schema = SchemaSnap::new(&snapshot);
                    if schema.get_blockchain_length() != author_block.block.block.id {
                        warn!(
                            "malicious block proposed by author {:?}!",
                            author_block.block.block.peer_id
                        );
                        warn!("malicious block proposed, invalid height from snapshot!");
                        warn!(
                            "block should proposed on height {:?}, but got block on height {:?}",
                            schema.get_blockchain_length(),
                            author_block.block.block.id
                        );
                        return;
                    }
                    if schema.get_root_block_hash() != author_block.block.block.prev_hash {
                        warn!(
                            "malicious block proposed by author {:?}!",
                            author_block.block.block.peer_id
                        );
                        warn!("malicious block proposed, invalid previous block hash compare to snapshot!");
                        warn!(
                            "previous_hash shuold be {:?}, but previous hash is {:?}",
                            schema.get_root_block_hash(),
                            author_block.block.get_hash()
                        );
                        return;
                    }
                    let custom_header: CustomHeaders =
                        match deserialize(&author_block.block.auth_headers) {
                            Ok(value) => value,
                            Err(_) => {
                                warn!("block custom headers couldn't deserialized");
                                return;
                            }
                        };
                    let root_block: SignedBlock = match schema.get_root_block() {
                        Some(block) => block,
                        None => {
                            warn!("previous block couldn't found");
                            return;
                        }
                    };
                    let last_custom_header: CustomHeaders =
                        match deserialize(&root_block.auth_headers) {
                            Ok(value) => value,
                            Err(_) => {
                                if root_block.block.id == 0 {
                                    CustomHeaders {
                                        timestamp: 0,
                                        round_number: 0,
                                    }
                                } else {
                                    warn!("block custom headers couldn't deserialized");
                                    return;
                                }
                            }
                        };
                    if custom_header.round_number <= last_custom_header.round_number {
                        warn!(
                            "malicious block proposed by author {:?}!",
                            author_block.block.block.peer_id
                        );
                        warn!(
                            "malicious block proposed, invalid round number compare to snapshot!"
                        );
                        warn!(
                            "block should proposed higher round number then {:?}, but got block on round number {:?}",
                            last_custom_header.round_number,
                            custom_header.round_number
                        );
                        return;
                    }
                    if custom_header.timestamp <= last_custom_header.timestamp {
                        warn!(
                            "malicious block proposed by author {:?}!",
                            author_block.block.block.peer_id
                        );
                        warn!("malicious block proposed, invalid timestamp compare to snapshot!");
                        warn!(
                            "block should proposed higher timestamp then {:?}, but got block on timestamp {:?}",
                            last_custom_header.timestamp,
                            custom_header.timestamp
                        );
                        return;
                    }
                }
            }
        }
        // we cannot validate block state
        // let author_block: AuthorBlock = AuthorBlock::create(signed_block.clone());
        // AuraMessageSender::send_author_block_msg(sender, author_block);
        let block_acceptance: BlockAcceptance =
            BlockAcceptance::create(&meta_data_obj.kp, author_block.block.get_hash());
        AuraMessageSender::send_block_acceptance_msg(&mut meta_data_obj.sender, block_acceptance);
        info!(
            "block accepted, created by {:?} with id {:?}, & hash {:?}",
            author_block.block.block.peer_id,
            author_block.block.block.id,
            author_block.block.get_hash().to_hex()
        );
        waiting_blocks_queue.last_block_hash = author_block.block.get_hash().to_hex();
        waiting_blocks_queue.last_block_acceptance.clear();
        waiting_blocks_queue
            .last_block_acceptance
            .insert(meta_data_obj.public_key.clone());
        waiting_blocks_queue
            .last_block_acceptance
            .insert(author_block.block.block.peer_id.clone());
        waiting_blocks_queue.queue.push(author_block.block);
    }

    // fn will process incoming BlockAcceptenceEnum data
    fn handle_block_acceptence_enum(
        block_acceptance: BlockAcceptance,
        waiting_blocks_queue: &mut WaitingBLocksQueue,
        meta_data_obj: &MetaData,
    ) {
        // data coming from verifed validator
        if !meta_data_obj
            .validator_mapping
            .contains_key(&block_acceptance.public_key)
        {
            warn!(
                "Data coming from untrusted source {:?}",
                block_acceptance.public_key
            );
            return;
        }

        // block acceptance for the correct_block
        if waiting_blocks_queue.last_block_hash != block_acceptance.block_hash.to_hex() {
            warn!("Data coming for different block");
            warn!(
                "current waiting block hash {:?} & data came for {:?}",
                waiting_blocks_queue.last_block_hash,
                block_acceptance.block_hash.to_hex()
            );
            return;
        }

        // verify data signature using sender's public key
        if !block_acceptance.verify() {
            warn!(
                "malicious aceeptance came from {:?}",
                block_acceptance.public_key
            );
            return;
        }
        info!(
            "valid block acceptance came from-> {:?}",
            block_acceptance.public_key
        );
        waiting_blocks_queue
            .last_block_acceptance
            .insert(block_acceptance.public_key);
    }

    // fn will handle incoming RoundOwnerEnum data
    fn handle_round_owner_enum(
        round_owner: RoundOwner,
        waiting_blocks_queue: &mut WaitingBLocksQueue,
        meta_data_obj: &MetaData,
    ) {
        if waiting_blocks_queue.queue.len() == 0 {
            info!("no waiting block to check aceeptance");
            return;
        }
        let current_owner: String = Aura::primary_leader(&meta_data_obj);
        if current_owner != round_owner.public_key {
            warn!(
                "malicious round owner claim created by {:?}",
                round_owner.public_key
            );
            return;
        }
        if round_owner.verify(meta_data_obj.step_time) {
            if String::from("temp_hash") != waiting_blocks_queue.last_block_hash.clone() {
                info!(
                    "block accepted by {:?}",
                    waiting_blocks_queue.last_block_acceptance
                );
                let got_votes = waiting_blocks_queue.last_block_acceptance.len() as u64;
                let minimum_votes: u64 = (meta_data_obj.validator_pool_size * 2) / 3;
                if minimum_votes <= got_votes {
                    waiting_blocks_queue.last_block_acceptance.clear();
                    waiting_blocks_queue.last_block_hash = String::from("temp_hash");
                // let signed_block: &SignedBlock = waiting_blocks_queue.queue.last().unwrap();
                // POOL.sync_pool(&signed_block.block.txn_pool);
                } else {
                    let length = waiting_blocks_queue.queue.len();
                    waiting_blocks_queue.last_block_hash = String::from("temp_hash");
                    warn!(
                        "last block got votes {:?} and required {:?}",
                        got_votes, minimum_votes
                    );
                    waiting_blocks_queue.queue.remove(length - 1);
                    error!("last block couldn't  got majority either delete the block or restart the consensus");
                }
            } else {
                info!("last block hash is NULL, can't initiate block acceptance process");
            }
        } else {
            warn!("data is either tempered or delayed/replayed");
        }
    }

    // fn will update waiting blocks in sequence to local db
    fn process_blocks(blocks_count: usize, waiting_blocks_queue: &mut WaitingBLocksQueue) {
        let mut blocks_count = blocks_count;
        while blocks_count > 0 {
            let signed_block: SignedBlock = waiting_blocks_queue.queue.remove(0);
            let fork = fork_db();
            {
                let mut schema = SchemaFork::new(&fork);
                if schema.update_block(&signed_block) {
                    POOL.sync_pool(&signed_block.block.txn_pool);
                    debug!(
                        "block with id {} & hash {} added in database",
                        signed_block.block.id,
                        signed_block.object_hash()
                    );
                } else {
                    error!(
                        "block with id {} & hash {} couldn't added in database",
                        signed_block.block.id,
                        signed_block.object_hash()
                    );
                }
            }
            patch_db(fork);
            blocks_count = blocks_count - 1;
        }
        info!("Blocks are updated in the database");
    }

    // fn will finalise blocks periodically in permanent db
    fn finalise_waiting_blocks(
        step_time: u64,
        waiting_blocks_queue: Arc<Mutex<WaitingBLocksQueue>>,
        meta_data: Arc<Mutex<MetaData>>,
    ) {
        loop {
            {
                let mut waiting_blocks_queue_obj = waiting_blocks_queue.lock().unwrap();
                let meta_data_obj = meta_data.lock().unwrap();
                let queue_length: usize = waiting_blocks_queue_obj.queue.len();
                if queue_length > meta_data_obj.block_queue_size + 1 {
                    debug!("queue length {:?}", queue_length);
                    let blocks_to_be_confirmed: usize = queue_length / 3 * 2;
                    Aura::process_blocks(blocks_to_be_confirmed, &mut waiting_blocks_queue_obj);
                    debug!(
                        "after processing queue length {:?}",
                        waiting_blocks_queue_obj.queue.len()
                    );
                }
            }
            thread::sleep(Duration::from_secs(step_time));
        }
    }

    // fn will listen incoming data from other peers via P2P system
    fn aura_msg_receiver(
        waiting_blocks_queue: Arc<Mutex<WaitingBLocksQueue>>,
        meta_data: Arc<Mutex<MetaData>>,
        rx: Arc<Mutex<Receiver<Option<Vec<u8>>>>>,
    ) {
        thread::spawn(move || {
            block_on(future::poll_fn(move |cx: &mut Context| {
                loop {
                    match rx.lock().unwrap().poll_next_unpin(cx) {
                        Poll::Ready(Some(msg)) => match msg {
                            None => info!("Empty msg received !"),
                            Some(msgtype) => {
                                if let Ok(msgtype) =
                                    deserialize::<AuraMessageTypes>(msgtype.as_slice())
                                {
                                    match msgtype {
                                        AuraMessageTypes::AuthorBlockEnum(data) => {
                                            let author_block: AuthorBlock = data;
                                            info!("AuthorBlock data received");
                                            let mut waiting_blocks_queue_obj =
                                                waiting_blocks_queue.lock().unwrap();
                                            let mut meta_data_obj = meta_data.lock().unwrap();
                                            Aura::handle_author_block_enum(
                                                author_block,
                                                &mut waiting_blocks_queue_obj,
                                                &mut meta_data_obj,
                                            );
                                        }
                                        AuraMessageTypes::BlockAcceptanceEnum(data) => {
                                            let block_acceptance: BlockAcceptance = data;
                                            info!("BlockAcceptance data received");
                                            let mut waiting_blocks_queue_obj =
                                                waiting_blocks_queue.lock().unwrap();
                                            let meta_data_obj = meta_data.lock().unwrap();
                                            Aura::handle_block_acceptence_enum(
                                                block_acceptance,
                                                &mut waiting_blocks_queue_obj,
                                                &meta_data_obj,
                                            );
                                        }
                                        AuraMessageTypes::RoundOwnerEnum(data) => {
                                            let round_config: RoundOwner = data;
                                            info!("RoundOwner data received");
                                            let mut waiting_blocks_queue_obj =
                                                waiting_blocks_queue.lock().unwrap();
                                            let meta_data_obj = meta_data.lock().unwrap();
                                            Aura::handle_round_owner_enum(
                                                round_config,
                                                &mut waiting_blocks_queue_obj,
                                                &meta_data_obj,
                                            );
                                        }
                                    }
                                }
                            }
                        },
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

    // fn will create new block to propose after processing waiting blocks
    fn propose_block(
        &self,
        waiting_blocks_queue: &WaitingBLocksQueue,
        meta_data: &MetaData,
    ) -> SignedBlock {
        let fork = fork_db();
        let mut schema = SchemaFork::new(&fork);
        for each_block in waiting_blocks_queue.queue.iter() {
            debug!("blocks order {:?}", each_block.block.id);
            schema.update_block(each_block);
        }
        let timestamp: u64 = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let round_number: u64 = Aura::calculate_round_number(meta_data);
        let custom_headers: CustomHeaders = CustomHeaders {
            timestamp,
            round_number,
        };
        let custom_headers: Vec<u8> = match serialize(&custom_headers) {
            Ok(value) => value,
            Err(_) => Vec::new(),
        };
        if self.force_sealing {
            schema.create_block(&self.keypair, custom_headers)
        } else {
            let (_fork_instance, signed_block) =
                schema.forge_new_block(&self.keypair, custom_headers);
            signed_block
        }
    }

    // fn will create new blocks periodically after checking round ownership
    fn state_machine(
        &mut self,
        waiting_blocks_queue: Arc<Mutex<WaitingBLocksQueue>>,
        meta_data: Arc<Mutex<MetaData>>,
        sender: &mut Sender<Option<MessageTypes>>,
    ) {
        let mut wait_till_one_round: u64 = (self.validator_mapping.len() * 10) as u64;
        wait_till_one_round = wait_till_one_round * self.leader_epoch;
        thread::sleep(Duration::from_millis(wait_till_one_round));
        let fork = fork_db();
        {
            let mut schema = SchemaFork::new(&fork);
            schema.sync_state();
        }
        patch_db(fork);
        #[allow(unused_assignments)]
        let mut leader_flag = false;
        loop {
            /*
            calculate round number and find out who is the leader
            need to check continuously
            if node is the leader, propose block on top of waiting block queue
            */
            {
                let mut am_i_leader: bool = false;
                {
                    let meta_data_obj = meta_data.lock().unwrap();
                    let current_leader: String = Aura::primary_leader(&meta_data_obj);
                    if current_leader == self.pk.clone() {
                        am_i_leader = true;
                    }
                }

                if am_i_leader {
                    info!("I'm the leader NOW!!");
                    let round_owner: RoundOwner = RoundOwner::create(&self.keypair);
                    AuraMessageSender::send_round_owner_msg(sender, round_owner.clone());
                    thread::sleep(Duration::from_millis(self.leader_epoch));
                    {
                        let meta_data_obj = meta_data.lock().unwrap();
                        let mut waiting_blocks_queue_obj = waiting_blocks_queue.lock().unwrap();
                        Aura::handle_round_owner_enum(
                            round_owner,
                            &mut waiting_blocks_queue_obj,
                            &meta_data_obj,
                        );
                        let signed_block: SignedBlock =
                            self.propose_block(&waiting_blocks_queue_obj, &meta_data_obj);
                        info!(
                            "new block created.. id {},hash {}",
                            signed_block.block.id,
                            signed_block.object_hash()
                        );
                        let author_block: AuthorBlock = AuthorBlock::create(signed_block.clone());
                        AuraMessageSender::send_author_block_msg(sender, author_block);
                        waiting_blocks_queue_obj.last_block_hash = signed_block.get_hash().to_hex();
                        waiting_blocks_queue_obj.queue.push(signed_block);
                        waiting_blocks_queue_obj.last_block_acceptance.clear();
                        waiting_blocks_queue_obj
                            .last_block_acceptance
                            .insert(meta_data_obj.public_key.clone());
                    }
                    leader_flag = true;
                }
            }
            if leader_flag {
                leader_flag = false;
                thread::sleep(Duration::from_millis(self.leader_epoch * 10));
            } else {
                thread::sleep(Duration::from_millis(self.leader_epoch));
            }
        }
    }

    pub fn init_aura_consensus(
        config: &Configuration,
        consensus_file_path: &str,
        sender: &mut Sender<Option<MessageTypes>>,
        msg_receiver: Arc<Mutex<Receiver<Option<Vec<u8>>>>>,
    ) {
        initialize_config(consensus_file_path);
        let aura_config: &crate::config::Configuration = &crate::config::AURA_CONFIG;
        let mut validator_mapping: HashMap<String, u64> = HashMap::new();
        for i in 0..aura_config.validator_set.len() {
            validator_mapping.insert(
                aura_config.validator_set[i].clone(),
                aura_config.validator_ids[i].clone(),
            );
        }

        let mut aura_obj = Aura {
            keypair: config.node.keypair.clone(),
            pk: hex::encode(config.node.keypair.public().encode()),
            validator_mapping: validator_mapping.clone(),
            leader_epoch: 100 * aura_config.step_time,
            force_sealing: aura_config.force_sealing,
            start_time: aura_config.start_time,
        };

        let consensus_meta_data = MetaData {
            validator_pool_size: aura_config.validator_set.len() as u64,
            validator_mapping,
            kp: config.node.keypair.clone(),
            public_key: hex::encode(config.node.keypair.public().encode()),
            sender: sender.clone(),
            start_time: aura_config.start_time,
            round_number: aura_config.round_number,
            step_time: aura_config.step_time,
            block_queue_size: aura_config.block_list_size,
        };
        let meta_data = Arc::new(Mutex::new(consensus_meta_data));
        let waiting_blocks_queue: Arc<Mutex<WaitingBLocksQueue>> =
            Arc::new(Mutex::new(WaitingBLocksQueue::new()));
        Aura::aura_msg_receiver(
            waiting_blocks_queue.clone(),
            meta_data.clone(),
            msg_receiver,
        );
        if config.node.genesis_block {
            aura_obj.init_state(&config.db.dbpath, &mut sender.clone());
        } else {
            let fork = fork_db();
            {
                let mut schema = SchemaFork::new(&fork);
                schema.sync_state();
            }
            patch_db(fork);
        }
        let cloned_waiting_blocks_queue = waiting_blocks_queue.clone();
        let cloned_meta_data = meta_data.clone();
        thread::spawn(move || {
            Aura::finalise_waiting_blocks(
                aura_config.step_time,
                cloned_waiting_blocks_queue,
                cloned_meta_data,
            );
        });
        aura_obj.state_machine(
            waiting_blocks_queue.clone(),
            meta_data.clone(),
            &mut sender.clone(),
        );
    }
}
