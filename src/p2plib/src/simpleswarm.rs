use futures::{channel::mpsc::*, executor::*, future, prelude::*, task::*};
use libp2p::{
    floodsub::{self, Floodsub, FloodsubEvent},
    identity,
    mdns::{Mdns, MdnsEvent},
    swarm::NetworkBehaviourEventProcess,
    Multiaddr, NetworkBehaviour, PeerId, Swarm,
};
use std::error::Error;
use std::sync::{Arc, Mutex};

use utils::configreader;
use utils::configreader::Configuration;

use utils::crypto::keypair;

use super::messages::*;
use super::p2pbehaviour::P2PBehaviour;

pub struct SimpleSwarm {
    // behaviour: Option<P2PBehaviour<TSubstream>>,
    pub topic_list: Vec<String>,
    pub tx: Sender<Option<MessageTypes>>,
    pub rx: Receiver<Option<MessageTypes>>,
}

impl SimpleSwarm {
    pub fn new() -> Self {
        let (mut tx1, mut rx1) = channel::<Option<MessageTypes>>(4194304);
        SimpleSwarm {
            topic_list: Vec::new(),
            tx: tx1,
            rx: rx1,
        }
    }
    pub fn process(
        &mut self,
        peer_id: PeerId,
        config: &Configuration,
    ) -> Result<(), Box<dyn Error>> {
        // let transport = libp2p::build_tcp_ws_secio_mplex_yamux(libp2p::identity::Keypair::Ed25519(
        // config.node.keypair.clone(),
        // ))
        // .unwrap();
        let transport = libp2p::build_development_transport(libp2p::identity::Keypair::Ed25519(
            config.node.keypair.clone(),
        ))
        .unwrap();
        let mut behaviour = P2PBehaviour::new(peer_id.clone());
        for topic in &self.topic_list {
            behaviour.subscribe(&topic);
        }
        let mut swarm = Swarm::new(transport, behaviour, peer_id);
        // behaviour.unwrap().subscribe(String::from("test-msg"));

        Swarm::listen_on(
            &mut swarm,
            format!("{}{}", "/ip4/0.0.0.0/tcp/", config.node.p2p_port)
                .parse()
                .unwrap(),
        )
        .unwrap();

        let mut listening = false;
        // let mut stdin = io::BufReader::new(io::stdin()).lines();

        block_on(future::poll_fn(move |cx: &mut Context| {
            loop {
                match self.rx.poll_next_unpin(cx) {
                    Poll::Ready(Some(msg)) => {
                        println!("msg received {:?}", msg);
                        match msg {
                            None => println!("empty message !"),
                            Some(msgtype) => match msgtype {
                                MessageTypes::TransactionCreate(data) => swarm.floodsub.publish(
                                    &floodsub::TopicBuilder::new(data.topic()).build(),
                                    "txn create test data",
                                ),
                                MessageTypes::BlockCreate(data) => swarm.floodsub.publish(
                                    &floodsub::TopicBuilder::new(data.topic()).build(),
                                    "block create test data",
                                ),
                                _ => println!("unhandled msgs"),
                            },
                        }
                    }
                    Poll::Ready(None) => {
                        println!("channel closed !");
                        return Poll::Ready(Ok(()));
                        // Poll::Ready(());
                    }
                    Poll::Pending => break,
                }
            }
            // loop {
            //     match stdin.try_poll_next_unpin(cx)? {
            //         Poll::Ready(Some(line)) => {
            //             swarm.floodsub.publish(
            //                 &floodsub::TopicBuilder::new("test-msg").build(),
            //                 line.as_bytes(),
            //             );
            //             println!("read data {:?}", line);
            //         }
            //         Poll::Ready(None) => panic!("Stdin closed"),
            //         Poll::Pending => break,
            //     }
            // }
            loop {
                match swarm.poll_next_unpin(cx) {
                    Poll::Ready(Some(event)) => println!("{:?}", event),
                    Poll::Ready(None) => return Poll::Ready(Ok(())),
                    Poll::Pending => {
                        if !listening {
                            if let Some(a) = Swarm::listeners(&swarm).next() {
                                println!("Listening on {:?}", a);
                                listening = true;
                            }
                        }
                        break;
                    }
                }
            }
            Poll::Pending
        }))
    }
}
