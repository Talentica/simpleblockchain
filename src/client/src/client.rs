use crate::cli_config;
use exonum_crypto::Hash;
use reqwest::{Client, Error};
use schema::block::SignedBlock;
use schema::transaction::{SignedTransaction, State};
use std::collections::HashMap;
use utils::serializer::{deserialize, serialize};

pub struct ClientObj {
    client: Client,
    url: String,
}

#[derive(Debug)]
pub struct SyncState {
    pub index: u64,
    pub block_map: HashMap<u64, SignedBlock>,
    pub txn_map: HashMap<Hash, SignedTransaction>,
}

impl SyncState {
    pub fn new() -> SyncState {
        SyncState {
            index: 0,
            block_map: HashMap::new(),
            txn_map: HashMap::new(),
        }
    }

    pub fn new_from(
        index: u64,
        block_map: HashMap<u64, SignedBlock>,
        txn_map: HashMap<Hash, SignedTransaction>,
    ) -> SyncState {
        SyncState {
            index,
            block_map,
            txn_map,
        }
    }
}

impl ClientObj {
    pub fn new() -> ClientObj {
        let cli_configuration: &cli_config::Configuration = &cli_config::GLOBAL_CONFIG;
        std::env::set_var("RUST_BACKTRACE", "1");
        ClientObj {
            client: Client::new(),
            url: cli_configuration.url.clone(),
        }
    }

    // request to peer to fetch pending transaction
    pub fn fetch_pending_transaction(&self, txn_hash: &Hash) -> Result<SignedTransaction, Error> {
        let mut url: String = self.url.clone();
        url.extend("client/fetch_pending_transaction".chars());
        let mut result = self
            .client
            .get(&url) // <- Create request builder
            .header("User-Agent", "Actix-web")
            //.send() // <- Send http request
            .body(serialize(txn_hash))
            .send()
            .expect("Failed to send request");
        let mut buf: Vec<u8> = vec![];
        result.copy_to(&mut buf)?;
        let signed_transaction: SignedTransaction = deserialize(buf.as_slice());
        Ok(signed_transaction)
    }

    // request to peer to fetch public_address state
    pub fn fetch_state(&self, public_address: &String) -> Result<State, Error> {
        let mut url: String = self.url.clone();
        url.extend("client/fetch_state".chars());
        let mut result = self
            .client
            .get(&url) // <- Create request builder
            .header("User-Agent", "Actix-web")
            .body(serialize(public_address))
            .send()
            .expect("Failed to send request");
        let mut buf: Vec<u8> = vec![];
        result.copy_to(&mut buf)?;
        let state: State = deserialize(buf.as_slice());
        Ok(state)
    }

    // request to peer to fetch block
    pub fn fetch_block(&self, block_index: &u64) -> Result<SignedBlock, Error> {
        let mut url: String = self.url.clone();
        url.extend("peer/fetch_block".chars());
        let mut result = self
            .client
            .get(&url) // <- Create request builder
            .header("User-Agent", "Actix-web")
            .body(serialize(block_index))
            .send()
            .expect("Failed to send request");
        let mut buf: Vec<u8> = vec![];
        result.copy_to(&mut buf)?;
        let signed_block: SignedBlock = deserialize(buf.as_slice());
        Ok(signed_block)
    }

    // request to peer to fetch confirmed transaction
    pub fn fetch_confirm_transaction(&self, txn_hash: &Hash) -> Result<SignedTransaction, Error> {
        let mut url: String = self.url.clone();
        url.extend("client/fetch_confirm_transaction".chars());
        let mut result = self
            .client
            .get(&url) // <- Create request builder
            .header("User-Agent", "Actix-web")
            .body(serialize(txn_hash)) // <- Send http request
            .send()
            .expect("Failed to send request");
        let mut buf: Vec<u8> = vec![];
        result.copy_to(&mut buf)?;
        let signed_transaction: SignedTransaction = deserialize(buf.as_slice());
        Ok(signed_transaction)
    }

    // request for fetching latest block
    pub fn fetch_latest_block(&self) -> Result<SignedBlock, Error> {
        let mut url: String = self.url.clone();
        url.extend("peer/fetch_latest_block".chars());
        let mut result = self
            .client
            .get(&url) // <- Create request builder
            .header("User-Agent", "Actix-web")
            //.send() // <- Send http request
            .send()
            .expect("Failed to send request");
        let mut buf: Vec<u8> = vec![];
        result.copy_to(&mut buf)?;
        let signed_block: SignedBlock = deserialize(buf.as_slice());
        Ok(signed_block)
    }

    // request for fetching latest block
    pub fn fetch_blockchain_length(&self) -> Result<u64, Error> {
        let mut url: String = self.url.clone();
        url.extend("peer/fetch_blockchain_length".chars());
        let mut result = self
            .client
            .get(&url) // <- Create request builder
            .header("User-Agent", "Actix-web")
            //.send() // <- Send http request
            .send()
            .expect("Failed to send request");
        let mut buf: Vec<u8> = vec![];
        result.copy_to(&mut buf)?;
        let length: u64 = deserialize(buf.as_slice());
        Ok(length)
    }

    /// this function will sync blockchain state with other peers
    pub fn fetch_sync_state(&self, current_length: u64) -> SyncState {
        let mut txn_pool: HashMap<Hash, SignedTransaction> = HashMap::new();
        let mut block_pool: HashMap<u64, SignedBlock> = HashMap::new();
        let mut own_chain_length = current_length;
        // let block_threads_vec = vec![];
        println!("start");
        let mut fetch_flag: bool = true;
        let is_blockchain_length: Result<u64, Error> = self.fetch_blockchain_length();
        let blockchain_length: u64 = match is_blockchain_length {
            Ok(length) => length,
            Err(_) => return SyncState::new(),
        };

        while own_chain_length < blockchain_length {
            let block: Result<SignedBlock, Error> = self.fetch_block(&own_chain_length);
            match block {
                Ok(signed_block) => {
                    block_pool.insert(own_chain_length.clone(), signed_block);
                    own_chain_length = own_chain_length + 1;
                }
                // no point in fetching higher block since lower is missing.
                Err(_) => own_chain_length = blockchain_length,
            }
        }
        println!("fetched blocks count -> {:#?}", block_pool.len());
        while fetch_flag {
            for (_key, value) in block_pool.iter() {
                for each in value.block.txn_pool.iter() {
                    let fetch_txn_output: Result<SignedTransaction, Error> =
                        self.fetch_confirm_transaction(each);
                    match fetch_txn_output {
                        Ok(txn) => {
                            txn_pool.insert(each.clone(), txn);
                        }
                        Err(_) => {
                            fetch_flag = false;
                            break;
                        }
                    }
                }
                if !fetch_flag {
                    break;
                }
            }
            fetch_flag = false;
        }
        return SyncState::new_from(blockchain_length, block_pool, txn_pool);
    }
}
