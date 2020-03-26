use crate::messages::{
    BlockConsensus, ConsensusMessageTypes, MessageTypes, NodeMessageTypes, SignedLeaderElection,
};
use futures::channel::mpsc::*;
use schema::{block::SignedBlock, transaction::SignedTransaction};

pub struct MessageSender {}

impl MessageSender {
    pub fn send_block_consensus_msg(
        sender: &mut Sender<Option<MessageTypes>>,
        msg: BlockConsensus,
    ) {
        let data = Some(MessageTypes::ConsensusMsg(
            ConsensusMessageTypes::BlockVote(msg),
        ));
        let error: Result<(), TrySendError<Option<MessageTypes>>> = sender.try_send(data);
        if error.is_err() {
            eprintln!("{:?}", error);
        }
    }

    pub fn send_block_msg(sender: &mut Sender<Option<MessageTypes>>, msg: SignedBlock) {
        let data = Some(MessageTypes::NodeMsg(NodeMessageTypes::SignedBlockEnum(
            msg,
        )));
        let error: Result<(), TrySendError<Option<MessageTypes>>> = sender.try_send(data);
        if error.is_err() {
            eprintln!("{:?}", error);
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
            eprintln!("{:?}", error);
        }
    }

    pub fn send_transaction_msg(sender: &mut Sender<Option<MessageTypes>>, msg: SignedTransaction) {
        let data = Some(MessageTypes::NodeMsg(
            NodeMessageTypes::SignedTransactionEnum(msg),
        ));
        let error: Result<(), TrySendError<Option<MessageTypes>>> = sender.try_send(data);
        if error.is_err() {
            eprintln!("{:?}", error);
        }
    }
}
