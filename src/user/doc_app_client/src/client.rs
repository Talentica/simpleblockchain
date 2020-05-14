extern crate futures;

use crate::cli_config::Configuration;
use crate::doc_app_types::SignedTransaction;
use crate::doc_app_types::{CryptoTransaction, DocState};
use awc::Client;
use bytes::Bytes;
use exonum_crypto::Hash;
use sdk::state::State;
use utils::crypto::keypair::{CryptoKeypair, Keypair, KeypairType};
use utils::serializer::{deserialize, serialize};

pub struct ClientObj {
    client: Client,
    url: String,
    keypair: KeypairType,
}

impl ClientObj {
    pub fn new(config: &Configuration) -> ClientObj {
        std::env::set_var("RUST_BACKTRACE", "1");
        // let public_address: String = hex::encode(kp.public().encode());
        let mut secret = hex::decode(config.secret.clone()).expect("invalid secret");
        let keypair = Keypair::generate_from(secret.as_mut_slice());
        ClientObj {
            client: Client::default(),
            url: config.url.clone(),
            keypair,
        }
    }

    pub fn get_keypair(&self) -> &KeypairType {
        &self.keypair
    }
    // request for transaction submission to validator
    pub async fn submit_transaction(&self, txn: &SignedTransaction) {
        let mut url: String = self.url.clone();
        url.extend("client/submit_transaction".chars());
        let serialized_body: Vec<u8> = match serialize(txn) {
            Result::Ok(value) => value,
            Result::Err(_) => vec![0],
        };
        let response = self
            .client
            .post(url) // <- Create request builder
            .header("User-Agent", "Actix-web")
            .send_body(Bytes::from(serialized_body)) // <- Send http request
            .await;
        match response {
            Ok(response) => info!("submit_transaction Status: {:?}", response.status()),
            Err(e) => error!("Error response: {:?}", e),
        }
    }

    // request to peer to fetch pending transaction
    pub async fn fetch_pending_transaction(&self, txn_hash: &Hash) {
        let mut url: String = self.url.clone();
        url.extend("client/fetch_pending_transaction".chars());
        let serialized_body: Vec<u8> = match serialize(txn_hash) {
            Result::Ok(value) => value,
            Result::Err(_) => vec![0],
        };
        let result = self
            .client
            .get(url) // <- Create request builder
            .header("User-Agent", "Actix-web")
            .send_body(Bytes::from(serialized_body))
            .await
            .map_err(|_| ());
        match result {
            Ok(mut response) => {
                let resp_body = response.body();
                info!("fetch_pending_transaction Status: {:?}", response.status());
                if response.status() == 200 {
                    match resp_body.await {
                        Ok(txnbody) => {
                            if let Ok(signed_transaction) =
                                deserialize::<SignedTransaction>(&txnbody)
                            {
                                if let Ok(crypto_transaction) =
                                    deserialize::<CryptoTransaction>(&signed_transaction.txn)
                                {
                                    info!("{:#?}", crypto_transaction);
                                } else {
                                    info!("crypto transaction couldn't deserialize");
                                }
                            } else {
                                info!("signed transaction couldn't deserialize");
                            }
                        }
                        Err(e) => error!("Error body: {:?}", e),
                    }
                }
            }
            Err(e) => error!("Error response: {:?}", e),
        }
    }

    // request to peer to fetch public_address state
    pub async fn fetch_state(&self, public_address: &String) {
        let mut url: String = self.url.clone();
        url.extend("client/fetch_state".chars());
        let serialized_body: Vec<u8> = match serialize(public_address) {
            Result::Ok(value) => value,
            Result::Err(_) => vec![0],
        };
        let result = self
            .client
            .get(url) // <- Create request builder
            .header("User-Agent", "Actix-web")
            //.send() // <- Send http request
            .send_body(Bytes::from(serialized_body))
            .await
            .map_err(|_| ());
        match result {
            Ok(mut response) => {
                let resp_body = response.body();
                info!("fetch_state Status: {:?}", response.status());
                if response.status() == 200 {
                    match resp_body.await {
                        Ok(state) => {
                            if let Ok(state) = deserialize::<State>(&state) {
                                if let Ok(doc_state) =
                                    deserialize::<DocState>(state.get_data().as_slice())
                                {
                                    info!("{:#?}", doc_state);
                                } else {
                                    info!("document state couldn't deserialize");
                                }
                            } else {
                                info!("state couldn't deserialize");
                            }
                        }
                        Err(e) => error!("Error body: {:?}", e),
                    }
                }
            }
            Err(e) => error!("Error response: {:?}", e),
        }
    }

    // request to peer to fetch block
    pub async fn fetch_block(&self, block_index: &u64) {
        let mut url: String = self.url.clone();
        url.extend("client/fetch_block".chars());
        let serialized_body: Vec<u8> = match serialize(block_index) {
            Result::Ok(value) => value,
            Result::Err(_) => vec![0],
        };
        let result = self
            .client
            .get(url) // <- Create request builder
            .header("User-Agent", "Actix-web")
            //.send() // <- Send http request
            .send_body(Bytes::from(serialized_body))
            .await
            .map_err(|_| ());
        match result {
            Ok(mut response) => {
                let resp_body = response.body();
                info!("fetch_block Status: {:?}", response.status());
                if response.status() == 200 {
                    match resp_body.await {
                        Ok(block) => {
                            match deserialize::<String>(&block) {
                                Result::Ok(fetched_block) => info!("{:#?}", fetched_block),
                                Result::Err(_) => info!("block couldn't deserializ"),
                            };
                        }
                        Err(e) => error!("Error body: {:?}", e),
                    }
                }
            }
            Err(e) => error!("Error response: {:?}", e),
        }
    }

    // request to peer to fetch confirmed transaction
    pub async fn fetch_confirm_transaction(&self, txn_hash: &Hash) {
        let mut url: String = self.url.clone();
        url.extend("client/fetch_confirm_transaction".chars());
        let serialized_body: Vec<u8> = match serialize(txn_hash) {
            Result::Ok(value) => value,
            Result::Err(_) => vec![0],
        };
        let result = self
            .client
            .get(url) // <- Create request builder
            .header("User-Agent", "Actix-web")
            //.send() // <- Send http request
            .send_body(Bytes::from(serialized_body))
            .await
            .map_err(|_| ());
        match result {
            Ok(mut response) => {
                let resp_body = response.body();
                info!("fetch_confirm_transaction Status: {:?}", response.status());
                if response.status() == 200 {
                    match resp_body.await {
                        Ok(txnbody) => {
                            if let Ok(signed_transaction) =
                                deserialize::<SignedTransaction>(&txnbody)
                            {
                                if let Ok(crypto_transaction) =
                                    deserialize::<CryptoTransaction>(&signed_transaction.txn)
                                {
                                    info!("{:#?}", crypto_transaction);
                                } else {
                                    info!("crypto transaction couldn't deserialize");
                                }
                            } else {
                                info!("signed transaction couldn't deserialize");
                            }
                        }
                        Err(e) => error!("Error body: {:?}", e),
                    }
                }
            }
            Err(e) => error!("Error response: {:?}", e),
        }
    }

    // request for fetching latest block
    pub async fn fetch_latest_block(&self) {
        let mut url: String = self.url.clone();
        url.extend("client/fetch_latest_block".chars());
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
                info!("fetch_block Status: {:?}", response.status());
                if response.status() == 200 {
                    match resp_body.await {
                        Ok(block) => {
                            match deserialize::<String>(&block) {
                                Result::Ok(fetched_block) => info!("{:#?}", fetched_block),
                                Result::Err(_) => info!("block couldn't deserializ"),
                            };
                        }
                        Err(e) => error!("Error body: {:?}", e),
                    }
                }
            }
            Err(e) => error!("Error response: {:?}", e),
        }
    }
}
