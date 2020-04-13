use super::constants;
use futures::{channel::mpsc::channel, channel::mpsc::Receiver, channel::mpsc::Sender};
use libp2p::floodsub::{protocol, Topic, TopicBuilder, TopicHash};
use schema::block::SignedBlock;
use schema::signed_transaction::SignedTransaction;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, Mutex};
use utils::keypair::{CryptoKeypair, Keypair, KeypairType, PublicKey, Verify};
use utils::serializer::{deserialize, serialize, Deserialize, Serialize};

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

pub const NODE_MSG_TOPIC_STR: &'static [&'static str] = &["SignedTransaction", "SignedBlock"];
pub const CONSENSUS_MSG_TOPIC_STR: &'static [&'static str] =
    &["LeaderElection", "ElectionPing", "ElectionPong"];

#[derive(Debug, Serialize, Deserialize)]
pub struct LeaderElection {
    pub block_height: u64,
    pub old_leader: String,
    pub new_leader: String,
}

impl Hash for LeaderElection {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.block_height.hash(state);
        self.old_leader.hash(state);
        self.new_leader.hash(state);
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SignedLeaderElection {
    pub leader_payload: LeaderElection,
    pub signature: Vec<u8>,
}

impl Message for SignedLeaderElection {
    const TOPIC: &'static str = CONSENSUS_MSG_TOPIC_STR[0];
    const MODULE_TOPIC: &'static str = constants::CONSENSUS;
    fn handler(&self) {
        info!("i am LeaderElection handler");
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ElectionPingPayload {
    pub height: u64,
    pub public_key: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ElectionPing {
    pub payload: ElectionPingPayload,
    pub signature: Vec<u8>,
}

impl ElectionPing {
    pub fn verify(&self) -> bool {
        let ser_payload = serialize(&self.payload);
        PublicKey::verify_from_encoded_pk(
            &self.payload.public_key,
            &ser_payload,
            &self.signature.as_ref(),
        )
    }

    fn sign(&mut self, kp: &KeypairType) {
        let ser_txn = serialize(&self.payload);
        let sign = Keypair::sign(&kp, &ser_txn);
        self.signature = sign;
    }

    pub fn create(kp: &KeypairType, height: u64) -> ElectionPing {
        let payload: ElectionPingPayload = ElectionPingPayload {
            height,
            public_key: hex::encode(kp.public().encode()),
        };
        let mut election_ping: ElectionPing = ElectionPing {
            payload,
            signature: vec![],
        };
        election_ping.sign(kp);
        election_ping
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ElectionPongPayload {
    pub height: u64,
    pub current_leader: String,
    pub may_be_leader: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ElectionPong {
    pub payload: ElectionPongPayload,
    pub signature: Vec<u8>,
}

impl ElectionPong {
    pub fn verify(&self) -> bool {
        let ser_payload = serialize(&self.payload);
        PublicKey::verify_from_encoded_pk(
            &self.payload.may_be_leader,
            &ser_payload,
            &self.signature.as_ref(),
        )
    }

    fn sign(&mut self, kp: &KeypairType) {
        let ser_txn = serialize(&self.payload);
        let sign = Keypair::sign(&kp, &ser_txn);
        self.signature = sign;
    }

    pub fn create(kp: &KeypairType, ping: &ElectionPing) -> ElectionPong {
        let payload: ElectionPongPayload = ElectionPongPayload {
            height: ping.payload.height.clone(),
            current_leader: ping.payload.public_key.clone(),
            may_be_leader: hex::encode(kp.public().encode()),
        };
        let mut election_pong: ElectionPong = ElectionPong {
            payload,
            signature: vec![],
        };
        election_pong.sign(kp);
        election_pong
    }
}

impl Message for ElectionPing {
    const TOPIC: &'static str = CONSENSUS_MSG_TOPIC_STR[1];
    const MODULE_TOPIC: &'static str = constants::CONSENSUS;
    fn handler(&self) {
        info!("i am ElectionPing handler");
    }
}

impl Message for ElectionPong {
    const TOPIC: &'static str = CONSENSUS_MSG_TOPIC_STR[2];
    const MODULE_TOPIC: &'static str = constants::CONSENSUS;
    fn handler(&self) {
        info!("i am ElectionPing handler");
    }
}

impl Message for SignedTransaction {
    const TOPIC: &'static str = NODE_MSG_TOPIC_STR[0];
    const MODULE_TOPIC: &'static str = constants::NODE;
    fn handler(&self) {
        info!("i am SignedTransaction handler");
    }
}

impl Message for SignedBlock {
    const TOPIC: &'static str = NODE_MSG_TOPIC_STR[1];
    const MODULE_TOPIC: &'static str = constants::NODE;
    fn handler(&self) {
        info!("i am SignedBlock handler");
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum NodeMessageTypes {
    SignedTransactionEnum(SignedTransaction),
    SignedBlockEnum(SignedBlock),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ConsensusMessageTypes {
    LeaderElect(SignedLeaderElection),
    ConsensusPing(ElectionPing),
    ConsensusPong(ElectionPong),
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
            NodeMessageTypes::SignedBlockEnum(data) => TopicBuilder::new(data.topic()).build(),
            NodeMessageTypes::SignedTransactionEnum(data) => {
                TopicBuilder::new(data.topic()).build()
            }
        }
    }
}

//TODO : Try using macro to implement this for all variations
impl From<ConsensusMessageTypes> for Topic {
    fn from(msg: ConsensusMessageTypes) -> Topic {
        match msg {
            ConsensusMessageTypes::LeaderElect(data) => TopicBuilder::new(data.topic()).build(),
            ConsensusMessageTypes::ConsensusPing(data) => TopicBuilder::new(data.topic()).build(),
            ConsensusMessageTypes::ConsensusPong(data) => TopicBuilder::new(data.topic()).build(),
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

///Process FloodSubMessages
///
pub trait MsgProcess {
    fn process(&self, topics: &Vec<TopicHash>, data: &Vec<u8>);
}

pub trait NodeMsgProcess {
    fn process_node_msg(&self, topic: &TopicHash, data: &Vec<u8>);
}

impl MsgProcess for protocol::FloodsubMessage {
    fn process(&self, topics: &Vec<TopicHash>, data: &Vec<u8>) {
        if topics[0] == TopicBuilder::new(constants::NODE).build().hash().clone() {
            info!("NodeMessageTypes data received");
            let deserialize_msg = deserialize::<NodeMessageTypes>(data);
            let result = MSG_DISPATCHER
                .node_msg_dispatcher
                .clone()
                .try_send(Some(deserialize_msg));
            if result.is_err() {
                result.unwrap_err().into_send_error();
            }
        } else if topics[0]
            == TopicBuilder::new(constants::CONSENSUS)
                .build()
                .hash()
                .clone()
        {
            info!("ConsensusMessageTypes data received");
            let deserialize_msg = deserialize::<ConsensusMessageTypes>(data);
            let result = MSG_DISPATCHER
                .consensus_msg_dispatcher
                .clone()
                .try_send(Some(deserialize_msg));
            if result.is_err() {
                result.unwrap_err().into_send_error();
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct MessageDispatcher {
    pub node_msg_dispatcher: Sender<Option<NodeMessageTypes>>,
    pub node_msg_receiver: Arc<Mutex<Receiver<Option<NodeMessageTypes>>>>,
    pub consensus_msg_dispatcher: Sender<Option<ConsensusMessageTypes>>,
    pub consensus_msg_receiver: Arc<Mutex<Receiver<Option<ConsensusMessageTypes>>>>,
}

impl MessageDispatcher {
    pub fn new() -> Self {
        let (tx, rx) = channel::<Option<NodeMessageTypes>>(1024);
        let (tx_consensus, rx_consensus) = channel::<Option<ConsensusMessageTypes>>(1024);
        MessageDispatcher {
            node_msg_dispatcher: tx,
            node_msg_receiver: Arc::new(Mutex::new(rx)),
            consensus_msg_dispatcher: tx_consensus,
            consensus_msg_receiver: Arc::new(Mutex::new(rx_consensus)),
        }
    }
    pub fn set_node_msg_dispatcher(&mut self, tx: &Sender<Option<NodeMessageTypes>>) {
        self.node_msg_dispatcher = tx.clone();
    }

    pub fn set_consensus_msg_dispatcher(&mut self, tx: &Sender<Option<ConsensusMessageTypes>>) {
        self.consensus_msg_dispatcher = tx.clone();
    }
}

lazy_static! {
    pub static ref MSG_DISPATCHER: MessageDispatcher = MessageDispatcher::new();
}

#[cfg(test)]
mod test_messages {

    #[test]
    pub fn check_leader_election_hash_comparison() {
        use super::*;
        use std::collections::hash_map::DefaultHasher;
        let mut hasher = DefaultHasher::new();
        let leader_1 = LeaderElection {
            block_height: 0,
            old_leader: "String".to_string(),
            new_leader: "String".to_string(),
        };
        let mut hasher_1 = DefaultHasher::new();
        let leader_2 = LeaderElection {
            block_height: 0,
            old_leader: "Strin3g".to_string(),
            new_leader: "String".to_string(),
        };
        leader_1.hash(&mut hasher);
        leader_2.hash(&mut hasher_1);
        info!("Hash is {:x}!", hasher_1.finish());
        info!("Hash is {:x}!", hasher.finish());
        if hasher.finish() > hasher_1.finish() {
            info!("leader_1");
        } else {
            info!("leader_2");
        }
    }
}
