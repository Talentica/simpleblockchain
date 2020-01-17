extern crate db;
extern crate db_service;
extern crate schema;
extern crate utils;
use chrono::prelude::Utc;
use db::db_layer::{fork_db, patch_db, snapshot_db};
use db_service::{db_fork_ref::SchemaFork, db_snapshot_ref::SchemaSnap};
use exonum_crypto::Hash;
use exonum_merkledb::{Fork, ObjectHash};
use schema::block::SignedBlock;
use schema::transaction::{SignedTransaction, Txn};
use schema::transaction_pool::{TransactionPool, TxnPool};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use utils::keypair::{CryptoKeypair, Keypair, KeypairType};

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
        let mut secret =
            hex::decode("97ba6f71a5311c4986e01798d525d0da8ee5c54acbf6ef7c3fadd1e2f624442f")
                .expect("invalid secret");
        let keypair = Keypair::generate_from(secret.as_mut_slice());
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
        let fork_1 = fork_db();
        {
            let schema = SchemaFork::new(&fork);
            println!(
                "merkle roots {} {}",
                schema.txn_trie_merkle_hash(),
                schema.state_trie_merkle_hash()
            );
            let _signed_block = schema.create_block(&self.keypair, &mut self.txn_pool);
            let schema_1 = SchemaFork::new(&fork_1);
            println!(
                "merkle root {} {}",
                schema_1.txn_trie_merkle_hash(),
                schema_1.state_trie_merkle_hash()
            );
            println!(
                "validation check  {}",
                schema_1.validate_block(&_signed_block.block, &mut self.txn_pool)
            )
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
        println!("\nblock generate callled");
        println!(
            "(txn_pool_len) -> ( {})",
            db_obj.txn_pool.length_order_pool()
        );
        db_obj.generate_block();
        println!("block proposed and committed\n");
    });
    threads.push(handle);

    loop {
        thread::sleep(Duration::from_millis(99));
        let clone = object.clone();
        // thread for adding txn into txn_pool
        thread::spawn(move || {
            let mut db_obj = clone.lock().unwrap();
            let one = SignedTransaction::generate(&db_obj.keypair);
            let time_instant = Utc::now().timestamp_nanos();
            db_obj.txn_pool.insert_op(&time_instant, &one);
        });
    }
}

pub fn check_blockchain() {
    let fork = fork_db();
    {
        let schema = SchemaFork::new(&fork);
        let blocks = schema.blocks();
        let mut i = 1;
        let mut prev_hash = blocks.get(0).unwrap().object_hash();
        while i < blocks.len() {
            let block: SignedBlock = blocks.get(i).unwrap();
            assert_eq!(prev_hash, block.block.prev_hash);
            prev_hash = block.object_hash();
            i = i + 1;
        }
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
            let time_instant = Utc::now().timestamp_nanos();
            block_chain.txn_pool.insert_op(&time_instant, &signed_txn);
        }
        while block_chain.txn_pool.length_order_pool() > 0 {
            thread::sleep(Duration::from_millis(100));
            block_chain.generate_block();
            println!("{:?}", block_chain.txn_pool.length_order_pool());
        }
    }
}
