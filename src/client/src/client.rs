use exonum_crypto::Hash;
use reqwest::{Client, Error};
use schema::block::SignedBlock;
use schema::signed_transaction::SignedTransaction;
use schema::state::State;
use std::collections::HashMap;
use std::net::IpAddr;
use utils::global_peer_data::{PeerData, GLOBALDATA};
use utils::serializer::{deserialize, serialize};

fn get_peer_url() -> Option<String> {
    let locked_peer_map = GLOBALDATA.lock().unwrap();
    for (_, peer_data) in locked_peer_map.peers.iter() {
        let peer: PeerData = peer_data.clone();
        match peer.get_network_addr() {
            Ok(ip_address) => {
                let ip_string: String = match ip_address {
                    IpAddr::V4(ip) => ip.to_string(),
                    IpAddr::V6(ip) => ip.to_string(),
                };
                let mut url: String = String::from("http://");
                url.extend(ip_string.chars());
                url.extend(":8089/".chars());
                return Some(url);
            }
            Err(_) => {}
        }
    }
    None
}

pub struct ClientObj {
    client: Client,
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
        std::env::set_var("RUST_BACKTRACE", "1");
        ClientObj {
            client: Client::new(),
        }
    }

    // request to peer to fetch pending transaction
    pub fn fetch_pending_transaction(
        &self,
        txn_hash: &Hash,
    ) -> Result<Option<SignedTransaction>, Error> {
        let mut url: String = match get_peer_url() {
            Some(url) => url,
            None => return Ok(None),
        };
        url.extend("client/fetch_pending_transaction".chars());
        let response = self
            .client
            .get(&url) // <- Create request builder
            .header("User-Agent", "Actix-web")
            //.send() // <- Send http request
            .body(serialize(txn_hash))
            .send()?;
        match response.error_for_status() {
            Ok(mut body) => {
                let mut buf: Vec<u8> = vec![];
                body.copy_to(&mut buf)?;
                let signed_transaction: SignedTransaction = deserialize(buf.as_slice());
                Ok(Some(signed_transaction))
            }
            Err(err) => {
                return Result::Err(err);
            }
        }
    }

    // request to peer to fetch public_address state
    pub fn fetch_state(&self, public_address: &String) -> Result<Option<State>, Error> {
        let mut url: String = match get_peer_url() {
            Some(url) => url,
            None => return Ok(None),
        };
        url.extend("client/fetch_state".chars());
        let response = self
            .client
            .get(&url) // <- Create request builder
            .header("User-Agent", "Actix-web")
            .body(serialize(public_address))
            .send()?;
        match response.error_for_status() {
            Ok(mut body) => {
                let mut buf: Vec<u8> = vec![];
                body.copy_to(&mut buf)?;
                let state: State = deserialize(buf.as_slice());
                Ok(Some(state))
            }
            Err(err) => {
                return Result::Err(err);
            }
        }
    }

    // request to peer to fetch block
    pub fn fetch_block(&self, block_index: &u64) -> Result<Option<SignedBlock>, Error> {
        let mut url: String = match get_peer_url() {
            Some(url) => url,
            None => return Ok(None),
        };
        url.extend("peer/fetch_block".chars());
        let response = self
            .client
            .get(&url) // <- Create request builder
            .header("User-Agent", "Actix-web")
            .body(serialize(block_index))
            .send()?;
        match response.error_for_status() {
            Ok(mut body) => {
                let mut buf: Vec<u8> = vec![];
                body.copy_to(&mut buf)?;
                let signed_block: SignedBlock = deserialize(buf.as_slice());
                Ok(Some(signed_block))
            }
            Err(err) => {
                return Result::Err(err);
            }
        }
    }

    // request to peer to fetch confirmed transaction
    pub fn fetch_confirm_transaction(
        &self,
        txn_hash: &Hash,
    ) -> Result<Option<SignedTransaction>, Error> {
        let mut url: String = match get_peer_url() {
            Some(url) => url,
            None => return Ok(None),
        };
        url.extend("client/fetch_confirm_transaction".chars());
        let response = self
            .client
            .get(&url) // <- Create request builder
            .header("User-Agent", "Actix-web")
            .body(serialize(txn_hash)) // <- Send http request
            .send()?;
        match response.error_for_status() {
            Ok(mut body) => {
                let mut buf: Vec<u8> = vec![];
                body.copy_to(&mut buf)?;
                let signed_transaction: SignedTransaction = deserialize(buf.as_slice());
                Ok(Some(signed_transaction))
            }
            Err(err) => {
                return Result::Err(err);
            }
        }
    }

    // request for fetching latest block
    pub fn fetch_latest_block(&self) -> Result<Option<SignedBlock>, Error> {
        let mut url: String = match get_peer_url() {
            Some(url) => url,
            None => return Ok(None),
        };
        url.extend("peer/fetch_latest_block".chars());
        let response = self
            .client
            .get(&url) // <- Create request builder
            .header("User-Agent", "Actix-web")
            //.send() // <- Send http request
            .send()?;
        match response.error_for_status() {
            Ok(mut body) => {
                let mut buf: Vec<u8> = vec![];
                body.copy_to(&mut buf)?;
                let signed_block: SignedBlock = deserialize(buf.as_slice());
                Ok(Some(signed_block))
            }
            Err(err) => {
                return Result::Err(err);
            }
        }
    }

    // request for fetching latest block
    pub fn fetch_blockchain_length(&self) -> Result<u64, Error> {
        let mut url: String = match get_peer_url() {
            Some(url) => url,
            None => return Ok(0),
        };
        url.extend("peer/fetch_blockchain_length".chars());
        let response = self
            .client
            .get(&url) // <- Create request builder
            .header("User-Agent", "Actix-web")
            //.send() // <- Send http request
            .send()?;
        match response.error_for_status() {
            Ok(mut body) => {
                let mut buf: Vec<u8> = vec![];
                body.copy_to(&mut buf)?;
                let length: u64 = deserialize(buf.as_slice());
                Ok(length)
            }
            Err(err) => {
                // asserting a 400 as an example
                // it could be any status between 400...599
                return Result::Err(err);
            }
        }
    }

    /// this function will sync blockchain state with other peers
    pub fn fetch_sync_state(&self, current_length: u64) -> SyncState {
        let mut block_pool: HashMap<u64, SignedBlock> = HashMap::new();
        let mut txn_map: HashMap<Hash, SignedTransaction> = HashMap::new();
        let mut own_chain_length = current_length;
        // let block_threads_vec = vec![];
        info!("sync-state function called");
        let mut fetch_flag: bool = true;
        let is_blockchain_length: Result<u64, Error> = self.fetch_blockchain_length();
        let blockchain_length: u64 = match is_blockchain_length {
            Ok(length) => length,
            Err(_) => return SyncState::new(),
        };
        if blockchain_length == 0 {
            return SyncState::new();
        }
        while own_chain_length < blockchain_length {
            let block: Result<Option<SignedBlock>, Error> = self.fetch_block(&own_chain_length);
            match block {
                Ok(is_signed_block) => {
                    match is_signed_block {
                        Some(signed_block) => {
                            block_pool.insert(own_chain_length.clone(), signed_block);
                            own_chain_length = own_chain_length + 1;
                        }
                        None => own_chain_length = blockchain_length,
                    };
                }
                // no point in fetching higher block since lower is missing.
                Err(_) => own_chain_length = blockchain_length,
            }
        }
        info!("Block fetched -> {:#?}", block_pool.len());
        while fetch_flag {
            for (_key, value) in block_pool.iter() {
                for each in value.block.txn_pool.iter() {
                    let fetch_txn_output: Result<Option<SignedTransaction>, Error> =
                        self.fetch_confirm_transaction(each);
                    match fetch_txn_output {
                        Ok(is_txn) => {
                            match is_txn {
                                Some(txn) => {
                                    txn_map.insert(each.clone(), txn);
                                }
                                None => fetch_flag = false,
                            };
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
        info!("Sync_State --All data fetched");
        return SyncState::new_from(blockchain_length, block_pool, txn_map);
    }
}
