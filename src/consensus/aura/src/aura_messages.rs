use exonum_crypto::Hash;
use libp2p::floodsub::{Topic, TopicBuilder};
use message_handler::constants;
use message_handler::message_traits::Message;
use schema::block::SignedBlock;
use std::time::SystemTime;
use utils::keypair::{CryptoKeypair, Keypair, KeypairType, PublicKey, Verify};
use utils::serializer::{serialize, Deserialize, Serialize};

pub const AURA_MSG_TOPIC_STR: &'static [&'static str] =
    &["RoundOwner", "BlockAcceptance", "AuthorBlock"];

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RoundDetails {
    pub unix_time: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RoundOwner {
    pub round_details: RoundDetails,
    pub signature: Vec<u8>,
    pub public_key: String,
}

impl RoundOwner {
    pub fn verify(&self, step_time: u64) -> bool {
        let stamp: u64 = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        if stamp - self.round_details.unix_time > step_time {
            return false;
        }
        let ser_payload: Vec<u8> = match serialize(&self.round_details) {
            Result::Ok(value) => value,
            Result::Err(_) => return false,
        };
        PublicKey::verify_from_encoded_pk(&self.public_key, &ser_payload, &self.signature.as_ref())
    }

    fn sign(&mut self, kp: &KeypairType) {
        let ser_txn: Vec<u8> = match serialize(&self.round_details) {
            Result::Ok(value) => value,
            Result::Err(_) => vec![0],
        };
        let sign = Keypair::sign(&kp, &ser_txn);
        self.signature = sign;
    }

    pub fn create(kp: &KeypairType) -> RoundOwner {
        let round_details: RoundDetails = RoundDetails {
            unix_time: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };
        let mut round_config: RoundOwner = RoundOwner {
            round_details,
            signature: vec![0],
            public_key: hex::encode(kp.public().encode()),
        };
        round_config.sign(kp);
        round_config
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BlockAcceptance {
    pub signature: Vec<u8>,
    pub block_hash: Hash,
    pub public_key: String,
}

impl BlockAcceptance {
    pub fn verify(&self) -> bool {
        let ser_payload: Vec<u8> = match serialize(&self.block_hash) {
            Result::Ok(value) => value,
            Result::Err(_) => return false,
        };
        PublicKey::verify_from_encoded_pk(&self.public_key, &ser_payload, &self.signature.as_ref())
    }

    fn sign(&mut self, kp: &KeypairType) {
        let ser_txn: Vec<u8> = match serialize(&self.block_hash) {
            Result::Ok(value) => value,
            Result::Err(_) => vec![0],
        };
        let sign = Keypair::sign(&kp, &ser_txn);
        self.signature = sign;
    }

    pub fn create(kp: &KeypairType, block_hash: Hash) -> BlockAcceptance {
        let mut block_acceptance: BlockAcceptance = BlockAcceptance {
            signature: vec![0],
            block_hash,
            public_key: hex::encode(kp.public().encode()),
        };
        block_acceptance.sign(kp);
        block_acceptance
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthorBlock {
    pub block: SignedBlock,
}

impl AuthorBlock {
    pub fn verify(&self) -> bool {
        self.block.validate()
    }

    pub fn create(block: SignedBlock) -> AuthorBlock {
        AuthorBlock { block }
    }
}

impl Message for RoundOwner {
    const TOPIC: &'static str = AURA_MSG_TOPIC_STR[0];
    const MODULE_TOPIC: &'static str = constants::CONSENSUS;
    fn handler(&self) {
        info!("i am RoundOwner handler");
    }
}

impl Message for BlockAcceptance {
    const TOPIC: &'static str = AURA_MSG_TOPIC_STR[1];
    const MODULE_TOPIC: &'static str = constants::CONSENSUS;
    fn handler(&self) {
        info!("i am BlockAcceptance handler");
    }
}

impl Message for AuthorBlock {
    const TOPIC: &'static str = AURA_MSG_TOPIC_STR[2];
    const MODULE_TOPIC: &'static str = constants::CONSENSUS;
    fn handler(&self) {
        info!("i am AuthorBlock handler");
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum AuraMessageTypes {
    RoundOwnerEnum(RoundOwner),
    BlockAcceptanceEnum(BlockAcceptance),
    AuthorBlockEnum(AuthorBlock),
}

//TODO : Try using macro to implement this for all variations
impl From<AuraMessageTypes> for Topic {
    fn from(msg: AuraMessageTypes) -> Topic {
        match msg {
            AuraMessageTypes::RoundOwnerEnum(data) => TopicBuilder::new(data.topic()).build(),
            AuraMessageTypes::BlockAcceptanceEnum(data) => TopicBuilder::new(data.topic()).build(),
            AuraMessageTypes::AuthorBlockEnum(data) => TopicBuilder::new(data.topic()).build(),
        }
    }
}

#[cfg(test)]
mod consensus_message_test {

    #[test]
    pub fn check_aura_messages_verification_process() {
        use super::*;
        use schema::block::{Block, BlockTraits};

        let kp: KeypairType = Keypair::generate();
        let pk: String = hex::encode(kp.public().encode());
        let round_config = RoundOwner::create(&kp);
        let step_time: u64 = 3;
        assert_eq!(round_config.verify(step_time), true);

        std::thread::sleep(std::time::Duration::from_secs(step_time + 1));
        assert_eq!(round_config.verify(step_time), false);
        let block: Block = Block::new_block(
            1,
            pk.clone(),
            Hash::zero(),
            vec![Hash::zero()],
            [Hash::zero(), Hash::zero(), Hash::zero()],
            Vec::new(),
        );
        let sign: Vec<u8> = block.sign(&kp);
        let signed_block: SignedBlock =
            SignedBlock::create_block(block.clone(), sign.clone(), Vec::new());
        let block_acceptance: BlockAcceptance =
            BlockAcceptance::create(&kp, signed_block.get_hash());
        assert_eq!(block_acceptance.verify(), true);

        let author_block: AuthorBlock = AuthorBlock {
            block: signed_block,
        };
        assert_eq!(author_block.verify(), true);
    }
}
