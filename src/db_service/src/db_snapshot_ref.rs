extern crate schema;
extern crate utils;

use exonum_crypto::Hash;
use exonum_derive::FromAccess;
use exonum_merkledb::{
    access::{Access, FromAccess, RawAccess},
    ListIndex, ObjectHash, ProofMapIndex,
};
use schema::block::SignedBlock;
use schema::signed_transaction::SignedTransaction;
use schema::state::State;

#[derive(FromAccess)]
pub struct SchemaSnap<T: Access> {
    pub txn_trie: ProofMapIndex<T::Base, Hash, SignedTransaction>,
    block_list: ListIndex<T::Base, SignedBlock>,
    state_trie: ProofMapIndex<T::Base, String, State>,
    storage_trie: ProofMapIndex<T::Base, Hash, SignedTransaction>,
}

impl<T: Access> SchemaSnap<T> {
    pub fn new(access: T) -> Self {
        Self::from_root(access).unwrap()
    }
}

impl<T: Access> SchemaSnap<T>
where
    T::Base: RawAccess,
{
    pub fn is_db_initialized(&self) -> bool {
        if self.get_blockchain_length() > 0 {
            true
        } else {
            false
        }
    }

    pub fn transactions(&self) -> &ProofMapIndex<T::Base, Hash, SignedTransaction> {
        &self.txn_trie
    }

    pub fn blocks(&self) -> &ListIndex<T::Base, SignedBlock> {
        &self.block_list
    }

    pub fn state(&self) -> &ProofMapIndex<T::Base, String, State> {
        &self.state_trie
    }

    pub fn storage(&self) -> &ProofMapIndex<T::Base, Hash, SignedTransaction> {
        &self.storage_trie
    }

    pub fn get_transaction_trie_hash(&self) -> Hash {
        self.txn_trie.object_hash()
    }

    pub fn get_state_trie_hash(&self) -> Hash {
        self.state_trie.object_hash()
    }

    pub fn get_storage_trie_hash(&self) -> Hash {
        self.storage_trie.object_hash()
    }

    pub fn get_transaction(&self, hash: Hash) -> Option<SignedTransaction> {
        self.transactions().get(&hash)
    }

    pub fn get_root_block(&self) -> Option<SignedBlock> {
        let length: u64 = self.get_blockchain_length();
        if length > 0 {
            return self.get_block(length - 1);
        } else {
            return Option::None;
        }
    }

    pub fn get_root_block_hash(&self) -> Hash {
        match self.get_root_block() {
            Some(root_block) => root_block.get_hash(),
            None => Hash::zero(),
        }
    }

    pub fn get_block(&self, index: u64) -> Option<SignedBlock> {
        self.block_list.get(index)
    }

    pub fn get_block_hash(&self, index: u64) -> Hash {
        if self.get_blockchain_length() > index {
            match self.get_block(index) {
                Some(block) => return block.get_hash(),
                None => return Hash::zero(),
            };
        } else {
            return Hash::zero();
        }
    }

    pub fn get_blockchain_length(&self) -> u64 {
        self.blocks().len()
    }

    pub fn get_state(&self, public_key: String) -> Option<State> {
        self.state().get(&public_key)
    }
}
