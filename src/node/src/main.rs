extern crate consensus;
extern crate controllers;
extern crate ctrlc;
extern crate db_service;
extern crate p2plib;
extern crate schema;

#[macro_use]
extern crate log;

mod nodemsgprocessor;
use consensus::consensus_interface;
use controllers::client_controller::{ClientController, Controller};
use libp2p::{identity::PublicKey, PeerId};
use nodemsgprocessor::*;
use p2plib::messages::{CONSENSUS_MSG_TOPIC_STR, MSG_DISPATCHER, NODE_MSG_TOPIC_STR};
use p2plib::simpleswarm::SimpleSwarm;

use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread;
use utils::configreader;
use utils::configreader::{Configuration, NODETYPE};
use utils::logger::*;

fn validator_process() {
    let config: &Configuration = &configreader::GLOBAL_CONFIG;
    let pk: PublicKey = PublicKey::Ed25519(config.node.public.clone());
    let peer_id = PeerId::from_public_key(pk);
    debug!("peer id = {:?}", peer_id);
    let mut swarm = SimpleSwarm::new();
    for each in NODE_MSG_TOPIC_STR {
        swarm.topic_list.push(String::from(each.clone()));
    }
    for each in CONSENSUS_MSG_TOPIC_STR {
        swarm.topic_list.push(String::from(each.clone()));
    }
    let mut node_msg_processor = NodeMsgProcessor::new(MSG_DISPATCHER.node_msg_receiver.clone());
    let mut sender = swarm.tx.clone();
    let txn_sender = swarm.tx.clone();
    {
        thread::spawn(move || {
            node_msg_processor.start();
        });
    }

    // this thread will be responsible for whole consensus part.
    // in future this thread will spwan new child thread accrding to consensus requirement.
    let consensus_msg_receiver_clone = MSG_DISPATCHER.consensus_msg_receiver.clone();
    thread::spawn(move || {
        consensus_interface::Consensus::init_consensus(
            config,
            &mut sender,
            consensus_msg_receiver_clone,
        )
    });
    thread::spawn(move || {
        swarm.process(peer_id, config);
    });
    std::env::set_var("RUST_BACKTRACE", "1");
    //Register the Ctrl-C handler so that user can use it to exit the application gracefully.
    let terminate = Arc::new(AtomicBool::new(false));
    register_signals(Arc::clone(&terminate));
    //Starting the Transaction Service
    //TODO: host/port details need to come from config
    let port_from_config = config.node.client_port;
    let host_from_config = config.node.client_host.clone();
    let mut api_service = ClientController::new(&host_from_config, port_from_config);
    debug!("Starting api_service");
    api_service.start(txn_sender);
    debug!("Started api_service");

    //On pressing ctrl-C, the boolean variable terminate will be set to 'true' in ctrlc handler and
    //the thread execution counter will come out of the loop. If we need to join on any thread,
    //we can do that after the loop. We should share the same boolean variable with those threads which
    //can keep checking this variable and exit gracefully.
    while !terminate.load(Ordering::SeqCst) {
        std::thread::park();
    }
    debug!("Stopping REST End Point");
    api_service.stop(); //blocking call
}

fn fullnode_process() {
    let config: &Configuration = &configreader::GLOBAL_CONFIG;
    let pk: PublicKey = PublicKey::Ed25519(config.node.public.clone());
    let peer_id = PeerId::from_public_key(pk);
    let mut swarm = SimpleSwarm::new();
    for each in NODE_MSG_TOPIC_STR {
        swarm.topic_list.push(String::from(each.clone()));
    }
    for each in CONSENSUS_MSG_TOPIC_STR {
        swarm.topic_list.push(String::from(each.clone()));
    }
    let mut node_msg_processor = NodeMsgProcessor::new(MSG_DISPATCHER.node_msg_receiver.clone());
    {
        thread::spawn(move || {
            node_msg_processor.start();
        });
    }
    swarm.process(peer_id, config);
}

fn register_signals(terminate: Arc<AtomicBool>) {
    let thread = std::thread::current();
    ctrlc::set_handler(move || {
        terminate.store(true, Ordering::SeqCst);
        thread.unpark();
    })
    .expect("Error setting Ctrl-C handler");
}

fn main() {
    file_logger_init_from_yml(&String::from("log.yml"));
    debug!("Node Bootstrapping");
    let config: &Configuration = &configreader::GLOBAL_CONFIG;
    match config.node.node_type {
        NODETYPE::Validator => {
            validator_process();
        }
        NODETYPE::FullNode => {
            fullnode_process();
        }
    }
}
