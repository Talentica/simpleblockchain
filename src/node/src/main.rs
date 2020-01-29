extern crate consensus;
extern crate p2plib;
extern crate schema;
use consensus::consensus_interface;
use p2plib::txn_pool_p2p;
use schema::transaction_pool::{TransactionPool, TxnPool};
use std::sync::{Arc, Mutex};
use std::thread;
use utils::configreader;
use utils::configreader::Configuration;

fn main() {
    let config: &Configuration = &configreader::GLOBAL_CONFIG;
    let transaction_pool: TransactionPool = TransactionPool::new();
    let object = Arc::new(Mutex::new(transaction_pool));
    let clone1 = object.clone();
    let clone2 = object.clone();
    // println!("db path {:?}", config.db.dbpath);
    // println!("public key {:?}", config.node.hex_public);
    // println!(
    //     "node type {:?},  genesis_block {}",
    //     config.node.node_type, config.node.genesis_block
    // );

    // this thread will be responsible for adding txn in txn_pool
    let mut threads = Vec::new();
    let handle = thread::spawn(move || {
        txn_pool_p2p::add_txn_to_txn_pool(&config.node.keypair.clone(), clone2)
    });
    threads.push(handle);

    // this thread will be responsible for whole consensus part.
    // in future this thread will spwan new child thread accrding to consensus requirement.
    let handle =
        thread::spawn(move || consensus_interface::Consensus::init_consensus(config, clone1));
    threads.push(handle);
    for each in threads {
        each.join().unwrap();
    }
}
