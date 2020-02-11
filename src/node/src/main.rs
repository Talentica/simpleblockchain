#[macro_use]
extern crate lazy_static;
mod nodemsgprocessor;
use libp2p::PeerId;
use nodemsgprocessor::*;
use p2plib::simpleswarm::SimpleSwarm;
use utils::configreader;
use utils::configreader::Configuration;

use p2plib::messages::msg_dispatcher;
use p2plib::messages::Message;
use p2plib::messages::*;

use futures::channel::{mpsc, oneshot};
use futures::executor::{block_on, block_on_stream};
use futures::future::{poll_fn, FutureExt};
use futures::pin_mut;
use futures::sink::{Sink, SinkExt};
use futures::stream::{Stream, StreamExt};
use futures::task::{Context, Poll};
// use futures::{prelude::*};
use std::{thread, time};

use async_std::io;
// use async_std::io::stdin;
use std::sync::Arc;
use std::sync::Mutex;

fn test_publish() {
    // let (mut tx, mut rx) = mpsc::channel::<Option<MessageTypes>>(4194304x);
    // let (mut tx, mut rx) = mpsc::channel::<i32>(0);
    // let mut stdin = io::BufReader::new(io::stdin());//.lines();
    let config: &Configuration = &configreader::GLOBAL_CONFIG;
    let peer_id = PeerId::from_public_key(config.node.public.clone());
    println!("peer id = {:?}", peer_id);
    let mut swarm = SimpleSwarm::new();
    swarm.topic_list.push(String::from(BlockCreate::TOPIC));
    swarm
        .topic_list
        .push(String::from(TransactionCreate::TOPIC));

    let mut node_msg_processor = NodeMsgProcessor::new(msg_dispatcher.node_msg_receiver.clone());
    let mut tx = swarm.tx.clone();

    {
        thread::spawn(move || {
            node_msg_processor.start();
        });
    }

    thread::spawn(move || {
        loop {
            let msg1 = Some(MessageTypes::NodeMsg(NodeMessageTypes::BlockCreate(
                BlockCreate {
                    height: 1,
                    hash: String::from("test"),
                },
            )));
            let msg2 = Some(MessageTypes::ConsensusMsg(
                ConsensusMessageTypes::LeaderElect(TransactionCreate {
                    nonce: 1,
                    payload: String::from("payload"),
                    signature: String::from("abcdefg"),
                }),
            ));

            thread::sleep(time::Duration::from_secs(2));
            tx.try_send(msg1);
            tx.try_send(msg2);
        }
        // let res = tx.try_send(msg1);
        // let res = tx.try_send(1);
        // println!("msg1 send res = {:?}", res);
        // let res1 = tx.try_send(msg2);
        // println!("msg2 send res = {:?}", res1);
    });

    // block_on(poll_fn(move |cx| {
    //     loop {
    //         match rx.poll_next_unpin(cx) {
    //             Poll::Ready(Some(msg)) => {
    //                 println!("msg received {:?}", msg);
    //             }
    //             Poll::Ready(None) => {
    //                 println!("channel closed !");
    //                 return Poll::Ready(());
    //             }
    //             Poll::Pending => break,
    //         }
    //     }
    // loop {
    //     match stdin.poll_next_unpin(cx) {
    //         Poll::Ready(Some(line)) => {
    //             println!("read data {:?}", line);
    //             let x = line.unwrap().trim().parse::<i32>().unwrap();
    //             println!("send resulted in {:?}", tx.try_send(x).unwrap());
    //         }
    //         Poll::Ready(None) => panic!("Stdin closed"),
    //         Poll::Pending => break,
    //     }
    // }
    //     Poll::Pending
    // }));
    // for i in 0..2 {
    // let data = rx.poll_next();
    // println!("outer = {:?}", data);
    //     match data {
    //         Ok(Some(b)) => match b {
    //             MessageTypes::BlockCreate(msg) => {
    //                 println!("i = {},  {:?}", i, msg.topic());
    //             }
    //             MessageTypes::TransactionCreate(txncreate) => {
    //                 println!("i = {}, txn create = {:?}", i, txncreate.topic())
    //             }
    //             _ => println!("no match found"),
    //         },
    //         _ => println!("Error"),
    //     }
    // }));
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

    let mut node_msg_processor = NodeMsgProcessor::new(msg_dispatcher.node_msg_receiver.clone());
    let mut tx = swarm.tx.clone();
    {
        thread::spawn(move || {
            node_msg_processor.start();
        });
    }
    swarm.process(peer_id, config);
    // let transport = libp2p::build_development_transport(libp2p::identity::Keypair::Ed25519(
}
