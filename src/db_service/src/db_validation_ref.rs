extern crate db;
extern crate schema;
extern crate utils;
use exonum_crypto::Hash;
use exonum_merkledb::{ListIndex, ObjectAccess, ObjectHash, ProofMapIndex, RefMut};
use schema::block::{Block, BlockTraits, SignedBlock, SignedBlockTraits};
use schema::transaction::{SignedTransaction, Txn};
use schema::transaction_pool::{TransactionPool, TxnPool};
use schema::wallet::Wallet;
use utils::keypair::{CryptoKeypair, Keypair, KeypairType};

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

    pub fn state(&self) -> RefMut<ProofMapIndex<T, String, Wallet>> {
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
        let mut wallets = self.state();
        let mut transaction_trie = self.transactions();
        let storage_trie = self.storage();
        let block = &signed_block.block;
        if !block.validate(&signed_block.block.peer_id, &signed_block.signature) {
            return false;
        }
        // TODO: this logic should be modified after consesus integration
        if self.validate_transactions(
            &block.txn_pool,
            &mut wallets,
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
            if wallets.object_hash() != block.header[0] {
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
        wallet: &mut RefMut<ProofMapIndex<T, String, Wallet>>,
        transaction_trie: &mut RefMut<ProofMapIndex<T, Hash, SignedTransaction>>,
        txn_pool: &TransactionPool,
    ) -> bool {
        for txn_hash in hash_vec.into_iter() {
            let txn: SignedTransaction = match txn_pool.get(txn_hash) {
                None => return false,
                Some(txn) => txn.clone(),
            };
            if txn.validate() {
                if wallet.contains(&txn.txn.from) {
                    let mut from_wallet: Wallet = wallet.get(&txn.txn.from).unwrap();
                    if from_wallet.get_balance() > txn.txn.amount {
                        if wallet.contains(&txn.txn.to) {
                            let mut to_wallet = wallet.get(&txn.txn.to).unwrap();
                            to_wallet.add_balance(txn.txn.amount);
                            wallet.put(&txn.txn.to.clone(), to_wallet);
                        } else {
                            let mut to_wallet: Wallet = Wallet::new();
                            to_wallet.add_balance(txn.txn.amount);
                            wallet.put(&txn.txn.to.clone(), to_wallet);
                        }
                        from_wallet.deduct_balance(txn.txn.amount);
                        from_wallet.increase_nonce();
                        wallet.put(&txn.txn.from.clone(), from_wallet);
                        transaction_trie.put(&txn_hash, txn.clone());
                    } else {
                        return false;
                    }
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

#[cfg(test)]
mod test_db_service {

    #[test]
    pub fn test_schema() {
        use super::*;
        use chrono::prelude::Utc;
        use db::db_layer::{fork_db};
        use utils::keypair::{CryptoKeypair, Keypair};
        let mut secret = hex::decode("97ba6f71a5311c4986e01798d525d0da8ee5c54acbf6ef7c3fadd1e2f624442f")
                .expect("invalid secret");
        let keypair = Keypair::generate_from(secret.as_mut_slice());
        let _public_key = String::from("2c8a35450e1d198e3834d933a35962600c33d1d0f8f6481d6e08f140791374d0");
        let fork = fork_db();
        {
            let schema = SchemaValidate::new(&fork);
            // let block = schema.validate_block(signed_block: &SignedBlock, txn_pool: &mut TransactionPool)(&keypair, &mut txn_pool);
            // println!("{:?}", block);
        }
    }
}
