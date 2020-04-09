extern crate schema;
extern crate utils;

use exonum_crypto::Hash;
use exonum_derive::FromAccess;
use exonum_merkledb::{
    access::{Access, FromAccess, RawAccess},
    ListIndex, ObjectHash, ProofMapIndex,
};
use schema::block::SignedBlock;
use schema::transaction::{SignedTransaction, State};

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
        if self.block_list.len() > 0 {
            return self.block_list.get(self.block_list.len() - 1);
        } else {
            return Option::None;
        }
    }

    pub fn get_root_block_hash(&self) -> Hash {
        if self.block_list.len() > 0 {
            return self
                .block_list
                .get(self.block_list.len() - 1)
                .unwrap()
                .object_hash();
        } else {
            return Hash::zero();
        }
    }

    pub fn get_block(&self, index: u64) -> Option<SignedBlock> {
        self.block_list.get(index)
    }

    pub fn get_block_hash(&self, index: u64) -> Hash {
        self.block_list.get(index).unwrap().object_hash()
    }

    pub fn get_blockchain_length(&self) -> u64 {
        self.blocks().len()
    }

    pub fn get_state(&self, public_key: String) -> Option<State> {
        self.state().get(&public_key)
    }
}

#[cfg(test)]
mod test_db_service {

    #[test]
    pub fn test_schema() {
        use super::*;
        use crate::db_layer::snapshot_db;
        let public_key =
            String::from("2c8a35450e1d198e3834d933a35962600c33d1d0f8f6481d6e08f140791374d0");
        let snapshot = snapshot_db();
        {
            let schema = SchemaSnap::new(&snapshot);
            info!("----printing details----");
            info!("block chain length {}", schema.get_blockchain_length());
            info!(
                "block chain root block hash {}",
                schema.get_root_block_hash()
            );
            info!(
                "transaction_trie_hash {}",
                schema.get_transaction_trie_hash()
            );
            info!("state_trie_hash {}", schema.get_state_trie_hash());
            info!("storage_trie_hash {}", schema.get_storage_trie_hash());
            info!("user state {:?}", schema.get_state(public_key));
        }
    }
}
