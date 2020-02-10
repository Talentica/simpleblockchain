use super::constants;
use futures::channel::mpsc::Sender;
use libp2p::floodsub::{self, protocol, Topic, TopicBuilder, TopicHash};
use utils::serializer::{Deserialize, Serialize, deserialize, serialize};
pub use schema::transaction::{SignedTransaction, Txn};
pub use schema::block::{SignedBlock, SignedBlockTraits};

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
const CONSENSUS_MSG_TOPIC_STR: &'static [&'static str] = &["leader-elect", "block-vote"];

#[derive(Debug, Serialize, Deserialize)]
pub struct TransactionCreate {
    pub nonce: i64,
    pub signature: String,
    pub payload: String,
}

impl Message for TransactionCreate {
    const TOPIC: &'static str = CONSENSUS_MSG_TOPIC_STR[0];
    const MODULE_TOPIC: &'static str = constants::NODE;
    fn handler(&self) {
        println!("i am txn create");
    }
}

impl Message for SignedTransaction{
    const TOPIC: &'static str = NODE_MSG_TOPIC_STR[0];
    const MODULE_TOPIC: &'static str = constants::NODE;
    fn handler(&self) {
        println!("i am txn create");
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BlockCreate {
    pub height: i64,
    pub hash: String,
}

impl Message for BlockCreate {
    const TOPIC: &'static str = CONSENSUS_MSG_TOPIC_STR[0];
    const MODULE_TOPIC: &'static str = constants::NODE;
    fn handler(&self) {
        println!("i am blockcreate");
    }
}

impl Message for SignedBlock {
    const TOPIC: &'static str = NODE_MSG_TOPIC_STR[1];
    const MODULE_TOPIC: &'static str = constants::NODE;
    fn handler(&self) {
        println!("i am blockcreate");
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum NodeMessageTypes {
    SignedTransactionEnum(SignedTransaction),
    SignedBlockEnum(SignedBlock),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ConsensusMessageTypes {
    LeaderElect(TransactionCreate),
    BlockVote(BlockCreate),
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
            SignedTransactionEnum => TopicBuilder::new(SignedTransaction::TOPIC).build(),
            SignedBlockEnum => TopicBuilder::new(SignedBlock::TOPIC).build(),
        }
    }
}

//TODO : Try using macro to implement this for all variations
impl From<ConsensusMessageTypes> for Topic {
    fn from(msg: ConsensusMessageTypes) -> Topic {
        match msg {
            LeaderElect => TopicBuilder::new(TransactionCreate::TOPIC).build(),
            BlockVote => TopicBuilder::new(BlockCreate::TOPIC).build(),
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

impl MsgProcess for protocol::FloodsubMessage {
    fn process(&self, topics: &Vec<TopicHash>, data: &Vec<u8>) {
        if topics[0] == TopicHash::from_raw(String::from(constants::NODE)) {
            println!("Node type msg");
            if topics[1] == TopicBuilder::new(SignedBlock::TOPIC).build().hash().clone() {
                let block_create_msg = deserialize::<SignedBlock>(data);
                println!(
                    "block create msg received in process = {:?}",
                    block_create_msg
                );
                MSG_DISPATCHER
                    .node_msg_dispatcher
                    .as_ref()
                    .unwrap()
                    .clone()
                    .try_send(NodeMessageTypes::SignedBlockEnum(block_create_msg));
            } else if topics[1]
                == TopicBuilder::new(SignedTransaction::TOPIC)
                    .build()
                    .hash()
                    .clone()
            {
                let mut txn_create_msg = deserialize::<SignedTransaction>(data);
                println!("txn create msg received in process = {:?}", txn_create_msg);
                MSG_DISPATCHER
                    .node_msg_dispatcher
                    .as_ref()
                    .unwrap()
                    .clone()
                    .try_send(NodeMessageTypes::SignedTransactionEnum(txn_create_msg));
            }
        } else if topics[0] == TopicHash::from_raw(String::from(constants::CONSENSUS)) {
            println!("Consensus type msg");
        }
    }
}

#[derive(Debug, Clone)]
pub struct MessageDispatcher {
    pub node_msg_dispatcher: Option<Sender<NodeMessageTypes>>,
    pub consensus_msg_dispatcher: Option<Sender<ConsensusMessageTypes>>,
}

impl MessageDispatcher {
    pub fn new() -> Self {
        MessageDispatcher {
            node_msg_dispatcher: None,
            consensus_msg_dispatcher: None,
        }
    }
}

lazy_static! {
    pub static ref MSG_DISPATCHER: MessageDispatcher = MessageDispatcher::new();
}