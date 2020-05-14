extern crate utils;
use exonum_crypto::Hash;
// use exonum_merkledb::{impl_object_hash_for_binary_value, BinaryValue, ObjectHash};
use std::{borrow::Cow, convert::AsRef};
use utils::keypair::{CryptoKeypair, Keypair, KeypairType, PublicKey, Verify};
use utils::serializer::{serialize, Deserialize, Serialize};

pub trait BlockTraits<T> {
    fn validate(&self, publickey: &String, signature: &[u8]) -> bool;
    fn sign(&self, kp: &T) -> Vec<u8>;
    fn genesis_block() -> Self;
    fn new_block(
        id: u64,
        peer_id: String,
        prev_hash: Hash,
        txn_pool: Vec<Hash>,
        header: [Hash; 3],
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
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, BinaryValue, ObjectHash)]
#[binary_value(codec = "bincode")]
pub struct SignedBlock {
    pub block: Block,
    pub signature: Vec<u8>,
}

impl Block {
    pub fn to_string_format(&self) -> String {
        let mut to_string: String = String::new();
        to_string.extend("id: ".chars());
        to_string.extend(self.id.to_string().chars());
        to_string.extend(", peer_id: ".chars());
        to_string.extend(self.peer_id.chars());
        to_string.extend(", prev_hash: ".chars());
        to_string.extend(self.prev_hash.to_hex().chars());
        to_string.extend(", txn_pool: ".chars());
        for each in self.txn_pool.iter() {
            to_string.extend(each.to_hex().chars());
            to_string.extend(", ".chars());
        }
        to_string.extend(", header: ".chars());
        for each in 0..3 {
            to_string.extend(self.header[each].to_hex().chars());
            to_string.extend(", ".chars());
        }
        to_string
    }
}

impl SignedBlock {
    pub fn to_string_format(&self) -> String {
        let mut to_string: String = String::new();
        to_string.extend("Block: ".chars());
        to_string.extend(self.block.to_string_format().chars());
        to_string
    }
    pub fn validate(&self, publickey: &String) -> bool {
        let ser_block: Vec<u8> = match serialize(&self.block) {
            Result::Ok(value) => value,
            Result::Err(_) => return false,
        };
        PublicKey::verify_from_encoded_pk(&publickey, &ser_block, &self.signature)
    }

    pub fn create_block(block: Block, signature: Vec<u8>) -> SignedBlock {
        SignedBlock { block, signature }
    }

    pub fn from_bytes(bytes: Cow<'_, [u8]>) -> anyhow::Result<Self> {
        bincode::deserialize(bytes.as_ref()).map_err(From::from)
    }
}

impl BlockTraits<KeypairType> for Block {
    fn validate(&self, publickey: &String, signature: &[u8]) -> bool {
        let ser_block: Vec<u8> = match serialize(&self) {
            Result::Ok(value) => value,
            Result::Err(_) => return false,
        };
        PublicKey::verify_from_encoded_pk(&publickey, &ser_block, &signature)
        // PublicKey::verify_from_encoded_pk(&self.txn.party_a, signing_string.as_bytes(), &self.signature.as_ref())
    }

    fn sign(&self, kp: &KeypairType) -> Vec<u8> {
        let ser_block: Vec<u8> = match serialize(&self) {
            Result::Ok(value) => value,
            Result::Err(_) => return vec![0],
        };
        let sign = Keypair::sign(&kp, &ser_block);
        sign
    }

    fn genesis_block() -> Block {
        Block {
            id: 0,
            peer_id: String::from("to_be_decided"),
            prev_hash: Hash::zero(),
            txn_pool: vec![],
            header: [Hash::zero(), Hash::zero(), Hash::zero()],
        }
    }

    fn new_block(
        id: u64,
        peer_id: String,
        prev_hash: Hash,
        txn_pool: Vec<Hash>,
        header: [Hash; 3],
    ) -> Block {
        Block {
            id,
            peer_id,
            prev_hash,
            txn_pool,
            header,
        }
    }
}

// #[cfg(test)]
// mod tests_blocks {
//     #[test]
//     pub fn test_block() {
//         use super::*;
//         let block: Block = Block::genesis_block();
//         let kp = Keypair::generate();
//         let public_key = &hex::encode(kp.public().encode());
//         let sign = block.sign(&kp);
//         let signed_block: SignedBlock = SignedBlock::create_block(block, sign);
//         println!("{}", signed_block.validate(&public_key));
//         let prev_hash: Hash = signed_block.get_hash();
//         let id = signed_block.block.id;
//         let block: Block = Block {
//             id: id + 1,
//             peer_id: String::from("peer_id"),
//             prev_hash,
//             txn_pool: vec![],
//             header: [Hash::zero(), Hash::zero(), Hash::zero()],
//         };
//         let sign = block.sign(&kp);
//         let validate = block.validate(&hex::encode(kp.public().encode()), &sign);
//         println!("{}", validate);
//     }
// }
