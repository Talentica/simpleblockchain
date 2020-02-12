mod nodemsgprocessor;
use libp2p::PeerId;
use nodemsgprocessor::*;
use p2plib::simpleswarm::SimpleSwarm;
use services::*;
use utils::configreader;
use utils::configreader::Configuration;
extern crate ctrlc;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use p2plib::messages::Message;
use p2plib::messages::*;

use std::{thread, time};

fn test_publish() {
    let config: &Configuration = &configreader::GLOBAL_CONFIG;
    let peer_id = PeerId::from_public_key(config.node.public.clone());
    println!("peer id = {:?}", peer_id);
    let mut swarm = SimpleSwarm::new();
    swarm.topic_list.push(String::from(BlockCreate::TOPIC));
    swarm
        .topic_list
        .push(String::from(TransactionCreate::TOPIC));

    let mut node_msg_processor = NodeMsgProcessor::new(MSG_DISPATCHER.node_msg_receiver.clone());
    let mut tx = swarm.tx.clone();

    {
        thread::spawn(move || {
            node_msg_processor.start();
        });
    }

    thread::spawn(move || {
        let mut ictr: i64 = 0;
        const NUM_MSG: i64 = 10;
        loop {
            ictr += 1;
            if ictr > NUM_MSG {
                break;
            }
            let msg1 = Some(MessageTypes::NodeMsg(NodeMessageTypes::BlockCreate(
                BlockCreate {
                    height: ictr,
                    hash: String::from("test"),
                },
            )));
            let msg2 = Some(MessageTypes::NodeMsg(NodeMessageTypes::TransactionCreate(
                TransactionCreate {
                    nonce: ictr,
                    payload: String::from("payload"),
                    signature: String::from("abcdefg"),
                },
            )));

            thread::sleep(time::Duration::from_secs(2));
            tx.try_send(msg1);
            tx.try_send(msg2);
        }
    });

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
    let config: &Configuration = &configreader::GLOBAL_CONFIG;
    let peer_id = PeerId::from_public_key(config.node.public.clone());
    println!("peer id = {:?}", peer_id);
    std::env::set_var("RUST_BACKTRACE", "1");
    //Register the Ctrl-C handler so that user can use it to exit the application gracefully.
    let terminate = Arc::new(AtomicBool::new(false));
    register_signals(Arc::clone(&terminate));

    let mut swarm = SimpleSwarm::new();
    swarm.topic_list.push(String::from(BlockCreate::TOPIC));
    swarm
        .topic_list
        .push(String::from(TransactionCreate::TOPIC));

    let mut node_msg_processor = NodeMsgProcessor::new(MSG_DISPATCHER.node_msg_receiver.clone());
    {
        thread::spawn(move || {
            node_msg_processor.start();
        });
    }
    swarm.process(peer_id, config);

    //Starting the Transaction Service
    //TODO: host/port details need to come from config
    let port_from_config = 8089;
    let host_from_config = "127.0.0.1".to_string();
    let mut api_service =
        transaction_service::TransactionService::new(&host_from_config, port_from_config);
    println!("Starting api_service");
    api_service.start();
    println!("Started api_service");

    //On pressing ctrl-C, the boolean variable terminate will be set to 'true' in ctrlc handler and
    //the thread execution counter will come out of the loop. If we need to join on any thread,
    //we can do that after the loop. We should share the same boolean variable with those threads which
    //can keep checking this variable and exit gracefully.
    while !terminate.load(Ordering::SeqCst) {
        std::thread::park();
    }
    println!("Stopping REST End Point");
    api_service.stop(); //blocking call
}
