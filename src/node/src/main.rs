use libp2p::PeerId;
use p2plib::simpleswarm::SimpleSwarm;
use utils::configreader;
use utils::configreader::Configuration;

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

fn main2() {
    let msg1 = Some(MessageTypes::BlockCreate(BlockCreate {}));
    // println!("data = {:?}", msg1.unwrap());
    println!("t = {:?}", TransactionCreate {}.topic());
    match msg1.unwrap() {
        MessageTypes::BlockCreate(a) => println!("inner data = {:?}", a.topic()),
        _ => println!("no"),
    }
}

fn test_publish() {
    // let (mut tx, mut rx) = mpsc::channel::<Option<MessageTypes>>(4194304);
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
    swarm.process(peer_id, config);
    let mut tx = swarm.tx.clone();

    thread::spawn(move || {
        loop {
            let msg1 = Some(MessageTypes::BlockCreate(BlockCreate {}));
            let msg2 = Some(MessageTypes::TransactionCreate(TransactionCreate {}));
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
    swarm.process(peer_id, config);
    // let transport = libp2p::build_development_transport(libp2p::identity::Keypair::Ed25519(
}
