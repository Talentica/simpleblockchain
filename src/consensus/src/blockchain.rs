extern crate db;
extern crate db_service;
extern crate schema;
extern crate utils;
use db::db_layer::{fork_db, patch_db, snapshot_db};
use db_service::{db_fork_ref::SchemaFork, db_snapshot_ref::SchemaSnap};
use exonum_crypto::Hash;
use exonum_merkledb::{Fork, ObjectHash};
use schema::transaction::{SignedTransaction, TransactionPool, Txn, TxnPool};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use utils::keypair::{CryptoKeypair, Keypair, KeypairType, PublicKey, Verify};

pub trait BlockchainTraits {
    fn new() -> Self;
    fn get_root_block(&self) -> Hash;
    fn commit_block(&mut self, fork: Fork);
    fn generate_block(&mut self);
}

pub struct BlockChain {
    keypair: KeypairType,
    root_hash: Hash,
    txn_pool: TransactionPool,
}

impl BlockchainTraits for BlockChain {
    // create genesis block and set its hash as root_hash of blockchain
    fn new() -> Self {
        let keypair = Keypair::generate();
        let snapshot = snapshot_db();
        let schema = SchemaSnap::new(&snapshot);
        if schema.is_db_initialized() {
            let root_hash = schema.get_root_block_hash();
            println!("DataBase Exists current hash {}", root_hash);
            return Self {
                keypair,
                root_hash,
                txn_pool: TransactionPool::new(),
            };
        }
        let fork = fork_db();
        {
            let schema = SchemaFork::new(&fork);
            schema.initialize_db(&keypair);
        }
        patch_db(fork);
        let snapshot = snapshot_db();
        let schema = SchemaSnap::new(&snapshot);
        let root_hash = schema.get_root_block_hash();
        println!("DataBase Created");
        Self {
            keypair,
            root_hash,
            txn_pool: TransactionPool::new(),
        }
    }

    fn get_root_block(&self) -> Hash {
        self.root_hash.clone()
    }

    fn commit_block(&mut self, fork: Fork) {
        patch_db(fork);
    }

    fn generate_block(&mut self) {
        let fork = fork_db();
        {
            let schema = SchemaFork::new(&fork);
            let _signed_block = schema.create_block(&self.keypair, &mut self.txn_pool);
        }
        self.commit_block(fork);
    }
}

pub fn poa_with_sep_th() {
    let block_chain_obj = BlockChain::new();
    let object = Arc::new(Mutex::new(block_chain_obj));
    let mut threads = Vec::new();
    let clone1 = object.clone();
    let handle = thread::spawn(move || loop {
        thread::sleep(Duration::from_millis(1000));
        let mut db_obj = clone1.lock().unwrap();
        db_obj.generate_block();
        println!("block proposed and committed");
        println!("(txn_pool_len) -> ( {})", db_obj.txn_pool.length_op());
    });
    threads.push(handle);

    loop {
        thread::sleep(Duration::from_millis(99));
        let clone = object.clone();
        // thread for adding txn into txn_pool
        thread::spawn(move || {
            let mut db_obj = clone.lock().unwrap();
            let one = SignedTransaction::generate(&db_obj.keypair);
            let time_instant = Instant::now();
            db_obj.txn_pool.insert_op(&time_instant, &one);
            // println!("lenght {:?}", db_obj.txn_pool.length_op());
        });
    }
}

#[cfg(test)]
mod tests_blocks {

    #[test]
    pub fn test_block_chain() {
        use super::*;
        let mut block_chain = BlockChain::new();
        for _i in 0..50 {
            let signed_txn = SignedTransaction::generate(&block_chain.keypair);
            let time_instant = Instant::now();
            block_chain.txn_pool.insert_op(&time_instant, &signed_txn);
        }
        while block_chain.txn_pool.length_op() > 0 {
            thread::sleep(Duration::from_millis(100));
            block_chain.generate_block();
            println!("{:?}", block_chain.txn_pool.length_op());
        }
    }
}
