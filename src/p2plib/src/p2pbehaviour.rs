use super::messages::*;
use futures::prelude::*;
use libp2p::{
    floodsub::{self, Floodsub, FloodsubEvent},
    mdns::{Mdns, MdnsEvent},
    swarm::NetworkBehaviourEventProcess,
    NetworkBehaviour, PeerId,
};

/// Network behavior defined combining, floodsub and mdns (for discovery)
///
#[derive(NetworkBehaviour)]
pub struct P2PBehaviour<TSubstream: AsyncRead + AsyncWrite> {
    pub floodsub: Floodsub<TSubstream>,
    pub mdns: Mdns<TSubstream>,
}

impl<TSubstream: AsyncRead + AsyncWrite> P2PBehaviour<TSubstream> {
    pub fn new(peer_id: PeerId) -> Self {
        let mdns = Mdns::new().unwrap();
        let behaviour = P2PBehaviour {
            floodsub: Floodsub::new(peer_id.clone()),
            mdns,
        };
        behaviour
    }
    pub fn subscribe(&mut self, topic_str: &String) {
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
                println!(
                    "Message received from {:?}, msg topic {:?}",
                    msg.source, msg.topics
                );
                msg.process(&msg.topics, &msg.data);
            }
            FloodsubEvent::Subscribed {
                peer_id: _,
                topic: _,
            } => {
                //println!("subscribed by peer {:?} topic {:?}", peer_id, topic);
            }
            FloodsubEvent::Unsubscribed {
                peer_id: _,
                topic: _,
            } => {
                //println!("unsubscribed by peer {:?} topic {:?}", peer_id, topic);
            }
        }
    }
}
