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

pub enum MessageTypes {
    TransactionCreate,
    BlockCreate,
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
impl<TSubstream: AsyncRead + AsyncWrite> NetworkBehaviourEventProcess<MdnsEvent>
    for P2PBehaviour<TSubstream>
{
    fn inject_event(&mut self, mdns_event: MdnsEvent) {
        match mdns_event {
            MdnsEvent::Discovered(discovered_nodes) => {
                println!("Discovered address {:?}", discovered_nodes);
                for (peer_id, _) in discovered_nodes {
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
                println!("subscribed by peer {:?} topic {:?}", peer_id, topic);
            }
            FloodsubEvent::Unsubscribed { peer_id, topic } => {
                println!("unsubscribed by peer {:?} topic {:?}", peer_id, topic);
            }
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let config: &Configuration = &configreader::GLOBAL_CONFIG;
    let peer_id = PeerId::from_public_key(config.node.public.clone());
    println!("peer id = {:?}", peer_id);
    let transport = libp2p::build_development_transport(libp2p::identity::Keypair::Ed25519(
        // let transport = libp2p::build_tcp_ws_secio_mplex_yamux(libp2p::identity::Keypair::Ed25519(
        config.node.keypair.clone(),
    ))
    .unwrap();
    // Create a Floodsub topic
    let floodsub_topic = floodsub::TopicBuilder::new("txn-messages").build();

    let mut swarm = {
        let mdns = task::block_on(Mdns::new()).unwrap();
        let behaviour = P2PBehaviour {
            floodsub: Floodsub::new(peer_id.clone()),
            mdns,
        };
        Swarm::new(transport, behaviour, peer_id)
    };

    let myport = std::env::args().nth(1).unwrap();
    let port1 = std::env::args().nth(2).unwrap();
    //let port2 = std::env::args().nth(3).unwrap();

    Swarm::listen_on(
        &mut swarm,
        format!("{}{}", "/ip4/0.0.0.0/tcp/", myport)
            .parse()
            .unwrap(),
    )
    .unwrap();

    //connect to all peers
    let mut peers_list: Vec<String> = Vec::new(); //config.node.peers;
    peers_list.push(String::from(format!("{}{}", "/ip4/0.0.0.0/tcp/", port1)));
    //peers_list.push(String::from(format!("{}{}", "/ip4/0.0.0.0/tcp/", port2)));
    // peers_list.push(String::from("/ip4/0.0.0.0/tcp/4446"));
    for peer in peers_list {
        let multi_addr = peer.parse::<Multiaddr>().unwrap();
        match libp2p::Swarm::dial_addr(&mut swarm, multi_addr.clone()) {
            Ok(_) => println!("Dialed {:?}", peer),
            Err(e) => println!("Dial {:?} failed: {:?}", multi_addr, e),
        }
    }

    // Read full lines from stdin
    let mut stdin = io::BufReader::new(io::stdin()).lines();

    let mut listening = false;
    task::block_on(future::poll_fn(move |cx: &mut Context| {
        loop {
            match stdin.try_poll_next_unpin(cx)? {
                Poll::Ready(Some(line)) => swarm.floodsub.publish(&floodsub_topic, line.as_bytes()),
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
