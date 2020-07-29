use crate::consensus_messages::{
    ConsensusMessageTypes, ElectionPing, ElectionPong, SignedLeaderElection,
};
use futures::channel::mpsc::*;
use message_handler::messages::MessageTypes;
use utils::serializer::serialize;

pub struct ConsensusMessageSender {}

impl ConsensusMessageSender {
    pub fn send_election_ping_msg(sender: &mut Sender<Option<MessageTypes>>, msg: ElectionPing) {
        if let Ok(serialize_msg) = serialize(&ConsensusMessageTypes::ConsensusPing(msg)) {
            let data = Some(MessageTypes::ConsensusMsg(serialize_msg));
            let error: Result<(), TrySendError<Option<MessageTypes>>> = sender.try_send(data);
            if error.is_err() {
                error!("{:?}", error);
            }
        }
    }

    pub fn send_election_pong_msg(sender: &mut Sender<Option<MessageTypes>>, msg: ElectionPong) {
        if let Ok(serialize_msg) = serialize(&ConsensusMessageTypes::ConsensusPong(msg)) {
            let data = Some(MessageTypes::ConsensusMsg(serialize_msg));
            let error: Result<(), TrySendError<Option<MessageTypes>>> = sender.try_send(data);
            if error.is_err() {
                error!("{:?}", error);
            }
        }
    }

    pub fn send_leader_election_msg(
        sender: &mut Sender<Option<MessageTypes>>,
        msg: SignedLeaderElection,
    ) {
        if let Ok(serialize_msg) = serialize(&ConsensusMessageTypes::LeaderElect(msg)) {
            let data = Some(MessageTypes::ConsensusMsg(serialize_msg));
            let error: Result<(), TrySendError<Option<MessageTypes>>> = sender.try_send(data);
            if error.is_err() {
                error!("{:?}", error);
            }
        }
    }
}
