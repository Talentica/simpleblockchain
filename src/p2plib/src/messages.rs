use super::constants;
use futures::{channel::mpsc::channel, channel::mpsc::Receiver, channel::mpsc::Sender};
use libp2p::floodsub::{self, protocol, Topic, TopicBuilder, TopicHash};
pub use schema::block::{SignedBlock, SignedBlockTraits};
pub use schema::transaction::{SignedTransaction, Txn, ObjectHash};
use std::sync::{Arc, Mutex};
use utils::serializer::{deserialize, serialize, Deserialize, Serialize};

pub trait Message {
    const TOPIC: &'static str;
    const MODULE_TOPIC: &'static str;

    fn handler(&self);

    fn topic(&self) -> String {
        String::from(Self::TOPIC)
    }

    fn module_topic(&self) -> String {
        String::from(Self::MODULE_TOPIC)
    }

    fn get_topics(&self) -> Vec<String> {
        let mut ret: Vec<String> = Vec::new();
        ret.push(self.module_topic());
        ret.push(self.topic());
        ret
    }
}

const NODE_MSG_TOPIC_STR: &'static [&'static str] = &["txn-create", "block-create"];
const CONSENSUS_MSG_TOPIC_STR: &'static [&'static str] = &["leader-elect", "block-create"];

#[derive(Debug, Serialize, Deserialize)]
pub struct TransactionCreate {
    pub nonce: i64,
    pub signature: String,
    pub payload: String,
}

impl Message for TransactionCreate {
    const TOPIC: &'static str = CONSENSUS_MSG_TOPIC_STR[0];
    const MODULE_TOPIC: &'static str = constants::CONSENSUS;
    fn handler(&self) {
        println!("i am txn create jabra fan of Consensus Module<>");
    }
}

impl Message for SignedTransaction {
    const TOPIC: &'static str = NODE_MSG_TOPIC_STR[0];
    const MODULE_TOPIC: &'static str = constants::NODE;
    fn handler(&self) {
        println!("i am SignedTransaction create jabra fan of Node Module");
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BlockCreate {
    pub height: i64,
    pub hash: String,
}

impl Message for BlockCreate {
    const TOPIC: &'static str = NODE_MSG_TOPIC_STR[1];
    const MODULE_TOPIC: &'static str = constants::NODE;
    fn handler(&self) {
        println!("i am block create jabra fan of Consensus Module<> ");
    }
}

impl Message for SignedBlock {
    const TOPIC: &'static str = CONSENSUS_MSG_TOPIC_STR[1];
    const MODULE_TOPIC: &'static str = constants::CONSENSUS;
    fn handler(&self) {
        println!("i am SignedBlock create jabra fan of Node Module");
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum NodeMessageTypes {
    SignedTransactionEnum(SignedTransaction),
    SignedBlockEnum(BlockCreate),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ConsensusMessageTypes {
    LeaderElect(TransactionCreate),
    BlockVote(SignedBlock),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum MessageTypes {
    NodeMsg(NodeMessageTypes),
    ConsensusMsg(ConsensusMessageTypes),
}

//TODO : Try using macro to implement this for all variations
impl From<NodeMessageTypes> for Topic {
    fn from(msg: NodeMessageTypes) -> Topic {
        match msg {
            NodeMessageTypes::SignedBlockEnum(data) => TopicBuilder::new(data.topic()).build(),
            NodeMessageTypes::SignedTransactionEnum(data) => {
                TopicBuilder::new(data.topic()).build()
            }
        }
    }
}

//TODO : Try using macro to implement this for all variations
impl From<ConsensusMessageTypes> for Topic {
    fn from(msg: ConsensusMessageTypes) -> Topic {
        match msg {
            ConsensusMessageTypes::LeaderElect(data) => TopicBuilder::new(data.topic()).build(),
            ConsensusMessageTypes::BlockVote(data) => TopicBuilder::new(data.topic()).build(),
        }
    }
}

//TODO : Try using macro to implement this for all variations
impl From<MessageTypes> for Vec<Topic> {
    fn from(msg: MessageTypes) -> Vec<Topic> {
        let mut ret: Vec<Topic> = Vec::new();
        match msg {
            MessageTypes::NodeMsg(data) => {
                ret.push(TopicBuilder::new(constants::NODE).build());
                ret.push(Topic::from(data));
            }
            MessageTypes::ConsensusMsg(data) => {
                ret.push(TopicBuilder::new(constants::CONSENSUS).build());
                ret.push(Topic::from(data));
            }
        }
        ret
    }
}

///Process FloodSubMessages
///
pub trait MsgProcess {
    fn process(&self, topics: &Vec<TopicHash>, data: &Vec<u8>);
}

pub trait NodeMsgProcess {
    fn process_node_msg(&self, topic: &TopicHash, data: &Vec<u8>);
}

impl MsgProcess for protocol::FloodsubMessage {
    fn process(&self, topics: &Vec<TopicHash>, data: &Vec<u8>) {
        if topics[0] == TopicBuilder::new(constants::NODE).build().hash().clone() {
            println!("Node type msg");
            let block_create_msg = deserialize::<NodeMessageTypes>(data);
            println!(
                "block create msg received in process = {:?}",
                block_create_msg
            );
            MSG_DISPATCHER
                .node_msg_dispatcher
                .clone()
                .try_send(Some(block_create_msg));
        } else if topics[0]
            == TopicBuilder::new(constants::CONSENSUS)
                .build()
                .hash()
                .clone()
        {
            println!("Consensus type msg");
            let block_create_msg = deserialize::<ConsensusMessageTypes>(data);
            println!(
                "block create msg received in process = {:?}",
                block_create_msg
            );
            MSG_DISPATCHER
                .consensus_msg_dispatcher
                .clone()
                .try_send(Some(block_create_msg));
        }
    }
}

#[derive(Debug, Clone)]
pub struct MessageDispatcher {
    pub node_msg_dispatcher: Sender<Option<NodeMessageTypes>>,
    pub node_msg_receiver: Arc<Mutex<Receiver<Option<NodeMessageTypes>>>>,
    pub consensus_msg_dispatcher: Sender<Option<ConsensusMessageTypes>>,
    pub consensus_msg_receiver: Arc<Mutex<Receiver<Option<ConsensusMessageTypes>>>>,
}

impl MessageDispatcher {
    pub fn new() -> Self {
        let (mut tx, mut rx) = channel::<Option<NodeMessageTypes>>(1024);
        let (mut tx_consensus, mut rx_consensus) = channel::<Option<ConsensusMessageTypes>>(1024);
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

    pub fn set_consensus_msg_dispatcher(&mut self, tx: &Sender<Option<ConsensusMessageTypes>>) {
        self.consensus_msg_dispatcher = tx.clone();
    }
}

lazy_static! {
    pub static ref MSG_DISPATCHER: MessageDispatcher = MessageDispatcher::new();
}
