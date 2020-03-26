extern crate schema;
extern crate utils;

use exonum_crypto::Hash;
use exonum_derive::FromAccess;
use exonum_merkledb::{
    access::{Access, FromAccess, RawAccessMut},
    ListIndex, ObjectHash, ProofMapIndex,
};
use generic_traits::traits::PoolTrait;
use schema::block::{Block, BlockTraits, SignedBlock, SignedBlockTraits};
use schema::transaction::{SignedTransaction, State};
use schema::transaction_pool::{TransactionPool, POOL};
use utils::keypair::{CryptoKeypair, Keypair, KeypairType, PublicKey, Verify};
use utils::serializer::serialize;

#[derive(FromAccess)]
pub struct SchemaFork<T: Access> {
    txn_trie: ProofMapIndex<T::Base, Hash, SignedTransaction>,
    block_list: ListIndex<T::Base, SignedBlock>,
    state_trie: ProofMapIndex<T::Base, String, State>,
    storage_trie: ProofMapIndex<T::Base, Hash, SignedTransaction>,
}

impl<T: Access> SchemaFork<T> {
    pub fn new(access: T) -> Self {
        Self::from_root(access).unwrap()
    }
}

impl<T: Access> SchemaFork<T>
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

    pub fn initialize_db(&mut self, kp: &KeypairType) -> (SignedBlock, Vec<SignedTransaction>) {
        self.state_trie.clear();
        self.txn_trie.clear();
        self.storage_trie.clear();
        self.block_list.clear();
        let mut block = Block::genesis_block();
        let public_key = hex::encode(Keypair::public(&kp).encode());
        block.peer_id = public_key.clone();
        block.header[0] = self.state_trie_merkle_hash();
        block.header[1] = self.storage_trie_merkle_hash();
        block.header[2] = self.txn_trie_merkle_hash();
        let signature = block.sign(kp);
        let genesis_block: SignedBlock = SignedBlock::create_block(block, signature);
        self.block_list.push(genesis_block.clone());
        return (genesis_block, vec![]);
    }

    /**
     * this function will iterate over txn_order_pool and return a vec of SignedTransaction and
     * all changes due to these transaction also updated in state_trie
     * TODO: // since fxn iterate over txnz-order_pool, so in case of invalid txn or expired txn will not be
     * deleted from txn_pool according to whole txn_pool
     * Update logic for that in future.  
     */
    pub fn execute_transactions(&mut self, txn_pool: &TransactionPool) -> Vec<Hash> {
        let txn_pool_as_trait = txn_pool as &dyn PoolTrait<T, State, SignedTransaction>;
        txn_pool_as_trait.execute_transactions(&mut self.state_trie, &mut self.txn_trie)
    }

    /// this function only will called when the node willing to propose block and for that agree to compute block
    pub fn create_block(&mut self, kp: &KeypairType) -> SignedBlock {
        // all trie's state before current block computation
        #[allow(unused_assignments)]
        let mut executed_txns: Vec<Hash> = vec![];
        {
            let mut txn_pool = POOL.pool.lock().unwrap();
            executed_txns = self.execute_transactions(&mut txn_pool);
        }
        // println!(
        //     "length {:?} {:?}",
        //     txn_pool.length_hash_pool(),
        //     txn_pool.length_order_pool()
        // );
        println!("txn count in proposed block {}", executed_txns.len());
        let length = self.block_list.len();
        let last_block: SignedBlock = self.block_list.get(length - 1).unwrap();
        // println!("{:?}", last_block);
        let prev_hash = last_block.object_hash();
        let header: [Hash; 3] = [
            self.state_trie_merkle_hash(),
            self.storage_trie_merkle_hash(),
            self.txn_trie_merkle_hash(),
        ];
        // updated merkle root of all tries
        let public_key = hex::encode(Keypair::public(&kp).encode());
        let block = Block::new_block(length, public_key, prev_hash, executed_txns, header);
        let signature: Vec<u8> = block.sign(kp);
        let signed_block: SignedBlock = SignedBlock::create_block(block, signature);
        self.block_list.push(signed_block.clone());
        signed_block
    }

    /// this function will update state_trie for given transaction
    pub fn update_transactions(
        &mut self,
        txn_pool: &TransactionPool,
        hash_vec: &Vec<Hash>,
    ) -> bool {
        let txn_pool_as_trait = txn_pool as &dyn PoolTrait<T, State, SignedTransaction>;
        txn_pool_as_trait.update_transactions(&mut self.state_trie, &mut self.txn_trie, hash_vec)
    }

    /// this function will update fork for given block
    pub fn update_block(&mut self, signed_block: &SignedBlock) -> bool {
        let length = self.block_list.len();
        // block height check
        if signed_block.block.id != length {
            eprintln!(
                "block length error block height {} blockchain height {}",
                signed_block.block.id, length
            );
            return false;
        }

        // block signature check
        let msg = serialize(&signed_block.block);
        if !PublicKey::verify_from_encoded_pk(
            &signed_block.block.peer_id,
            &msg,
            &signed_block.signature,
        ) {
            eprintln!("block signature couldn't verified");
            return false;
        }

        // genesis block check
        if signed_block.block.id == 0 {
            // let executed_txns = &signed_block.block.txn_pool;
            // for each in executed_txns.iter() {
            //     let signed_txn = txn_pool.get(each);
            //     if let Some(txn) = signed_txn {
            //         self.txn_trie.put(each, txn.clone());
            //         self.execute_genesis_transactions(&txn);
            //     } else {
            //         eprintln!("block transaction execution error");
            //         return false;
            //     }
            // }
            let header: [Hash; 3] = [
                self.state_trie_merkle_hash(),
                self.storage_trie_merkle_hash(),
                self.txn_trie_merkle_hash(),
            ];
            if header[0] != signed_block.block.header[0] {
                eprintln!("block header state_trie merkle root error");
                return false;
            }
            if header[1] != signed_block.block.header[1] {
                eprintln!("block header storage_trie merkle root error");
                return false;
            }
            if header[2] != signed_block.block.header[2] {
                eprintln!("block header transaction_trie merkle root error");
                return false;
            }
            self.block_list.push(signed_block.clone());
            return true;
        } else {
            // block pre_hash check
            let last_block: SignedBlock = self.block_list.get(length - 1).unwrap();
            let prev_hash = last_block.object_hash();
            if signed_block.block.prev_hash != prev_hash {
                eprintln!(
                    "block prev_hash error block prev_hash {}, blockchain root {}",
                    signed_block.block.prev_hash, prev_hash
                );
                return false;
            }

            // block txn pool validation
            {
                let txn_pool = POOL.pool.lock().unwrap();
                if !self.update_transactions(&txn_pool, &signed_block.block.txn_pool) {
                    eprintln!("block txn_pool couldn't updated, block declined");
                    return false;
                }
            }

            // block header check
            let header: [Hash; 3] = [
                self.state_trie_merkle_hash(),
                self.storage_trie_merkle_hash(),
                self.txn_trie_merkle_hash(),
            ];
            if header[0] != signed_block.block.header[0] {
                eprintln!("block header state_trie merkle root error");
                return false;
            }
            if header[1] != signed_block.block.header[1] {
                eprintln!("block header storage_trie merkle root error");
                return false;
            }
            if header[2] != signed_block.block.header[2] {
                eprintln!("block header transaction_trie merkle root error");
                return false;
            }
            self.block_list.push(signed_block.clone());
            return true;
        }
    }
}

#[cfg(test)]
mod test_db_service {

    #[test]
    pub fn test_schema() {
        use super::*;
        use crate::db_layer::{fork_db, patch_db};
        use generic_traits::traits::TransactionTrait;
        use schema::transaction_pool::TxnPool;
        use std::time::SystemTime;
        let mut secret =
            hex::decode("97ba6f71a5311c4986e01798d525d0da8ee5c54acbf6ef7c3fadd1e2f624442f")
                .expect("invalid secret");
        let keypair = Keypair::generate_from(secret.as_mut_slice());
        let _public_key =
            String::from("2c8a35450e1d198e3834d933a35962600c33d1d0f8f6481d6e08f140791374d0");
        let fork = fork_db();
        // put genesis blockin database
        {
            let mut schema = SchemaFork::new(&fork);
            schema.initialize_db(&keypair);
        }
        patch_db(fork);
        println!("block proposal testing");
        let fork = fork_db();
        {
            for _ in 1..10 {
                let time_instant = SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap()
                    .as_micros();
                POOL.insert_op(&time_instant, &SignedTransaction::generate(&keypair));
            }
            let mut schema = SchemaFork::new(&fork);
            let block = schema.create_block(&keypair);
            println!("{:?}", block);
        }
    }
}
