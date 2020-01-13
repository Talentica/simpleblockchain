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

pub trait Message {
    const TOPIC: &'static str;
    fn handler(&self);
}

pub struct TransactionCreate {}

impl Message for TransactionCreate {
    const TOPIC: &'static str = "txn-create";
    fn handler(&self) {}
}

pub struct BlockCreate {}
impl Message for BlockCreate {
    const TOPIC: &'static str = "block-create";
    fn handler(&self) {}
}
pub enum MessageTypes {
    TransactionCreate(TransactionCreate),
    BlockCreate(BlockCreate),
    BlockFinalize,
    MsgTest,
}

/// Network behavior defined combining, floodsub and mdns (for discovery)
///
#[derive(NetworkBehaviour)]
struct P2PBehaviour<TSubstream: AsyncRead + AsyncWrite> {
    floodsub: Floodsub<TSubstream>,
    mdns: Mdns<TSubstream>,
}

impl<TSubstream: AsyncRead + AsyncWrite> P2PBehaviour<TSubstream> {
    pub fn new(peer_id: PeerId) -> Self {
        let mdns = task::block_on(Mdns::new()).unwrap();
        let mut behaviour = P2PBehaviour {
            floodsub: Floodsub::new(peer_id.clone()),
            mdns,
        };
        behaviour
    }
    pub fn subscribe(&mut self, topic_str: String) {
        let floodsub_topic = floodsub::TopicBuilder::new(topic_str).build();
        self.floodsub.subscribe(floodsub_topic);
    }
}

impl<TSubstream: AsyncRead + AsyncWrite> NetworkBehaviourEventProcess<MdnsEvent>
    for P2PBehaviour<TSubstream>
{
    fn inject_event(&mut self, mdns_event: MdnsEvent) {
        match mdns_event {
            MdnsEvent::Discovered(discovered_nodes) => {
                // println!("Discovered address {:?}", discovered_nodes);
                for (peer_id, _) in discovered_nodes {
                    //println!("peer discovered {}", peer_id);
                    self.floodsub.add_node_to_partial_view(peer_id);
                }
            }
            MdnsEvent::Expired(expired_nodes) => {
                println!("Expired address {:?}", expired_nodes);
                for (peer_id, _) in expired_nodes {
                    self.floodsub.remove_node_from_partial_view(&peer_id);
                }
            }
        }
    }
}

impl<TSubstream: AsyncRead + AsyncWrite> NetworkBehaviourEventProcess<FloodsubEvent>
    for P2PBehaviour<TSubstream>
{
    fn inject_event(&mut self, pubsub_event: FloodsubEvent) {
        match pubsub_event {
            FloodsubEvent::Message(msg) => {
                println!("Message received {:?}", msg);
            }
            FloodsubEvent::Subscribed { peer_id, topic } => {
                //println!("subscribed by peer {:?} topic {:?}", peer_id, topic);
            }
            FloodsubEvent::Unsubscribed { peer_id, topic } => {
                //println!("unsubscribed by peer {:?} topic {:?}", peer_id, topic);
            }
        }
    }
}

pub struct SimpleSwarm;

impl SimpleSwarm {
    pub fn process(peer_id: PeerId, config: &Configuration) -> Result<(), Box<dyn Error>> {
        let transport = libp2p::build_tcp_ws_secio_mplex_yamux(libp2p::identity::Keypair::Ed25519(
            config.node.keypair.clone(),
        ))
        .unwrap();

        let mut swarm = {
            let mut behaviour = P2PBehaviour::new(peer_id.clone());
            behaviour.subscribe(String::from(TransactionCreate::TOPIC));
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
                    Poll::Ready(Some(line)) => swarm.floodsub.publish(
                        &floodsub::TopicBuilder::new(TransactionCreate::TOPIC).build(),
                        line.as_bytes(),
                    ),
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

fn main() {
    let config: &Configuration = &configreader::GLOBAL_CONFIG;
    let peer_id = PeerId::from_public_key(config.node.public.clone());
    println!("peer id = {:?}", peer_id);
    SimpleSwarm::process(peer_id, config);
    // let transport = libp2p::build_development_transport(libp2p::identity::Keypair::Ed25519(
}
