use async_std::{io, task};
use futures::{future, prelude::*};
use libp2p::{
    floodsub::{self, Floodsub, FloodsubEvent},
    identity,
    mdns::{Mdns, MdnsEvent},
    swarm::NetworkBehaviourEventProcess,
    Multiaddr, NetworkBehaviour, PeerId, Swarm,
};
use std::{
    error::Error,
    task::{Context, Poll},
};

use utils::configreader;
use utils::configreader::Configuration;

use utils::crypto::keypair;

use super::p2pbehaviour::P2PBehaviour;

pub struct SimpleSwarm;

impl SimpleSwarm {
    pub fn process(peer_id: PeerId, config: &Configuration) -> Result<(), Box<dyn Error>> {
        // let transport = libp2p::build_tcp_ws_secio_mplex_yamux(libp2p::identity::Keypair::Ed25519(
        // config.node.keypair.clone(),
        // ))
        // .unwrap();
        let transport = libp2p::build_development_transport(libp2p::identity::Keypair::Ed25519(
            config.node.keypair.clone(),
        ))
        .unwrap();
        let mut behaviour = P2PBehaviour::new(peer_id.clone());
        let mut swarm = {
            behaviour.subscribe(String::from("test-msg"));
            Swarm::new(transport, behaviour, peer_id)
        };

        Swarm::listen_on(
            &mut swarm,
            format!("{}{}", "/ip4/0.0.0.0/tcp/", config.node.p2p_port)
                .parse()
                .unwrap(),
        )
        .unwrap();

        let mut listening = false;
        let mut stdin = io::BufReader::new(io::stdin()).lines();

        task::block_on(future::poll_fn(move |cx: &mut Context| {
            loop {
                match stdin.try_poll_next_unpin(cx)? {
                    Poll::Ready(Some(line)) => {
                        swarm.floodsub.publish(
                            &floodsub::TopicBuilder::new("test-msg").build(),
                            line.as_bytes(),
                        );
                        println!("read data {:?}", line);
                    }
                    Poll::Ready(None) => panic!("Stdin closed"),
                    Poll::Pending => break,
                }
            }
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
