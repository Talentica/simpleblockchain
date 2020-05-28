use crate::messages::MessageTypes;
use crate::node_messages::NodeMessageTypes;
use futures::channel::mpsc::*;
use schema::{block::SignedBlock, signed_transaction::SignedTransaction};

pub struct MessageSender {}

impl MessageSender {
    pub fn send_block_msg(sender: &mut Sender<Option<MessageTypes>>, msg: SignedBlock) {
        let data = Some(MessageTypes::NodeMsg(NodeMessageTypes::SignedBlockEnum(
            msg,
        )));
        let error: Result<(), TrySendError<Option<MessageTypes>>> = sender.try_send(data);
        if error.is_err() {
            error!("{:?}", error);
        }
    }

    pub fn send_transaction_msg(sender: &mut Sender<Option<MessageTypes>>, msg: SignedTransaction) {
        let data = Some(MessageTypes::NodeMsg(
            NodeMessageTypes::SignedTransactionEnum(msg),
        ));
        let error: Result<(), TrySendError<Option<MessageTypes>>> = sender.try_send(data);
        if error.is_err() {
            error!("{:?}", error);
        }
    }
}
