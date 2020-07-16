extern crate utils;
use exonum_crypto::Hash;
use exonum_merkledb::ObjectHash;
use std::fmt;
use std::time::SystemTime;
use std::{borrow::Cow, convert::AsRef};
use utils::keypair::{CryptoKeypair, Keypair, KeypairType, PublicKey, Verify};
use utils::serializer::{serialize, Deserialize, Serialize};

pub trait BlockTraits<T> {
    fn validate(&self, publickey: &String, signature: &[u8]) -> bool;
    fn sign(&self, kp: &T) -> Vec<u8>;
    fn genesis_block(custom_headers: Vec<u8>) -> Self;
    fn new_block(
        id: u64,
        peer_id: String,
        prev_hash: Hash,
        txn_pool: Vec<Hash>,
        header: [Hash; 3],
        custom_headers: Vec<u8>,
    ) -> Self;
}

pub trait SignedBlockTraits<T> {
    fn validate(&self, publickey: &String) -> bool;
    fn create_block(block: Block, sig: Vec<u8>) -> Self;
    fn get_hash(&self) -> Hash;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, BinaryValue, ObjectHash)]
#[binary_value(codec = "bincode")]
pub struct Block {
    pub id: u64,
    pub peer_id: String,
    pub prev_hash: Hash,
    pub txn_pool: Vec<Hash>,
    // txn_trie, state_trie, storage_trie
    pub header: [Hash; 3],
    // block creation time
    pub timestamp: u128,
    // custom defined data
    pub custom_headers: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, BinaryValue, ObjectHash)]
#[binary_value(codec = "bincode")]
pub struct SignedBlock {
    pub block: Block,
    pub signature: Vec<u8>,
    pub auth_headers: Vec<u8>,
}

impl fmt::Display for Block {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "id: {:?} \
            peer_id: {:?} \
            previous_hash: {:?} \
            timestamp: {:?} \
            header: {:?} \
            txn_pool: {:?}",
            self.id, self.peer_id, self.prev_hash, self.timestamp, self.header, self.txn_pool
        )
    }
}

impl fmt::Display for SignedBlock {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.block)
    }
}

impl Block {
    pub fn to_string_format(&self) -> String {
        self.to_string()
    }
}

impl SignedBlock {
    pub fn to_string_format(&self) -> String {
        self.to_string()
    }

    pub fn validate(&self) -> bool {
        let ser_block: Vec<u8> = match serialize(&self.block) {
            Result::Ok(value) => value,
            Result::Err(_) => return false,
        };
        PublicKey::verify_from_encoded_pk(&self.block.peer_id, &ser_block, &self.signature)
    }

    pub fn create_block(block: Block, signature: Vec<u8>, auth_headers: Vec<u8>) -> SignedBlock {
        SignedBlock {
            block,
            signature,
            auth_headers,
        }
    }

    pub fn from_bytes(bytes: Cow<'_, [u8]>) -> anyhow::Result<Self> {
        bincode::deserialize(bytes.as_ref()).map_err(From::from)
    }

    pub fn get_hash(&self) -> Hash {
        self.object_hash()
    }
}

impl BlockTraits<KeypairType> for Block {
    fn validate(&self, publickey: &String, signature: &[u8]) -> bool {
        let ser_block: Vec<u8> = match serialize(&self) {
            Result::Ok(value) => value,
            Result::Err(_) => return false,
        };
        PublicKey::verify_from_encoded_pk(&publickey, &ser_block, &signature)
    }

    fn sign(&self, kp: &KeypairType) -> Vec<u8> {
        let ser_block: Vec<u8> = match serialize(&self) {
            Result::Ok(value) => value,
            Result::Err(_) => return vec![0],
        };
        let sign = Keypair::sign(&kp, &ser_block);
        sign
    }

    fn genesis_block(custom_headers: Vec<u8>) -> Block {
        let timestamp: u128 = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_micros();
        Block {
            id: 0,
            peer_id: String::from("genesis_block"),
            prev_hash: Hash::zero(),
            txn_pool: vec![],
            header: [Hash::zero(), Hash::zero(), Hash::zero()],
            timestamp,
            custom_headers,
        }
    }

    fn new_block(
        id: u64,
        peer_id: String,
        prev_hash: Hash,
        txn_pool: Vec<Hash>,
        header: [Hash; 3],
        custom_headers: Vec<u8>,
    ) -> Block {
        let timestamp: u128 = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_micros();
        Block {
            id,
            peer_id,
            prev_hash,
            txn_pool,
            header,
            timestamp,
            custom_headers,
        }
    }
}

#[cfg(test)]
mod tests_block {

    use super::*;

    #[test]
    pub fn test_create_block() {
        let kp: KeypairType = Keypair::generate();
        let pk: String = hex::encode(kp.public().encode());
        let block: Block = Block::new_block(
            1,
            pk.clone(),
            Hash::zero(),
            vec![Hash::zero()],
            [Hash::zero(), Hash::zero(), Hash::zero()],
            Vec::new(),
        );
        let signed_block: SignedBlock =
            SignedBlock::create_block(block.clone(), block.sign(&kp), Vec::new());
        assert_eq!(
            signed_block.validate(),
            true,
            "Issue with Signature Verification"
        );
    }

    #[test]
    pub fn test_genesis_block() {
        let kp: KeypairType = Keypair::generate();
        let genesis_block: Block = Block::genesis_block(Vec::new());
        let signature: Vec<u8> = genesis_block.sign(&kp);
        let signed_block: SignedBlock =
            SignedBlock::create_block(genesis_block, signature, Vec::new());
        assert_eq!(
            signed_block.validate(),
            false,
            "Issue with Signature Verification"
        );
        let data: String = signed_block.to_string_format();
        println!("{:?}", data);
        let data: String = signed_block.block.to_string_format();
        println!("{:?}", data);
    }
}
