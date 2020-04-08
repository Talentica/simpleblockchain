use crate::messages::{
    ConsensusMessageTypes, ElectionPing, ElectionPong, MessageTypes, NodeMessageTypes,
    SignedLeaderElection,
};
use futures::channel::mpsc::*;
use schema::{block::SignedBlock, transaction::SignedTransaction};

pub struct MessageSender {}

impl MessageSender {
    pub fn send_election_ping_msg(sender: &mut Sender<Option<MessageTypes>>, msg: ElectionPing) {
        let data = Some(MessageTypes::ConsensusMsg(
            ConsensusMessageTypes::ConsensusPing(msg),
        ));
        let error: Result<(), TrySendError<Option<MessageTypes>>> = sender.try_send(data);
        if error.is_err() {
            error!("{:?}", error);
        }
    }

    pub fn send_election_pong_msg(sender: &mut Sender<Option<MessageTypes>>, msg: ElectionPong) {
        let data = Some(MessageTypes::ConsensusMsg(
            ConsensusMessageTypes::ConsensusPong(msg),
        ));
        let error: Result<(), TrySendError<Option<MessageTypes>>> = sender.try_send(data);
        if error.is_err() {
            error!("{:?}", error);
        }
    }

    pub fn send_block_msg(sender: &mut Sender<Option<MessageTypes>>, msg: SignedBlock) {
        let data = Some(MessageTypes::NodeMsg(NodeMessageTypes::SignedBlockEnum(
            msg,
        )));
        let error: Result<(), TrySendError<Option<MessageTypes>>> = sender.try_send(data);
        if error.is_err() {
            error!("{:?}", error);
        }
    }

    pub fn send_leader_election_msg(
        sender: &mut Sender<Option<MessageTypes>>,
        msg: SignedLeaderElection,
    ) {
        let data = Some(MessageTypes::ConsensusMsg(
            ConsensusMessageTypes::LeaderElect(msg),
        ));
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
