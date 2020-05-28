use super::constants;
use super::message_traits::Message;
use libp2p::floodsub::{Topic, TopicBuilder};
use schema::block::SignedBlock;
use schema::signed_transaction::SignedTransaction;
use utils::serializer::{Deserialize, Serialize};

pub const NODE_MSG_TOPIC_STR: &'static [&'static str] = &["SignedTransaction", "SignedBlock"];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeMessageTypes {
    SignedTransactionEnum(SignedTransaction),
    SignedBlockEnum(SignedBlock),
}

impl Message for SignedTransaction {
    const TOPIC: &'static str = NODE_MSG_TOPIC_STR[0];
    const MODULE_TOPIC: &'static str = constants::NODE;
    fn handler(&self) {
        // info!("i am SignedTransaction handler");
    }
}

impl Message for SignedBlock {
    const TOPIC: &'static str = NODE_MSG_TOPIC_STR[1];
    const MODULE_TOPIC: &'static str = constants::NODE;
    fn handler(&self) {
        info!("i am SignedBlock handler");
    }
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
