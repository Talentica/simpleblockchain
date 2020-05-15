use crate::messages::MessageTypes;
use crate::node_messages::NodeMessageTypes;
use futures::channel::mpsc::*;
use schema::{block::SignedBlock, signed_transaction::SignedTransaction};
use utils::serializer::serialize;

pub struct MessageSender {}

impl MessageSender {
    pub fn send_block_msg(sender: &mut Sender<Option<MessageTypes>>, msg: SignedBlock) {
        if let Ok(serialize_msg) = serialize(&NodeMessageTypes::SignedBlockEnum(msg)) {
            let data = Some(MessageTypes::NodeMsg(serialize_msg));
            let error: Result<(), TrySendError<Option<MessageTypes>>> = sender.try_send(data);
            if error.is_err() {
                error!("{:?}", error);
            }
        }
    }

    pub fn send_transaction_msg(sender: &mut Sender<Option<MessageTypes>>, msg: SignedTransaction) {
        if let Ok(serialize_msg) = serialize(&NodeMessageTypes::SignedTransactionEnum(msg)) {
            let data = Some(MessageTypes::NodeMsg(serialize_msg));
            let error: Result<(), TrySendError<Option<MessageTypes>>> = sender.try_send(data);
            if error.is_err() {
                error!("{:?}", error);
            }
        }
    }
}
