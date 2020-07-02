extern crate aura;
extern crate controllers;
extern crate ctrlc;
extern crate db_service;
extern crate p2plib;
extern crate schema;

#[macro_use]
extern crate log;

mod nodemsgprocessor;
use aura::aura_interface as consensus_interface;
use controllers::client_controller::{ClientController, Controller};
use libloading::{Library, Symbol};
use schema::appdata::APPDATA;
use sdk::traits::AppHandler;
use std::path::Path;

use clap::{App, Arg};
use libp2p::{identity::PublicKey, PeerId};
use message_handler::constants;
use message_handler::messages::MSG_DISPATCHER;
use nodemsgprocessor::*;
use p2plib::simpleswarm::SimpleSwarm;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use std::thread;
use utils::configreader;
use utils::configreader::initialize_config;
use utils::configreader::{Configuration, NODETYPE};
use utils::logger::logger_init_from_yml;

fn validator_process(consensus_file_path: String) {
    let config: &Configuration = &configreader::GLOBAL_CONFIG;
    let pk: PublicKey = PublicKey::Ed25519(config.node.public.clone());
    let peer_id = PeerId::from_public_key(pk);
    info!("peer id = {:?}", peer_id);
    let mut swarm = SimpleSwarm::new();
    swarm
        .topic_list
        .push(String::from(constants::CONSENSUS.clone()));
    swarm.topic_list.push(String::from(constants::NODE.clone()));
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
        consensus_interface::Aura::init_aura_consensus(
            config,
            &consensus_file_path,
            &mut sender,
            consensus_msg_receiver_clone,
        )
    });
    thread::spawn(move || {
        let process = swarm.process(peer_id, config);
        process.expect("swarm messganing system broken");
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
    api_service.start_validator_controller(txn_sender);
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
    swarm.topic_list.push(String::from(constants::NODE.clone()));
    let mut node_msg_processor = NodeMsgProcessor::new(MSG_DISPATCHER.node_msg_receiver.clone());
    let txn_sender = swarm.tx.clone();
    {
        thread::spawn(move || {
            node_msg_processor.start();
        });
    }
    thread::spawn(move || {
        let process = swarm.process(peer_id, config);
        process.expect("swarm messganing system broken");
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
    api_service.start_fullnode_controller(txn_sender);
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
        let mut locked_app_data = APPDATA.lock().unwrap();
        locked_app_data.lib.push(Arc::new(applib));
        locked_app_data
            .appdata
            .insert(app_name.clone(), app_handle.clone());
    }
}

fn main() {
    let matches = App::new("SimpleBlockchain Framework")
        .version("0.1.0")
        .author("gaurav agarwal <gaurav.agarwal@talentica.com>")
        .about("SimpleBlockchain Framework Node Process")
        .arg(
            Arg::with_name("node_config_path")
                .short("c")
                .long("node_config")
                .takes_value(true)
                .help("node config file"),
        )
        .arg(
            Arg::with_name("consensus_config_path")
                .short("C")
                .long("consensus_config")
                .takes_value(true)
                .help("consensus config file"),
        )
        .arg(
            Arg::with_name("logger_file_path")
                .short("l")
                .long("logger")
                .takes_value(true)
                .help("logger file path"),
        )
        .get_matches();
    let config_file_path = matches
        .value_of("node_config_path")
        .unwrap_or("config.toml");

    let consensus_file_path: String = match matches.value_of("consensus_config_path") {
        Some(path) => String::from(path),
        None => panic!("kindly provide consensus file path"),
    };
    let logger_file_path = matches.value_of("logger_file_path").unwrap_or("log.yml");
    initialize_config(config_file_path);
    logger_init_from_yml(logger_file_path);
    info!("Node Bootstrapping");
    let config: &Configuration = &configreader::GLOBAL_CONFIG;
    load_apps();
    match config.node.node_type {
        NODETYPE::Validator => {
            validator_process(consensus_file_path);
        }
        NODETYPE::FullNode => {
            fullnode_process();
        }
    }
}
