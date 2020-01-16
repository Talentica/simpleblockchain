extern crate db;
extern crate schema;
extern crate utils;
use exonum_crypto::Hash;
use exonum_merkledb::{ListIndex, ObjectAccess, ObjectHash, ProofMapIndex, RefMut};
use schema::block::{Block, BlockTraits, SignedBlock, SignedBlockTraits};
use schema::transaction::{SignedTransaction, TransactionPool, Txn, TxnPool};
use schema::wallet::Wallet;
use std::collections::HashMap;
use std::time::Instant;
use utils::keypair::{CryptoKeypair, Keypair, KeypairType, PublicKey, Verify};

pub struct SchemaFork<T: ObjectAccess>(T);

impl<T: ObjectAccess> SchemaFork<T> {
    pub fn new(object_access: T) -> Self {
        Self(object_access)
    }

    pub fn transactions(&self) -> RefMut<ProofMapIndex<T, Hash, SignedTransaction>> {
        self.0.get_object("transactions")
    }

    pub fn blocks(&self) -> RefMut<ListIndex<T, SignedBlock>> {
        self.0.get_object("blocks")
    }

    pub fn state(&self) -> RefMut<ProofMapIndex<T, String, Wallet>> {
        self.0.get_object("state_trie")
    }

    pub fn storage(&self) -> RefMut<ProofMapIndex<T, Hash, SignedTransaction>> {
        self.0.get_object("storage_trie")
    }

    pub fn initialize_db(&self, kp: &KeypairType) {
        let mut blocks = self.blocks();
        if blocks.len() == 0 {
            let block = Block::genesis_block();
            let signature = block.sign(kp);
            let genesis_block: SignedBlock = SignedBlock::create_block(block, signature);
            blocks.push(genesis_block);
        } else {
            println!("database exists");
        }
    }

    pub fn execute_transactions(
        &self,
        txn_pool: &mut TransactionPool,
        wallet: &mut RefMut<ProofMapIndex<T, String, Wallet>>,
    ) -> Vec<SignedTransaction> {
        let mut temp_vec = Vec::<SignedTransaction>::with_capacity(10);
        while temp_vec.len() < 10 && txn_pool.length_op() > 0 {
            let txn: SignedTransaction = txn_pool.pop_front();
            if txn.validate() {
                if wallet.contains(&txn.txn.from) {
                    let mut from_wallet = wallet.get(&txn.txn.from).unwrap();
                    if from_wallet.get_balance() > txn.txn.amount {
                        if wallet.contains(&txn.txn.to) {
                            let mut to_wallet = wallet.get(&txn.txn.to).unwrap();
                            to_wallet.set_balance(to_wallet.get_balance() + txn.txn.amount);
                            wallet.put(&txn.txn.to.clone(), to_wallet);
                        } else {
                            let mut to_wallet = Wallet::new();
                            to_wallet.set_balance(txn.txn.amount);
                            wallet.put(&txn.txn.to.clone(), to_wallet);
                        }
                        from_wallet.set_balance(from_wallet.get_balance() - txn.txn.amount);
                        wallet.put(&txn.txn.from.clone(), from_wallet);
                        temp_vec.push(txn);
                    }
                }
            }
        }
        temp_vec
    }

    pub fn create_block(&self, kp: &KeypairType, txn_pool: &mut TransactionPool) -> SignedBlock {
        let mut wallets = self.state();
        let mut transaction_trie = self.transactions();
        let storage_trie = self.storage();
        let executed_txns = self.execute_transactions(txn_pool, &mut wallets);
        let mut vec_txn_hash = vec![];
        for each in executed_txns.iter() {
            let hash = each.object_hash();
            transaction_trie.put(&hash, each.clone());
            vec_txn_hash.push(hash);
        }
        let mut blocks = self.blocks();
        let length = blocks.len();
        println!("blockchain length {}", length);
        let prev_hash = blocks.get(length - 1).unwrap().object_hash();
        println!("prev_hash {:?}", prev_hash);
        let mut header: HashMap<String, Hash> = HashMap::new();
        header.insert(String::from("state_trie"), wallets.object_hash());
        header.insert(
            String::from("transaction_trie"),
            transaction_trie.object_hash(),
        );
        header.insert(String::from("storage_trie"), storage_trie.object_hash());
        let block = Block::new_block(
            length,
            String::from("to_be_decided"),
            prev_hash,
            vec_txn_hash,
            header,
        );
        let signature: Vec<u8> = block.sign(kp);
        let signed_block: SignedBlock = SignedBlock::create_block(block, signature);
        blocks.push(signed_block.clone());
        signed_block
    }

    pub fn validate_block(&self, block: &Block, txn_pool: &mut TransactionPool) -> bool {
        let mut wallets = self.state();
        let mut transaction_trie = self.transactions();
        let storage_trie = self.storage();
        // TODO: this logic should be modified after consesus integration
        let mut temp_txn_pool = TransactionPool::new();

        for _each in block.txn_pool.iter() {
            let instant = Instant::now();
            match txn_pool.get(&instant) {
                None => return false,
                Some(txn) => temp_txn_pool.insert_op(&instant, &txn),
            }
        }
        if self.validate_transactions(&temp_txn_pool, &mut wallets) {
            for (_order, txn) in temp_txn_pool.pool.iter() {
                let hash = txn.object_hash();
                transaction_trie.put(&hash, txn.clone());
            }
            let blocks = self.blocks();
            let length = blocks.len();
            let prev_hash = blocks.get(length - 1).unwrap().object_hash();
            if prev_hash != block.prev_hash {
                return false;
            }
            if length != block.id {
                return false;
            }
            if wallets.object_hash() != block.header.get("state_trie").unwrap().clone() {
                return false;
            }
            if transaction_trie.object_hash()
                != block.header.get("transaction_trie").unwrap().clone()
            {
                return false;
            }
            if storage_trie.object_hash() != block.header.get("storage_trie").unwrap().clone() {
                return false;
            }
            return true;
        } else {
            return false;
        }
    }

    pub fn validate_transactions(
        &self,
        txn_pool: &TransactionPool,
        wallet: &mut RefMut<ProofMapIndex<T, String, Wallet>>,
    ) -> bool {
        for (_order, txn) in txn_pool.pool.iter() {
            if txn.validate() {
                if wallet.contains(&txn.txn.from) {
                    let mut from_wallet = wallet.get(&txn.txn.from).unwrap();
                    if from_wallet.get_balance() > txn.txn.amount {
                        if wallet.contains(&txn.txn.to) {
                            let mut to_wallet = wallet.get(&txn.txn.to).unwrap();
                            to_wallet.set_balance(to_wallet.get_balance() + txn.txn.amount);
                            wallet.put(&txn.txn.to.clone(), to_wallet);
                        } else {
                            let mut to_wallet = Wallet::new();
                            to_wallet.set_balance(txn.txn.amount);
                            wallet.put(&txn.txn.to.clone(), to_wallet);
                        }
                        from_wallet.set_balance(from_wallet.get_balance() - txn.txn.amount);
                        wallet.put(&txn.txn.from.clone(), from_wallet);
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
        use db::db_layer::{fork_db, patch_db};
        use std::time::Instant;
        let mut secret =
            hex::decode("97ba6f71a5311c4986e01798d525d0da8ee5c54acbf6ef7c3fadd1e2f624442f")
                .expect("invalid secret");
        let keypair = Keypair::generate_from(secret.as_mut_slice());
        let public_key =
            String::from("2c8a35450e1d198e3834d933a35962600c33d1d0f8f6481d6e08f140791374d0");
        let fork = fork_db();
        // put genesis blockin database
        {
            let mut blocks = SchemaFork::new(&fork).blocks();
            if blocks.len() == 0 {
                let block = Block::genesis_block();
                let signature = block.sign(&keypair);
                let genesis_block: SignedBlock = SignedBlock::create_block(block, signature);
                blocks.push(genesis_block);
            } else {
                println!("database exists");
            }
        }
        patch_db(fork);
        let fork = fork_db();
        {
            let mut wallets = SchemaFork::new(&fork).state();
            for (_key, _value) in wallets.iter() {
                println!("{} {:?} ", _key, _value);
            }
            if wallets.contains(&public_key) {
                let alice_wallet = wallets.get(&public_key).unwrap();
                println!("{:?}", alice_wallet);
            } else {
                let mut alice_wallet: Wallet = Wallet::new();
                alice_wallet.set_balance(1100);
                wallets.put(&public_key, alice_wallet.clone());
            }
        }
        println!("block proposal testing");
        patch_db(fork);
        let fork = fork_db();
        {
            let mut txn_pool = TransactionPool::new();
            for _ in 1..10 {
                let instant = Instant::now();
                txn_pool.insert_op(&instant, &SignedTransaction::generate(&keypair));
            }
            let schema = SchemaFork::new(&fork);
            let block = schema.create_block(&keypair, &mut txn_pool);
            println!("{:?}", block);
        }
        // patch_db(fork);
    }
}
