extern crate futures;

use crate::cli_config;
use awc::Client;
use bytes::Bytes;
use exonum_crypto::Hash;
use futures::*;
use schema::block::SignedBlock;
use schema::transaction::{SignedTransaction, State};
use utils::serializer::{deserialize, serialize};

pub struct ClientObj {
    client: Client,
    url: String,
}

impl ClientObj {
    pub fn new() -> ClientObj {
        let cli_configuration: &cli_config::Configuration = &cli_config::GLOBAL_CONFIG;
        std::env::set_var("RUST_BACKTRACE", "1");
        ClientObj {
            client: Client::default(),
            url: cli_configuration.url.clone(),
        }
    }

    // request to peer to fetch pending transaction
    pub async fn fetch_pending_transaction(&self, txn_hash: &Hash) -> Option<SignedTransaction> {
        let mut url: String = self.url.clone();
        url.extend("client/fetch_pending_transaction".chars());
        let result = self
            .client
            .get(url) // <- Create request builder
            .header("User-Agent", "Actix-web")
            //.send() // <- Send http request
            .send_body(Bytes::from(serialize(txn_hash)))
            .await
            .map_err(|_| ());
        match result {
            Ok(mut response) => {
                let resp_body = response.body();
                println!("fetch_pending_transaction Status: {:?}", response.status());
                if response.status() == 200 {
                    match resp_body.await {
                        Ok(txnbody) => {
                            let fetched_txn: SignedTransaction = deserialize(&txnbody);
                            return Some(fetched_txn);
                        }
                        Err(_) => return None,
                    }
                }
                return None;
            }
            Err(_) => return None,
        }
    }

    // request to peer to fetch public_address state
    pub async fn fetch_state(&self, public_address: &String) -> Option<State> {
        let mut url: String = self.url.clone();
        url.extend("client/fetch_state".chars());
        let result = self
            .client
            .get(url) // <- Create request builder
            .header("User-Agent", "Actix-web")
            //.send() // <- Send http request
            .send_body(Bytes::from(serialize(public_address)))
            .await
            .map_err(|_| ());
        match result {
            Ok(mut response) => {
                let resp_body = response.body();
                println!("fetch_state Status: {:?}", response.status());
                if response.status() == 200 {
                    match resp_body.await {
                        Ok(serialized_state) => {
                            let state: State = deserialize(&serialized_state);
                            return Some(state);
                        }
                        Err(_) => return None,
                    }
                }
                return None;
            }
            Err(_) => return None,
        }
    }

    // request to peer to fetch block
    pub async fn fetch_block(&self, block_index: &u64) -> Option<SignedBlock> {
        let mut url: String = self.url.clone();
        url.extend("peer/fetch_block".chars());
        let result = self
            .client
            .get(url) // <- Create request builder
            .header("User-Agent", "Actix-web")
            //.send() // <- Send http request
            .send_body(Bytes::from(serialize(block_index)))
            .await
            .map_err(|_| ());
        match result {
            Ok(mut response) => {
                let resp_body = response.body();
                println!("fetch_block Status: {:?}", response.status());
                if response.status() == 200 {
                    match resp_body.await {
                        Ok(state) => {
                            let fetched_block: SignedBlock = deserialize(&state);
                            return Some(fetched_block);
                        }
                        Err(_) => return None,
                    }
                }
                return None;
            }
            Err(_) => return None,
        }
    }

    // request to peer to fetch confirmed transaction
    pub async fn fetch_confirm_transaction(&self, txn_hash: &Hash) -> Option<SignedTransaction> {
        let mut url: String = self.url.clone();
        url.extend("client/fetch_confirm_transaction".chars());
        let result = self
            .client
            .get(url) // <- Create request builder
            .header("User-Agent", "Actix-web")
            //.send() // <- Send http request
            .send_body(Bytes::from(serialize(txn_hash)))
            .await
            .map_err(|_| ());
        match result {
            Ok(mut response) => {
                let resp_body = response.body();
                println!("fetch_confirm_transaction Status: {:?}", response.status());
                if response.status() == 200 {
                    match resp_body.await {
                        Ok(txnbody) => {
                            let fetched_txn: SignedTransaction = deserialize(&txnbody);
                            return Some(fetched_txn);
                        }
                        Err(_) => return None,
                    }
                }
                return None;
            }
            Err(_) => return None,
        }
    }

    // request for fetching latest block
    pub async fn fetch_latest_block(&self) -> Option<SignedBlock> {
        let mut url: String = self.url.clone();
        url.extend("peer/fetch_latest_block".chars());
        let result = self
            .client
            .get(url) // <- Create request builder
            .header("User-Agent", "Actix-web")
            //.send() // <- Send http request
            .send()
            .await
            .map_err(|_| ());
        match result {
            Ok(mut response) => {
                let resp_body = response.body();
                println!("fetch_block Status: {:?}", response.status());
                if response.status() == 200 {
                    match resp_body.await {
                        Ok(state) => {
                            let fetched_block: SignedBlock = deserialize(&state);
                            return Some(fetched_block);
                        }
                        Err(_) => return None,
                    }
                }
                return None;
            }
            Err(_) => return None,
        }
    }

    // pub fn fetch(&self, url: String, block_index: &u64) -> Option<SignedBlock>{
    //     let mut url: String = url;
    //     url.extend("peer/fetch_block".chars());
    //     let result = self.client
    //             .get(url) // <- Create request builder
    //             .header("User-Agent", "Actix-web")
    //             .send_body(Bytes::from(serialize(block_index)));
    //     let result = executor::block_on(result);
    //     match result {
    //         Ok(mut response) => {
    //             let resp_body = response.body();
    //             println!("fetch_block Status: {:?}", response.status());
    //             if response.status() == 200 {
    //                 match resp_body {
    //                     Ok(state) => {
    //                         let fetched_block: SignedBlock = deserialize(&state);
    //                         return Some(fetched_block);
    //                     }
    //                     Err(_) => return None,
    //                 }
    //             }
    //             return None;
    //         }
    //         Err(_) => return None,
    //     }
    // }
}
