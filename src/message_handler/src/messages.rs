use super::constants;
use super::node_messages::NodeMessageTypes;
use futures::{channel::mpsc::channel, channel::mpsc::Receiver, channel::mpsc::Sender};
use libp2p::floodsub::{protocol, Topic};
use std::sync::{Arc, Mutex};
use utils::serializer::{deserialize, Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageTypes {
    NodeMsg(NodeMessageTypes),
    ConsensusMsg(Vec<u8>),
}

//TODO : Try using macro to implement this for all variations
impl From<MessageTypes> for Vec<Topic> {
    fn from(msg: MessageTypes) -> Vec<Topic> {
        let mut ret: Vec<Topic> = Vec::new();
        match msg {
            MessageTypes::NodeMsg(_data) => {
                ret.push(Topic::new(constants::NODE));
            }
            MessageTypes::ConsensusMsg(_data) => {
                ret.push(Topic::new(constants::CONSENSUS));
            }
        }
        ret
    }
}

///Process FloodSubMessages
///
pub trait MsgProcess {
    fn process(&self, topics: &Vec<Topic>, data: &Vec<u8>);
}

impl MsgProcess for protocol::FloodsubMessage {
    fn process(&self, topics: &Vec<Topic>, data: &Vec<u8>) {
        if topics[0] == Topic::new(constants::NODE) {
            debug!("NodeMessageTypes data received");
            if let Ok(deserialize_msg) = deserialize::<NodeMessageTypes>(data) {
                let result = MSG_DISPATCHER
                    .node_msg_dispatcher
                    .clone()
                    .try_send(Some(deserialize_msg));
                if result.is_err() {
                    result.unwrap_err().into_send_error();
                }
            }
        } else if topics[0] == Topic::new(constants::CONSENSUS) {
            debug!("ConsensusMessageTypes data received");
            if let Ok(deserialize_msg) = deserialize::<Vec<u8>>(data) {
                let result = MSG_DISPATCHER
                    .consensus_msg_dispatcher
                    .clone()
                    .try_send(Some(deserialize_msg));
                if result.is_err() {
                    result.unwrap_err().into_send_error();
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct MessageDispatcher {
    pub node_msg_dispatcher: Sender<Option<NodeMessageTypes>>,
    pub node_msg_receiver: Arc<Mutex<Receiver<Option<NodeMessageTypes>>>>,
    pub consensus_msg_dispatcher: Sender<Option<Vec<u8>>>,
    pub consensus_msg_receiver: Arc<Mutex<Receiver<Option<Vec<u8>>>>>,
}

impl MessageDispatcher {
    pub fn new() -> Self {
        let (tx, rx) = channel::<Option<NodeMessageTypes>>(1024);
        let (tx_consensus, rx_consensus) = channel::<Option<Vec<u8>>>(1024);
        MessageDispatcher {
            node_msg_dispatcher: tx,
            node_msg_receiver: Arc::new(Mutex::new(rx)),
            consensus_msg_dispatcher: tx_consensus,
            consensus_msg_receiver: Arc::new(Mutex::new(rx_consensus)),
        }
    }
    pub fn set_node_msg_dispatcher(&mut self, tx: &Sender<Option<NodeMessageTypes>>) {
        self.node_msg_dispatcher = tx.clone();
    }

    pub fn set_consensus_msg_dispatcher(&mut self, tx: &Sender<Option<Vec<u8>>>) {
        self.consensus_msg_dispatcher = tx.clone();
    }
}

lazy_static! {
    pub static ref MSG_DISPATCHER: MessageDispatcher = MessageDispatcher::new();
}
