extern crate consensus;
extern crate p2plib;
extern crate schema;

mod nodemsgprocessor;
use consensus::consensus_interface;
use libp2p::{identity::PublicKey, PeerId};
use nodemsgprocessor::*;
use p2plib::messages::Message;
use p2plib::messages::*;
use p2plib::simpleswarm::SimpleSwarm;
use p2plib::txn_pool_p2p;
use std::thread;
use utils::configreader;
use utils::configreader::{Configuration, NODETYPE};

fn validator_process() {
    let config: &Configuration = &configreader::GLOBAL_CONFIG;
    let pk: PublicKey = PublicKey::Ed25519(config.node.public.clone());
    let peer_id = PeerId::from_public_key(pk);
    println!("peer id = {:?}", peer_id);
    let mut swarm = SimpleSwarm::new();
    swarm
        .topic_list
        .push(String::from(SignedLeaderElection::TOPIC));
    swarm.topic_list.push(String::from(BlockConsensus::TOPIC));
    swarm.topic_list.push(String::from(SignedBlock::TOPIC));
    swarm
        .topic_list
        .push(String::from(SignedTransaction::TOPIC));

    let mut node_msg_processor = NodeMsgProcessor::new(MSG_DISPATCHER.node_msg_receiver.clone());
    let mut sender = swarm.tx.clone();
    let mut txn_sender = swarm.tx.clone();
    {
        thread::spawn(move || {
            node_msg_processor.start();
        });
    }

    // this thread will be responsible for adding txn in txn_pool
    thread::spawn(move || {
        txn_pool_p2p::add_txn_to_txn_pool(&config.node.keypair.clone(), &mut txn_sender)
    });

    // this thread will be responsible for whole consensus part.
    // in future this thread will spwan new child thread accrding to consensus requirement.
    let mut consensus_msg_receiver_clone = MSG_DISPATCHER.consensus_msg_receiver.clone();
    thread::spawn(move || {
        consensus_interface::Consensus::init_consensus(config, &mut sender, consensus_msg_receiver_clone)
    });
    swarm.process(peer_id, config);
}

fn fullnode_process() {
    let config: &Configuration = &configreader::GLOBAL_CONFIG;
    let pk: PublicKey = PublicKey::Ed25519(config.node.public.clone());
    let peer_id = PeerId::from_public_key(pk);
    let mut swarm = SimpleSwarm::new();
    swarm
        .topic_list
        .push(String::from(SignedLeaderElection::TOPIC));
    swarm.topic_list.push(String::from(BlockConsensus::TOPIC));
    swarm.topic_list.push(String::from(SignedBlock::TOPIC));
    swarm
        .topic_list
        .push(String::from(SignedTransaction::TOPIC));

    let mut node_msg_processor = NodeMsgProcessor::new(MSG_DISPATCHER.node_msg_receiver.clone());
    {
        thread::spawn(move || {
            node_msg_processor.start();
        });
    }
    swarm.process(peer_id, config);
}

fn main() {
    let config: &Configuration = &configreader::GLOBAL_CONFIG;
    match config.node.node_type {
        NODETYPE::Validator => validator_process(),
        NODETYPE::FullNode => fullnode_process(),
    }
}
