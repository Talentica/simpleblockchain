#[cfg(test)]
mod test_message_handler {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use crate::constants;
    use crate::message_sender::MessageSender;
    use crate::messages::{MessageTypes, MSG_DISPATCHER};
    use crate::node_messages::NodeMessageTypes;
    use futures::channel::mpsc::*;
    use libp2p::floodsub::{Topic, TopicBuilder};
    use schema::block::{Block, BlockTraits, SignedBlock};
    use std::{thread, time::Duration};
    use utils::serializer::{deserialize, serialize};
    // fn to test state operations

    #[test]
    fn test_node_message_operations() {
        let (tx1, mut rx1) = channel::<Option<MessageTypes>>(4194304);
        let mut sender = tx1.clone();
        let peer_id: String = String::from("mock_peer_id");
        let genesis_block: Block = Block::genesis_block(peer_id);
        let block: SignedBlock = SignedBlock::create_block(genesis_block, vec![0], Vec::new());
        // let data = MessageTypes::NodeMsg(NodeMessageTypes::SignedBlockEnum(block));
        MessageSender::send_block_msg(&mut sender, block.clone());
        thread::sleep(Duration::from_millis(1000));
        let received_data = rx1.try_next().unwrap();
        match received_data {
            Some(some_msg1) => {
                match some_msg1 {
                    Some(msgtype) => {
                        match msgtype {
                            MessageTypes::NodeMsg(data) => {
                                info!("NodeMsg received {:?}", data);
                                let topics: Vec<Topic> =
                                    Vec::<Topic>::from(MessageTypes::NodeMsg(data.clone()));
                                assert_eq!(
                                    topics[0].hash().clone(),
                                    TopicBuilder::new(constants::NODE).build().hash().clone()
                                );
                                let result = MSG_DISPATCHER
                                    .node_msg_dispatcher
                                    .clone()
                                    .try_send(Some(data));
                                if result.is_err() {
                                    result.unwrap_err().into_send_error();
                                }
                            }
                            MessageTypes::ConsensusMsg(data) => {
                                info!("ConsensusMsg received {:?}", data);
                                let topics: Vec<Topic> =
                                    Vec::<Topic>::from(MessageTypes::ConsensusMsg(data.clone()));
                                assert_eq!(
                                    topics[0].hash().clone(),
                                    TopicBuilder::new(constants::CONSENSUS)
                                        .build()
                                        .hash()
                                        .clone()
                                );
                                let result = MSG_DISPATCHER
                                    .consensus_msg_dispatcher
                                    .clone()
                                    .try_send(Some(data));
                                if result.is_err() {
                                    result.unwrap_err().into_send_error();
                                }
                            }
                        };
                    }
                    None => panic!("test case failed due to data failure at receiver end"),
                };
            }
            None => panic!("test case failed due to data failure at receiver end"),
        };
        thread::sleep(Duration::from_millis(1000));
        let dispatched_data = MSG_DISPATCHER
            .node_msg_receiver
            .lock()
            .unwrap()
            .try_next()
            .unwrap();
        let dispatched_data = dispatched_data.unwrap().unwrap();
        match dispatched_data {
            NodeMessageTypes::SignedBlockEnum(value) => assert_eq!(value, block),
            NodeMessageTypes::SignedTransactionEnum(_) => panic!("wrong data"),
        };
    }

    #[test]
    fn test_consensus_message_operations() {
        let (tx1, mut rx1) = channel::<Option<MessageTypes>>(4194304);
        let mut sender = tx1.clone();
        let peer_id: String = String::from("mock_peer_id");
        // let data = MessageTypes::NodeMsg(NodeMessageTypes::SignedBlockEnum(block));
        let data = Some(MessageTypes::ConsensusMsg(serialize(&peer_id).unwrap()));
        let error: Result<(), TrySendError<Option<MessageTypes>>> = sender.try_send(data);
        if error.is_err() {
            error!("{:?}", error);
        }
        thread::sleep(Duration::from_millis(1000));
        let received_data = rx1.try_next().unwrap();
        match received_data {
            Some(some_msg1) => {
                match some_msg1 {
                    Some(msgtype) => {
                        match msgtype {
                            MessageTypes::NodeMsg(data) => {
                                info!("NodeMsg received {:?}", data);
                                let topics: Vec<Topic> =
                                    Vec::<Topic>::from(MessageTypes::NodeMsg(data.clone()));
                                assert_eq!(
                                    topics[0].hash().clone(),
                                    TopicBuilder::new(constants::NODE).build().hash().clone()
                                );
                                let result = MSG_DISPATCHER
                                    .node_msg_dispatcher
                                    .clone()
                                    .try_send(Some(data));
                                if result.is_err() {
                                    result.unwrap_err().into_send_error();
                                }
                            }
                            MessageTypes::ConsensusMsg(data) => {
                                info!("ConsensusMsg received {:?}", data);
                                let topics: Vec<Topic> =
                                    Vec::<Topic>::from(MessageTypes::ConsensusMsg(data.clone()));
                                assert_eq!(
                                    topics[0].hash().clone(),
                                    TopicBuilder::new(constants::CONSENSUS)
                                        .build()
                                        .hash()
                                        .clone()
                                );
                                let result = MSG_DISPATCHER
                                    .consensus_msg_dispatcher
                                    .clone()
                                    .try_send(Some(data));
                                if result.is_err() {
                                    result.unwrap_err().into_send_error();
                                }
                            }
                        };
                    }
                    None => panic!("test case failed due to data failure at receiver end"),
                };
            }
            None => panic!("test case failed due to data failure at receiver end"),
        };
        thread::sleep(Duration::from_millis(1000));
        let dispatched_data = MSG_DISPATCHER
            .consensus_msg_receiver
            .lock()
            .unwrap()
            .try_next()
            .unwrap();
        let dispatched_data = deserialize::<String>(&dispatched_data.unwrap().unwrap()).unwrap();
        assert_eq!(dispatched_data, peer_id);
    }
}
