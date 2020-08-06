use libp2p::floodsub::Topic;
use message_handler::constants;
use message_handler::message_traits::Message;
use std::hash::{Hash, Hasher};
use utils::keypair::{CryptoKeypair, Keypair, KeypairType, PublicKey, Verify};
use utils::serializer::{serialize, Deserialize, Serialize};

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
        let ser_payload: Vec<u8> = match serialize(&self.payload) {
            Result::Ok(value) => value,
            Result::Err(_) => return false,
        };
        PublicKey::verify_from_encoded_pk(
            &self.payload.public_key,
            &ser_payload,
            &self.signature.as_ref(),
        )
    }

    fn sign(&mut self, kp: &KeypairType) {
        let ser_txn: Vec<u8> = match serialize(&self.payload) {
            Result::Ok(value) => value,
            Result::Err(_) => vec![0],
        };
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
        let ser_payload: Vec<u8> = match serialize(&self.payload) {
            Result::Ok(value) => value,
            Result::Err(_) => return false,
        };
        PublicKey::verify_from_encoded_pk(
            &self.payload.may_be_leader,
            &ser_payload,
            &self.signature.as_ref(),
        )
    }

    fn sign(&mut self, kp: &KeypairType) {
        let ser_txn: Vec<u8> = match serialize(&self.payload) {
            Result::Ok(value) => value,
            Result::Err(_) => vec![0],
        };
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

#[derive(Debug, Serialize, Deserialize)]
pub enum ConsensusMessageTypes {
    LeaderElect(SignedLeaderElection),
    ConsensusPing(ElectionPing),
    ConsensusPong(ElectionPong),
}

//TODO : Try using macro to implement this for all variations
impl From<ConsensusMessageTypes> for Topic {
    fn from(msg: ConsensusMessageTypes) -> Topic {
        match msg {
            ConsensusMessageTypes::LeaderElect(data) => Topic::new(data.topic()),
            ConsensusMessageTypes::ConsensusPing(data) => Topic::new(data.topic()),
            ConsensusMessageTypes::ConsensusPong(data) => Topic::new(data.topic()),
        }
    }
}

#[cfg(test)]
mod consensus_message_test {

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
