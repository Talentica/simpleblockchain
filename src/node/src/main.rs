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
use p2plib::messages::Message;
use p2plib::messages::*;
use p2plib::simpleswarm::SimpleSwarm;
use libp2p::{PeerId, identity::PublicKey};

fn main() {
    let config: &Configuration = &configreader::GLOBAL_CONFIG;
    let pk: PublicKey = PublicKey::Ed25519(config.node.public.clone());
    let peer_id = PeerId::from_public_key(pk);
    println!("peer id = {:?}", peer_id);
    let mut swarm = SimpleSwarm::new();
    swarm.topic_list.push(String::from(BlockCreate::TOPIC));
    swarm
        .topic_list
        .push(String::from(TransactionCreate::TOPIC));
    swarm.topic_list.push(String::from(SignedBlock::TOPIC));
    swarm
        .topic_list
        .push(String::from(SignedTransaction::TOPIC));
    swarm.process(peer_id, config);

    let transaction_pool: TransactionPool = TransactionPool::new();
    let object = Arc::new(Mutex::new(transaction_pool));
    let clone1 = object.clone();
    let clone2 = object.clone();
    

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
