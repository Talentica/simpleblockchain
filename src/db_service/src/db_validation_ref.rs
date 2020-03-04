extern crate schema;
extern crate utils;
use app_1::state::State;
use exonum_crypto::Hash;
use exonum_derive::FromAccess;
use exonum_merkledb::{
    access::{Access, FromAccess, RawAccessMut},
    ListIndex, ObjectHash, ProofMapIndex,
};
use generic_traits::traits::{StateTraits, TransactionTrait};
use schema::block::{BlockTraits, SignedBlock};
use schema::transaction::SignedTransaction;
use schema::transaction_pool::{TransactionPool, TxnPool};

#[derive(FromAccess)]
pub struct SchemaValidate<T: Access> {
    txn_trie: ProofMapIndex<T::Base, Hash, SignedTransaction>,
    block_list: ListIndex<T::Base, SignedBlock>,
    state_trie: ProofMapIndex<T::Base, String, State>,
    storage_trie: ProofMapIndex<T::Base, Hash, SignedTransaction>,
}

impl<T: Access> SchemaValidate<T> {
    fn new(access: T) -> Self {
        Self::from_root(access).unwrap()
    }
}

impl<T: Access> SchemaValidate<T>
where
    T::Base: RawAccessMut,
{
    pub fn txn_trie_merkle_hash(&self) -> Hash {
        self.txn_trie.object_hash()
    }

    pub fn state_trie_merkle_hash(&self) -> Hash {
        self.state_trie.object_hash()
    }

    pub fn storage_trie_merkle_hash(&self) -> Hash {
        self.storage_trie.object_hash()
    }

    pub fn validate_block(
        &mut self,
        signed_block: &SignedBlock,
        txn_pool: &mut TransactionPool,
    ) -> bool {
        let block = &signed_block.block;
        if !block.validate(&signed_block.block.peer_id, &signed_block.signature) {
            return false;
        }
        // TODO: this logic should be modified after consesus integration
        if self.validate_transactions(&block.txn_pool, &txn_pool) {
            let length = self.block_list.len();
            let last_block: SignedBlock = self.block_list.get(length - 1).unwrap();
            let prev_hash = last_block.object_hash();
            if prev_hash != block.prev_hash {
                println!("check1");
                return false;
            }
            if length != block.id {
                println!("check2");
                return false;
            }
            if self.state_trie_merkle_hash() != block.header[0] {
                println!("check3");
                return false;
            }
            if self.storage_trie_merkle_hash() != block.header[1] {
                println!("check5");
                return false;
            }
            if self.txn_trie_merkle_hash() != block.header[2] {
                println!("check4");
                return false;
            }
            return true;
        } else {
            println!("check0");
            return false;
        }
    }

    pub fn validate_transactions(&self, hash_vec: &Vec<Hash>, txn_pool: &TransactionPool) -> bool {
        for txn_hash in hash_vec.into_iter() {
            let txn: SignedTransaction = match txn_pool.get(txn_hash) {
                None => return false,
                Some(txn) => txn.clone(),
            };
            if txn.validate() {
                // if txn.execute(state_trie) {
                //     transaction_trie.put(&txn_hash, txn.clone());
                // } else {
                //     return false;
                // }
            } else {
                return false;
            }
        }
        return true;
    }
}
