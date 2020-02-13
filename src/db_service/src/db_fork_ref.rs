extern crate db;
extern crate schema;
extern crate utils;
use db::db_layer::{fork_db, patch_db};
use exonum_crypto::Hash;
use exonum_merkledb::{Fork, ListIndex, ObjectAccess, ObjectHash, ProofMapIndex, RefMut};
use schema::block::{Block, BlockTraits, SignedBlock, SignedBlockTraits};
use schema::transaction::{SignedTransaction, Txn};
use schema::transaction_pool::{TransactionPool, TxnPool};
use schema::wallet::Wallet;
use utils::keypair::{CryptoKeypair, Keypair, KeypairType, PublicKey, Verify};
use utils::serializer::serialize;

pub struct SchemaFork<T: ObjectAccess>(T);

impl<T: ObjectAccess> SchemaFork<T> {
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

    pub fn initialize_db(&self, kp: &KeypairType) -> SignedBlock {
        let mut blocks = self.blocks();
        let mut wallets = self.state();
        let mut transaction_trie = self.transactions();
        let mut storage_trie = self.storage();
        wallets.clear();
        transaction_trie.clear();
        storage_trie.clear();
        blocks.clear();
        let mut block = Block::genesis_block();
        let public_key = hex::encode(Keypair::public(&kp).encode());
        let mut alice_wallet: Wallet = Wallet::new();
        block.peer_id = public_key.clone();
        alice_wallet.add_balance(100000000);
        wallets.put(&public_key, alice_wallet.clone());
        block.header[0] = wallets.object_hash();
        block.header[1] = storage_trie.object_hash();
        block.header[2] = transaction_trie.object_hash();
        let signature = block.sign(kp);
        let genesis_block: SignedBlock = SignedBlock::create_block(block, signature);
        blocks.push(genesis_block.clone());
        return genesis_block;
    }

    /**
     * this function will iterate over txn_order_pool and return a vec of SignedTransaction and
     * all changes due to these transaction also updated in state_trie called wallet
     * TODO: // since fxn iterate over txnz-order_pool, so in case of invalid txn or expired txn will not be
     * deleted from txn_pool according to whole txn_pool
     * Update logic for that in future.  
     */
    pub fn execute_transactions(
        &self,
        txn_pool: &mut TransactionPool,
        wallet: &mut RefMut<ProofMapIndex<T, String, Wallet>>,
    ) -> Vec<SignedTransaction> {
        let mut temp_vec = Vec::<SignedTransaction>::with_capacity(15);
        // compute until order_pool exhusted or transaction limit crossed
        for (_key, value) in txn_pool.order_pool.iter() {
            if temp_vec.len() < 15 {
                let txn: SignedTransaction = value.clone();
                if txn.validate() {
                    if wallet.contains(&txn.txn.from) {
                        let mut from_wallet: Wallet = wallet.get(&txn.txn.from).unwrap();
                        if from_wallet.get_balance() > txn.txn.amount {
                            if wallet.contains(&txn.txn.to) {
                                let mut to_wallet: Wallet = wallet.get(&txn.txn.to).unwrap();
                                to_wallet.add_balance(txn.txn.amount);
                                wallet.put(&txn.txn.to.clone(), to_wallet);
                            } else {
                                let mut to_wallet = Wallet::new();
                                to_wallet.add_balance(txn.txn.amount);
                                wallet.put(&txn.txn.to.clone(), to_wallet);
                            }
                            from_wallet.deduct_balance(txn.txn.amount);
                            from_wallet.increase_nonce();
                            wallet.put(&txn.txn.from.clone(), from_wallet);
                            temp_vec.push(txn);
                        }
                    }
                }
            } else {
                break;
            }
        }
        temp_vec
    }

    /// this function only will called when the node willing to propose block and for that agree to compute block
    pub fn create_block(&self, kp: &KeypairType, txn_pool: &mut TransactionPool) -> SignedBlock {
        // all trie's state before current block computation
        let mut wallets = self.state();
        let mut transaction_trie = self.transactions();
        let storage_trie = self.storage();

        let executed_txns = self.execute_transactions(txn_pool, &mut wallets);
        println!(
            "length {:?} {:?}",
            txn_pool.length_hash_pool(),
            txn_pool.length_order_pool()
        );
        let mut vec_txn_hash = vec![];
        for each in executed_txns.iter() {
            let hash = each.object_hash();
            transaction_trie.put(&hash, each.clone());
            vec_txn_hash.push(hash);
        }
        println!("txn count in proposed block {}", vec_txn_hash.len());
        let mut blocks = self.blocks();
        let length = blocks.len();
        let last_block: SignedBlock = blocks.get(length - 1).unwrap();
        // println!("{:?}", last_block);
        let prev_hash = last_block.object_hash();
        let header: [Hash; 3] = [
            wallets.object_hash(),
            storage_trie.object_hash(),
            transaction_trie.object_hash(),
        ];
        // updated merkle root of all tries
        let public_key = hex::encode(Keypair::public(&kp).encode());
        let block = Block::new_block(length, public_key, prev_hash, vec_txn_hash, header);
        let signature: Vec<u8> = block.sign(kp);
        let signed_block: SignedBlock = SignedBlock::create_block(block, signature);
        blocks.push(signed_block.clone());
        signed_block
    }

    /// this function will update wallet for given transaction
    pub fn update_transaction(
        &self,
        txn: SignedTransaction,
        wallet: &mut RefMut<ProofMapIndex<T, String, Wallet>>,
    ) -> bool {
        if txn.validate() {
            if wallet.contains(&txn.txn.from) {
                let mut from_wallet: Wallet = wallet.get(&txn.txn.from).unwrap();
                if from_wallet.get_balance() > txn.txn.amount {
                    if wallet.contains(&txn.txn.to) {
                        let mut to_wallet: Wallet = wallet.get(&txn.txn.to).unwrap();
                        to_wallet.add_balance(txn.txn.amount);
                        wallet.put(&txn.txn.to.clone(), to_wallet);
                    } else {
                        let mut to_wallet = Wallet::new();
                        to_wallet.add_balance(txn.txn.amount);
                        wallet.put(&txn.txn.to.clone(), to_wallet);
                    }
                    from_wallet.deduct_balance(txn.txn.amount);
                    from_wallet.increase_nonce();
                    wallet.put(&txn.txn.from.clone(), from_wallet);
                    return true;
                }
            }
        }
        return false;
    }

    /// this function will update fork for given block
    pub fn update_block(&self, signed_block: &SignedBlock, txn_pool: &TransactionPool) -> bool {
        let mut wallets = self.state();
        let mut transaction_trie = self.transactions();
        let storage_trie = self.storage();
        let mut blocks = self.blocks();
        let length = blocks.len();
        // block height check
        if signed_block.block.id != length {
            return false;
        }

        // block signature check
        let msg = serialize(signed_block);
        if !PublicKey::verify_from_encoded_pk(
            &signed_block.block.peer_id,
            &msg,
            &signed_block.signature,
        ) {
            return false;
        }
        
        // genesis block check
        if signed_block.block.id == 0 {
            let mut alice_wallet: Wallet = Wallet::new();
            alice_wallet.add_balance(100000000);
            wallets.put(&signed_block.block.peer_id, alice_wallet.clone());
            let header: [Hash; 3] = [
                wallets.object_hash(),
                storage_trie.object_hash(),
                transaction_trie.object_hash(),
            ];
            if header[0] != signed_block.block.header[0] {
                return false;
            }
            if header[1] != signed_block.block.header[1] {
                return false;
            }
            if header[2] != signed_block.block.header[2] {
                return false;
            }
            blocks.push(signed_block.clone());
            return true;
        }
        else{
            // block pre_hash check
            let last_block: SignedBlock = blocks.get(length - 1).unwrap();
            let prev_hash = last_block.object_hash();
            if signed_block.block.prev_hash != prev_hash {
                return false;
            }

            // block txn pool validation
            let executed_txns = &signed_block.block.txn_pool;
            for each in executed_txns.iter() {
                let signed_txn = txn_pool.get(each);
                if let Some(txn) = signed_txn {
                    transaction_trie.put(each, txn.clone());
                    self.update_transaction(txn.clone(), &mut wallets);
                } else {
                    return false;
                }
            }

            // block header check
            let header: [Hash; 3] = [
                wallets.object_hash(),
                storage_trie.object_hash(),
                transaction_trie.object_hash(),
            ];
            if header[0] != signed_block.block.header[0] {
                return false;
            }
            if header[1] != signed_block.block.header[1] {
                return false;
            }
            if header[2] != signed_block.block.header[2] {
                return false;
            }
            blocks.push(signed_block.clone());
            return true;
        }
    }
}

#[cfg(test)]
mod test_db_service {

    #[test]
    pub fn test_schema() {
        use super::*;
        use chrono::prelude::Utc;
        let mut secret =
            hex::decode("97ba6f71a5311c4986e01798d525d0da8ee5c54acbf6ef7c3fadd1e2f624442f")
                .expect("invalid secret");
        let keypair = Keypair::generate_from(secret.as_mut_slice());
        let _public_key =
            String::from("2c8a35450e1d198e3834d933a35962600c33d1d0f8f6481d6e08f140791374d0");
        let fork = fork_db();
        // put genesis blockin database
        {
            let schema = SchemaFork::new(&fork);
            schema.initialize_db(&keypair);
        }
        patch_db(fork);
        println!("block proposal testing");
        let fork = fork_db();
        {
            let mut txn_pool = TransactionPool::new();
            for _ in 1..10 {
                let time_instant = Utc::now().timestamp_nanos();
                txn_pool.insert_op(&time_instant, &SignedTransaction::generate(&keypair));
            }
            let schema = SchemaFork::new(&fork);
            let block = schema.create_block(&keypair, &mut txn_pool);
            println!("{:?}", block);
        }
    }
}
