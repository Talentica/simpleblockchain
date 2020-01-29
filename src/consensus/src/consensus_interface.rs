extern crate db;
extern crate db_service;
extern crate schema;
extern crate utils;
use chrono::prelude::Utc;
use db::db_layer::{fork_db, patch_db, snapshot_db};
use db_service::db_fork_ref::SchemaFork;
use exonum_crypto::Hash;
use exonum_merkledb::{Fork, ObjectHash};
use schema::block::{Block, BlockTraits, SignedBlock, SignedBlockTraits};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use utils::configreader::{Configuration, NODETYPE};
use utils::keypair::{CryptoKeypair, Keypair, KeypairType};

pub struct Consensus {
    keypair: KeypairType,
}

/*{
    Basic idea behind generic consensus module.
    point one -> run consensus() which will take config as a parameter and will decide genesis need to create or
    need to fetch data from other peers

    INIT_STATE(CONFIG, STATE){
        either
            GENESIS
        OR
            FETCH DATA UP TO CURRENT STATE
    }

    INIT_REQUIRED_CONNECTION_ESTD(){
        MAKE CONNECTION WITH MINIMUM REQUIRED PEERS ( PING..PONG)
        set connection config such as connection name
            example proposal block will be transferred  over xyz chat with sone zzz prefix etc.
                    fetch data will be transferred over sss ......
    }

    AM_I_ELIGIBLE(){
        CHECK WHETHERE NODE IS ELIGIBLE TO PROPOSE BLOCK
        example in gosig we use vrf and probabiltiy check
                in POW every-one can create block
        return bool, required stats
    }

    LISTENING_TO_P2P_FOR_CONSENSUS(){
        CONSENSUS_ACHIEVED()
        COMMIT_BLOCK()
    }

    PROPOSE_BLOCK(){
        CHECK FOR CONSENSUS ON BLOCK
    }

    START_MINING(){
        LOOP{
            AM_I_ELIGIBLE
                SCHEMA::CREATE_BLOCK();
                self.PROPOSE_BLOCK();
                self.LISTENING_TO_P2P_FOR_CONSENSUS();
        }
    }

    AUDITOR(){

    }

    run_consensus( config){
        INIT_REQUIRED_CONNECTION_ESTD()
        // init state means we either creating genesis block or fetch data upto current state
        // in both case we are simply updating all trie and block-list and duming it into db
        // no local variable state changes in my understanding.
        STATE::NEW()
        INIT_STATE(CONFIG, STATE)

        NODE_BEHAVIOUR{
            EITHER
                START_MINING()
            OR
                AUDITOR()
        }


    }
}*/

impl Consensus {
    fn init_required_connection_estd() {
        println!(
            "fn init_required_connection_estd will be implemented after p2p module integration"
        );
    }

    fn init_state(&self, genesis_block: bool, _db_path: &String) {
        if genesis_block {
            let fork = fork_db();
            {
                let schema = SchemaFork::new(&fork);
                schema.initialize_db(&self.keypair);
            }
            patch_db(fork);
        } else {
            println!("FETCH DATA UP TO CURRENT STATE");
            println!("init_state for genesis block-false, will be implemented after p2p module integration")
        }
    }

    fn am_i_eligible(&self) -> bool {
        true
    }

    fn consensus_achieved(&self, fork: Option<Fork>, _signed_block: Option<&SignedBlock>) -> Fork {
        let fork_obj = fork.unwrap_or_else(|| fork_db());
        fork_obj
    }

    fn commit_block(&self, fork: Fork) {
        patch_db(fork);
    }

    fn listening_to_p2_p_for_consensus(
        &self,
        fork: Option<Fork>,
        signed_block: Option<&SignedBlock>,
    ) {
        let fork_obj = self.consensus_achieved(fork, signed_block);
        self.commit_block(fork_obj);
    }

    fn propose_block(&self, _signed_block: &SignedBlock) -> bool {
        // call P2P fn to broadcast this block for consensus
        true
    }

    fn start_mining(
        &self,
        locked_txn_pool: Arc<std::sync::Mutex<schema::transaction_pool::TransactionPool>>,
    ) {
        loop {
            thread::sleep(Duration::from_millis(5000));
            if self.am_i_eligible() {
                // no polling machenism of txn_pool and create block need to implement or modified here
                // if one want to change the create_block and txn priority then change/ implment that part in
                // schema operations and p2p module
                let fork = fork_db();
                let mut signed_block: SignedBlock =
                    SignedBlock::create_block(Block::genesis_block(), vec![]);
                {
                    let schema = SchemaFork::new(&fork);
                    let mut txn_pool = locked_txn_pool.lock().unwrap();
                    signed_block = schema.create_block(&self.keypair, &mut txn_pool);
                    println!("{:?}", signed_block);
                }
                if self.propose_block(&signed_block) {}
                self.listening_to_p2_p_for_consensus(Some(fork), Some(&signed_block));
            } else {
                self.listening_to_p2_p_for_consensus(Option::None, Option::None);
            }
        }
    }

    fn auditor(&self) {}

    pub fn init_consensus(
        config: &Configuration,
        txn_pool: Arc<std::sync::Mutex<schema::transaction_pool::TransactionPool>>,
    ) {
        Consensus::init_required_connection_estd();
        let consensus_obj = Consensus {
            keypair: config.node.keypair.clone(),
        };
        consensus_obj.init_state(config.node.genesis_block, &config.db.dbpath);

        match config.node.node_type {
            NODETYPE::FullNode => consensus_obj.start_mining(txn_pool),
            NODETYPE::Validator => consensus_obj.auditor(),
        }
    }
}
