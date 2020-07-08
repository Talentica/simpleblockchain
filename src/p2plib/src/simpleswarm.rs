use super::p2pbehaviour::P2PBehaviour;
use futures::{channel::mpsc::*, executor::*, future, prelude::*, task::*};
use libp2p::{floodsub::Topic, PeerId, Swarm};
use message_handler::messages::*;
use std::error::Error;
use utils::configreader::Configuration;
use utils::serializer::*;

pub struct SimpleSwarm {
    // behaviour: Option<P2PBehaviour<TSubstream>>,
    pub topic_list: Vec<String>,
    pub tx: Sender<Option<MessageTypes>>,
    pub rx: Receiver<Option<MessageTypes>>,
}

impl SimpleSwarm {
    pub fn new() -> Self {
        let (tx1, rx1) = channel::<Option<MessageTypes>>(4194304);
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
        block_on(future::poll_fn(move |cx: &mut Context| {
            loop {
                match self.rx.poll_next_unpin(cx) {
                    Poll::Ready(Some(msg)) => {
                        match msg {
                            None => info!("empty message !"),
                            Some(msgtype) => match msgtype {
                                MessageTypes::NodeMsg(data) => {
                                    info!("NodeMsg received, publishing to p2p network");
                                    if let Ok(value) = serialize(&data) {
                                        let topics: Vec<Topic> =
                                            Vec::<Topic>::from(MessageTypes::NodeMsg(data)); //TODO Find way to get rid of clone
                                        swarm.floodsub.publish_many(topics, value);
                                    } else {
                                        warn!("NodeMsg can't able to publish to the network");
                                    }
                                }
                                MessageTypes::ConsensusMsg(data) => {
                                    info!("ConsensusMsg received, publishing to p2p network");
                                    if let Ok(value) = serialize(&data) {
                                        let topics: Vec<Topic> =
                                            Vec::<Topic>::from(MessageTypes::ConsensusMsg(data));
                                        swarm.floodsub.publish_many(topics, value);
                                    } else {
                                        warn!("ConsensusMsg can't able to publish to the network");
                                    }
                                }
                            },
                        }
                    }
                    Poll::Ready(None) => {
                        info!("channel closed !");
                        return Poll::Ready(Ok(()));
                        // Poll::Ready(());
                    }
                    Poll::Pending => break,
                }
            }

            loop {
                match swarm.poll_next_unpin(cx) {
                    Poll::Ready(Some(event)) => info!("{:?}", event),
                    Poll::Ready(None) => return Poll::Ready(Ok(())),
                    Poll::Pending => {
                        if !listening {
                            if let Some(a) = Swarm::listeners(&swarm).next() {
                                info!("Listening on {:?}", a);
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
