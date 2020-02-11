mod nodemsgprocessor;
use libp2p::PeerId;
use nodemsgprocessor::*;
use p2plib::simpleswarm::SimpleSwarm;
use utils::configreader;
use utils::configreader::Configuration;

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

fn main() {
    let config: &Configuration = &configreader::GLOBAL_CONFIG;
    let peer_id = PeerId::from_public_key(config.node.public.clone());
    println!("peer id = {:?}", peer_id);
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
}
