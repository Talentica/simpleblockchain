extern crate schema;
extern crate utils;
use app_2::state::State;
use exonum_crypto::Hash;
use exonum_merkledb::{ListIndex, ObjectAccess, ObjectHash, ProofMapIndex, RefMut};
use generic_traits::traits::{StateTraits, TransactionTrait};
use schema::block::{BlockTraits, SignedBlock};
use schema::transaction::SignedTransaction;
use schema::transaction_pool::{TransactionPool, TxnPool};

pub struct SchemaValidate<T: ObjectAccess>(T);

impl<T: ObjectAccess> SchemaValidate<T> {
    pub fn new(object_access: T) -> Self {
        Self(object_access)
    }

    pub fn transactions(&self) -> RefMut<ProofMapIndex<T, Hash, SignedTransaction>> {
        self.0.get_object("transactions")
    }

    pub fn txn_trie_merkle_hash(&self) -> Hash {
        self.transactions().object_hash()
    }

    pub fn blocks(&self) -> RefMut<ListIndex<T, SignedBlock>> {
        self.0.get_object("blocks")
    }

    pub fn state(&self) -> RefMut<ProofMapIndex<T, String, State>> {
        self.0.get_object("state_trie")
    }

    pub fn state_trie_merkle_hash(&self) -> Hash {
        self.state().object_hash()
    }

    pub fn storage(&self) -> RefMut<ProofMapIndex<T, Hash, SignedTransaction>> {
        self.0.get_object("storage_trie")
    }

    pub fn storage_trie_merkle_hash(&self) -> Hash {
        self.storage().object_hash()
    }

    pub fn validate_block(
        &self,
        signed_block: &SignedBlock,
        txn_pool: &mut TransactionPool,
    ) -> bool {
        let mut state_trie = self.state();
        let mut transaction_trie = self.transactions();
        let storage_trie = self.storage();
        let block = &signed_block.block;
        if !block.validate(&signed_block.block.peer_id, &signed_block.signature) {
            return false;
        }
        // TODO: this logic should be modified after consesus integration
        if self.validate_transactions(
            &block.txn_pool,
            &mut state_trie,
            &mut transaction_trie,
            &txn_pool,
        ) {
            let blocks = self.blocks();
            let length = blocks.len();
            let last_block: SignedBlock = blocks.get(length - 1).unwrap();
            let prev_hash = last_block.object_hash();
            if prev_hash != block.prev_hash {
                println!("check1");
                return false;
            }
            if length != block.id {
                println!("check2");
                return false;
            }
            if state_trie.object_hash() != block.header[0] {
                println!("check3");
                return false;
            }
            if storage_trie.object_hash() != block.header[1] {
                println!("check5");
                return false;
            }
            if transaction_trie.object_hash() != block.header[2] {
                println!("check4");
                return false;
            }
            return true;
        } else {
            println!("check0");
            return false;
        }
    }

    pub fn validate_transactions(
        &self,
        hash_vec: &Vec<Hash>,
        state_trie: &mut RefMut<ProofMapIndex<T, String, State>>,
        transaction_trie: &mut RefMut<ProofMapIndex<T, Hash, SignedTransaction>>,
        txn_pool: &TransactionPool,
    ) -> bool {
        for txn_hash in hash_vec.into_iter() {
            let txn: SignedTransaction = match txn_pool.get(txn_hash) {
                None => return false,
                Some(txn) => txn.clone(),
            };
            if txn.validate() {
                if txn.execute(state_trie) {
                    transaction_trie.put(&txn_hash, txn.clone());
                } else {
                    return false;
                }
            } else {
                return false;
            }
        }
        return true;
    }
}
