extern crate utils;
use exonum_crypto::Hash;
use exonum_merkledb::{impl_object_hash_for_binary_value, BinaryValue, ObjectHash};
use failure::Error;
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
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct Block {
    pub id: u64,
    pub peer_id: String,
    pub prev_hash: Hash,
    pub txn_pool: Vec<Hash>,
    // txn_trie, state_trie, storage_trie
    pub header: [Hash; 3],
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct SignedBlock {
    pub block: Block,
    pub signature: Vec<u8>,
}

impl BlockTraits<KeypairType> for Block {
    fn validate(&self, publickey: &String, signature: &[u8]) -> bool {
        // unimplemented!();
        let ser_block = serialize(&self);
        PublicKey::verify_from_encoded_pk(&publickey, &ser_block, &signature)
        // PublicKey::verify_from_encoded_pk(&self.txn.party_a, signing_string.as_bytes(), &self.signature.as_ref())
    }

    fn sign(&self, kp: &KeypairType) -> Vec<u8> {
        // unimplemented!();
        let ser_block = serialize(&self);
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

impl SignedBlockTraits<KeypairType> for SignedBlock {
    fn validate(&self, publickey: &String) -> bool {
        let ser_block = serialize(&self.block);
        PublicKey::verify_from_encoded_pk(&publickey, &ser_block, &self.signature)
    }

    fn create_block(block: Block, signature: Vec<u8>) -> SignedBlock {
        SignedBlock { block, signature }
    }
}

impl BinaryValue for SignedBlock {
    fn to_bytes(&self) -> Vec<u8> {
        bincode::serialize(self).unwrap()
    }

    fn from_bytes(bytes: Cow<'_, [u8]>) -> Result<Self, Error> {
        bincode::deserialize(bytes.as_ref()).map_err(From::from)
    }
}

impl_object_hash_for_binary_value! { SignedBlock}

#[cfg(test)]
mod tests_blocks {

    #[test]
    pub fn test_block() {
        use super::*;
        let block: Block = Block::genesis_block();
        let kp = Keypair::generate();
        let public_key = &hex::encode(kp.public().encode());
        let sign = block.sign(&kp);
        let signed_block: SignedBlock = SignedBlock::create_block(block, sign);
        println!("{}", signed_block.validate(&public_key));
        let prev_hash = signed_block.object_hash();
        let id = signed_block.block.id;
        let block: Block = Block {
            id: id + 1,
            peer_id: String::from("peer_id"),
            prev_hash,
            txn_pool: vec![],
            header: [Hash::zero(), Hash::zero(), Hash::zero()],
        };
        let sign = block.sign(&kp);
        let validate = block.validate(&hex::encode(kp.public().encode()), &sign);
        println!("{}", validate);
    }
}
