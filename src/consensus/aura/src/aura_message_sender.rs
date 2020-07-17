use crate::aura_messages::{AuraMessageTypes, AuthorBlock, BlockAcceptance, RoundOwner};
use futures::channel::mpsc::*;
use message_handler::messages::MessageTypes;
use utils::serializer::serialize;

pub struct AuraMessageSender {}

impl AuraMessageSender {
    pub fn send_round_owner_msg(sender: &mut Sender<Option<MessageTypes>>, msg: RoundOwner) {
        if let Ok(serialize_msg) = serialize(&AuraMessageTypes::RoundOwnerEnum(msg)) {
            let data = Some(MessageTypes::ConsensusMsg(serialize_msg));
            let error: Result<(), TrySendError<Option<MessageTypes>>> = sender.try_send(data);
            if error.is_err() {
                error!("{:?}", error);
            } else {
                info!("msg send send_round_owner_msg");
            }
        }
    }

    pub fn send_block_acceptance_msg(
        sender: &mut Sender<Option<MessageTypes>>,
        msg: BlockAcceptance,
    ) {
        if let Ok(serialize_msg) = serialize(&AuraMessageTypes::BlockAcceptanceEnum(msg)) {
            let data = Some(MessageTypes::ConsensusMsg(serialize_msg));
            let error: Result<(), TrySendError<Option<MessageTypes>>> = sender.try_send(data);
            if error.is_err() {
                error!("{:?}", error);
            } else {
                info!("msg send send_block_acceptance_msg");
            }
        }
    }

    pub fn send_author_block_msg(sender: &mut Sender<Option<MessageTypes>>, msg: AuthorBlock) {
        if let Ok(serialize_msg) = serialize(&AuraMessageTypes::AuthorBlockEnum(msg)) {
            let data = Some(MessageTypes::ConsensusMsg(serialize_msg));
            let error: Result<(), TrySendError<Option<MessageTypes>>> = sender.try_send(data);
            if error.is_err() {
                error!("{:?}", error);
            } else {
                info!("msg send send_author_block_msg");
            }
        }
    }
}
