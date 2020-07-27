extern crate client;
extern crate schema;
extern crate utils;

use super::db_layer::fork_db;
use client::client::{ClientObj, SyncState};
use exonum_crypto::Hash;
use exonum_derive::FromAccess;
use exonum_merkledb::{
    access::{Access, FromAccess, RawAccessMut},
    Fork, ListIndex, ObjectHash, ProofMapIndex,
};
use schema::block::{Block, BlockTraits, SignedBlock};
use schema::signed_transaction::SignedTransaction;
use schema::state::State;
use schema::transaction_pool::{
    TransactionPool, TransactionPoolTraits, TxnPool, TxnPoolKeyType, POOL,
};
use sdk::traits::{PoolTrait, StateContext};
use std::time::SystemTime;
use utils::configreader;
use utils::configreader::BlockConfig;
use utils::keypair::{CryptoKeypair, Keypair, KeypairType};

#[derive(FromAccess)]
pub struct SchemaFork<T: Access> {
    txn_trie: ProofMapIndex<T::Base, Hash, SignedTransaction>,
    block_list: ListIndex<T::Base, SignedBlock>,
    state_trie: ProofMapIndex<T::Base, String, State>,
    storage_trie: ProofMapIndex<T::Base, Hash, SignedTransaction>,
}

impl<T: Access> SchemaFork<T> {
    pub fn new(access: T) -> Self {
        match Self::from_root(access) {
            Result::Ok(root_access) => return root_access,
            Result::Err(error) => {
                error!("fork access error {:?}", error);
                panic!("{:?}", error);
            }
        }
    }
}

impl<T: Access> StateContext for SchemaFork<T>
where
    T::Base: RawAccessMut,
{
    fn put(&mut self, key: &String, state: State) {
        self.state_trie.put(key, state);
    }
    fn get(&self, key: &String) -> Option<State> {
        self.state_trie.get(key)
    }
    fn contains(&self, key: &String) -> bool {
        self.state_trie.contains(key)
    }
    fn put_txn(&mut self, key: &Hash, txn: SignedTransaction) {
        self.txn_trie.put(key, txn);
    }
    fn get_txn(&self, key: &Hash) -> Option<SignedTransaction> {
        self.txn_trie.get(key)
    }
    fn contains_txn(&self, key: &Hash) -> bool {
        self.txn_trie.contains(key)
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

    pub fn blockchain_length(&self) -> u64 {
        self.block_list.len()
    }

    pub fn initialize_db(&mut self, custom_headers: Vec<u8>, timestamp: u128) -> SignedBlock {
        self.state_trie.clear();
        self.txn_trie.clear();
        self.storage_trie.clear();
        self.block_list.clear();
        let mut block = Block::genesis_block(custom_headers, timestamp);
        block.header[0] = self.state_trie_merkle_hash();
        block.header[1] = self.storage_trie_merkle_hash();
        block.header[2] = self.txn_trie_merkle_hash();
        let signature: Vec<u8> = Vec::new();
        let sign_headers: Vec<u8> = Vec::new();
        let genesis_block: SignedBlock = SignedBlock::create_block(block, signature, sign_headers);
        self.block_list.push(genesis_block.clone());
        return genesis_block;
    }

    /**
     * this function will iterate over txn_order_pool and return a vec of SignedTransaction and
     * all changes due to these transaction also updated in state_trie
     */
    pub fn execute_transactions(&mut self, txn_pool: &mut TransactionPool) -> Vec<Hash> {
        let txn_pool_as_trait = txn_pool as &dyn PoolTrait<T, State, SignedTransaction>;
        let state_context = self as &mut dyn StateContext;
        let (executed_txns, unknown_app_txns_hash) =
            txn_pool_as_trait.execute_transactions(state_context);
        txn_pool.sync_pool(&unknown_app_txns_hash);
        executed_txns
    }

    /// this function only will called when the node willing to propose block and for that agree to compute block
    pub fn create_block(&mut self, kp: &KeypairType, custom_headers: Vec<u8>) -> SignedBlock {
        // all trie's state before current block computation
        #[allow(unused_assignments)]
        let mut executed_txns: Vec<Hash> = vec![];
        {
            let mut txn_pool = POOL.pool.lock().unwrap();
            executed_txns = self.execute_transactions(&mut txn_pool);
        }
        info!("txn count in proposed block {}", executed_txns.len());
        let length = self.block_list.len();
        let last_block: SignedBlock = match self.block_list.get(length - 1) {
            Some(block) => block,
            None => {
                error!("last block not found");
                panic!("last block not found");
            }
        };
        let prev_hash = last_block.object_hash();
        let header: [Hash; 3] = [
            self.state_trie_merkle_hash(),
            self.storage_trie_merkle_hash(),
            self.txn_trie_merkle_hash(),
        ];
        // updated merkle root of all tries
        let public_key = hex::encode(Keypair::public(&kp).encode());
        let block = Block::new_block(
            length,
            public_key,
            prev_hash,
            executed_txns,
            header,
            custom_headers,
        );
        let signature: Vec<u8> = block.sign(kp);
        let auth_headers: Vec<u8> = Vec::new();
        let signed_block: SignedBlock = SignedBlock::create_block(block, signature, auth_headers);
        self.block_list.push(signed_block.clone());
        signed_block
    }

    pub fn forge_new_block(
        &self,
        kp: &KeypairType,
        custom_headers: Vec<u8>,
    ) -> (Fork, SignedBlock) {
        let block_config: &BlockConfig = &configreader::GLOBAL_CONFIG.block_config;
        let mut timestamp: TxnPoolKeyType = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_micros();
        timestamp = timestamp + block_config.block_creation_time_limit;
        let mut current_timestamp: TxnPoolKeyType = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_micros();
        #[allow(unused_assignments)]
        let mut fork_instance: Fork = fork_db();
        // dummy signed block
        let mut block_instance: SignedBlock =
            SignedBlock::create_block(Block::genesis_block(Vec::new(), 0), Vec::new(), Vec::new());
        while current_timestamp < timestamp {
            {
                let mut schema = SchemaFork::new(&fork_instance);
                block_instance = schema.create_block(kp, custom_headers.clone());
            }
            if block_instance.block.txn_pool.len() >= block_config.block_transaction_limit as usize
            {
                current_timestamp = timestamp;
            } else {
                current_timestamp = SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap()
                    .as_micros();
                if current_timestamp < timestamp {
                    fork_instance = fork_db();
                }
            }
            let sleep_time: u64 = block_config.block_creation_time_limit as u64 / 10;
            std::thread::sleep(std::time::Duration::from_micros(sleep_time));
        }
        (fork_instance, block_instance)
    }

    /// this function will update state_trie for given transaction
    pub fn update_transactions(
        &mut self,
        txn_pool: &TransactionPool,
        hash_vec: &Vec<Hash>,
    ) -> bool {
        let txn_pool_as_trait = txn_pool as &dyn PoolTrait<T, State, SignedTransaction>;
        let state_context = self as &mut dyn StateContext;
        txn_pool_as_trait.update_transactions(state_context, hash_vec)
    }

    /// this function will update fork for given block
    pub fn update_block(&mut self, signed_block: &SignedBlock) -> bool {
        let length = self.block_list.len();
        // block height check
        if signed_block.block.id != length {
            error!(
                "block length error block height {} blockchain height {}",
                signed_block.block.id, length
            );
            return false;
        }

        // genesis block check
        if signed_block.block.id == 0 {
            let header: [Hash; 3] = [
                self.state_trie_merkle_hash(),
                self.storage_trie_merkle_hash(),
                self.txn_trie_merkle_hash(),
            ];
            if header[0] != signed_block.block.header[0] {
                error!("block header state_trie merkle root error");
                return false;
            }
            if header[1] != signed_block.block.header[1] {
                error!("block header storage_trie merkle root error");
                return false;
            }
            if header[2] != signed_block.block.header[2] {
                error!("block header transaction_trie merkle root error");
                return false;
            }
            self.block_list.push(signed_block.clone());
            return true;
        } else {
            // block pre_hash check
            let last_block: SignedBlock = match self.block_list.get(length - 1) {
                Some(block) => block,
                None => return false,
            };
            let prev_hash = last_block.object_hash();
            if signed_block.block.prev_hash != prev_hash {
                error!(
                    "block prev_hash error block prev_hash {}, blockchain root {}",
                    signed_block.block.prev_hash, prev_hash
                );
                return false;
            }

            // block signature check
            if !signed_block.validate() {
                error!("block signature couldn't verified");
                return false;
            }

            // check all transactions are present or not in the POOL
            let client_obj: ClientObj = ClientObj::new();
            let pk: String = signed_block.block.peer_id.clone();
            if !client_obj.sync_txn_pool(pk, &signed_block.block.txn_pool) {
                error!("some transactions are missing from the txn_pool, block declined");
                return false;
            }

            // block txn pool validation
            {
                let txn_pool = POOL.pool.lock().unwrap();
                if !self.update_transactions(&txn_pool, &signed_block.block.txn_pool) {
                    error!("block txn_pool couldn't updated, block declined");
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
                error!("block header state_trie merkle root error");
                return false;
            }
            if header[1] != signed_block.block.header[1] {
                error!("block header storage_trie merkle root error");
                return false;
            }
            if header[2] != signed_block.block.header[2] {
                error!("block header transaction_trie merkle root error");
                return false;
            }
            self.block_list.push(signed_block.clone());
            return true;
        }
    }

    /// this function will sync blockchain state with other peers
    pub fn sync_state(&mut self) -> bool {
        let client_instance = ClientObj::new();
        let mut own_chain_length = self.block_list.len();
        // let block_threads_vec = vec![];
        #[allow(unused_assignments)]
        let mut block_fetch_flag: bool = true;
        let sync_data: SyncState = client_instance.fetch_sync_state(own_chain_length.clone());
        if sync_data.index == 0 {
            return false;
        }
        block_fetch_flag = true;
        while block_fetch_flag && own_chain_length < sync_data.index {
            match sync_data.block_map.get(&own_chain_length) {
                Some(signed_block) => {
                    let signed_block: &SignedBlock = &signed_block;
                    for each in signed_block.block.txn_pool.iter() {
                        if let Some(txn) = sync_data.txn_map.get(each) {
                            match txn.header.get(&String::from("timestamp")) {
                                Some(string) => {
                                    match string.parse::<TxnPoolKeyType>() {
                                        Ok(timestamp) => POOL.insert_op(&timestamp, &txn),
                                        Err(error) => {
                                            block_fetch_flag = false;
                                            error!("transaction timestamp error {:?}", error);
                                        }
                                    };
                                }
                                None => block_fetch_flag = false,
                            }
                        } else {
                            block_fetch_flag = false;
                        }
                    }
                    if block_fetch_flag {
                        if self.update_block(signed_block) {
                            own_chain_length = own_chain_length + 1;
                        } else {
                            block_fetch_flag = false;
                        }
                    }
                    POOL.sync_pool(&signed_block.block.txn_pool);
                }
                None => {
                    own_chain_length = sync_data.index;
                    block_fetch_flag = false;
                }
            }
        }
        return true;
    }
}
