extern crate consensus;
extern crate controllers;
extern crate ctrlc;
extern crate db_service;
extern crate p2plib;
extern crate schema;

#[macro_use]
extern crate log;
use libloading::{Library, Symbol};
use std::collections::HashMap;
use std::path::Path;
use controllers::client_controller::{ClientController, Controller};
use db_service::db_fork_ref::SchemaFork;
use db_service::db_layer::{fork_db, patch_db};
use generic_traits::traits::AppHandler;
use schema::appdata::{AppData, APPDATA};
use schema::block::SignedBlock;
use schema::signed_transaction::SignedTransaction;

use libp2p::{identity::PublicKey, PeerId};
use nodemsgprocessor::*;
use p2plib::messages::{CONSENSUS_MSG_TOPIC_STR, MSG_DISPATCHER, NODE_MSG_TOPIC_STR};
use p2plib::simpleswarm::SimpleSwarm;

use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use std::thread;
use utils::configreader;
use utils::configreader::{Configuration, NODETYPE};
use utils::logger::*;

fn validator_process() {
    let config: &Configuration = &configreader::GLOBAL_CONFIG;
    let pk: PublicKey = PublicKey::Ed25519(config.node.public.clone());
    let peer_id = PeerId::from_public_key(pk);
    info!("peer id = {:?}", peer_id);
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
    info!("Starting api_service");
    api_service.start(txn_sender);
    info!("Started api_service");

    //On pressing ctrl-C, the boolean variable terminate will be set to 'true' in ctrlc handler and
    //the thread execution counter will come out of the loop. If we need to join on any thread,
    //we can do that after the loop. We should share the same boolean variable with those threads which
    //can keep checking this variable and exit gracefully.
    while !terminate.load(Ordering::SeqCst) {
        std::thread::park();
    }
    info!("Stopping REST End Point");
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

fn load_apps() {
    let config: &Configuration = &configreader::GLOBAL_CONFIG;
    println!("config = {:?}", config.node);
    let mut app_iter = IntoIterator::into_iter(&config.node.client_apps);
    while let Some(app) = app_iter.next() {
        println!("loading library {:?}", app);
        let app_path = Path::new(app);
        println!("is file = {:?}", app_path.is_file());
        let applib = Library::new(app)
            .expect(format!("Loading App library {:?} failed", app.clone()).as_str());
        let app_register: Symbol<fn() -> Box<dyn AppHandler + Send>> =
            unsafe { applib.get(b"register_app") }.expect(
                format!(
                    "register_app symbol is not found in library {:?}",
                    app.clone()
                )
                .as_str(),
            );
        let app_handle = Arc::new(Mutex::new(app_register()));
        let app_name = app_handle.lock().unwrap().name();
        println!("Loaded app {:?}", app_name);
        APPDATA
            .lock()
            .unwrap()
            .appdata
            .insert(app_name.clone(), app_handle);
    }
}

fn main() {
    file_logger_init_from_yml(&String::from("log.yml"));
    info!("Node Bootstrapping");
    let config: &Configuration = &configreader::GLOBAL_CONFIG;
    load_apps();
    match config.node.node_type {
        NODETYPE::Validator => {
            validator_process();
        }
        NODETYPE::FullNode => {
            fullnode_process();
        }
    }
}
