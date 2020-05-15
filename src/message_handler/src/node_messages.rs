use libp2p::floodsub::{Topic, TopicBuilder};
use schema::block::SignedBlock;
use schema::signed_transaction::SignedTransaction;
use sdk::message_traits::Message;
use utils::serializer::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum NodeMessageTypes {
    SignedTransactionEnum(SignedTransaction),
    SignedBlockEnum(SignedBlock),
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
