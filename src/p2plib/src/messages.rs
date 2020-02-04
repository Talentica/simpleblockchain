use super::constants;
use libp2p::floodsub::{self, Topic, TopicBuilder};
use serde::{Deserialize, Serialize};

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

#[derive(Debug, Serialize, Deserialize)]
pub struct TransactionCreate {
    pub nonce: i64,
    pub signature: String,
    pub payload: String,
}

impl Message for TransactionCreate {
    const TOPIC: &'static str = "txn-create";
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
    const TOPIC: &'static str = "block-create";
    const MODULE_TOPIC: &'static str = constants::NODE;
    fn handler(&self) {
        println!("i am blockcreate");
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum NodeMessageTypes {
    TransactionCreate(TransactionCreate),
    BlockCreate(BlockCreate),
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
            TransactionCreate => TopicBuilder::new(TransactionCreate::TOPIC).build(),
            BlockCreate => TopicBuilder::new(BlockCreate::TOPIC).build(),
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
