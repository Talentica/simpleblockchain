extern crate futures;

use awc::Client;
use bytes::Bytes;
use doc_app::state::State;
use doc_app::transaction::SignedTransaction;
use exonum_crypto::Hash;
use utils::crypto::keypair::{CryptoKeypair, Keypair, KeypairType};
use utils::serializer::{deserialize, serialize};

use crate::cli_config::Configuration;

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
        let response = self
            .client
            .post(url) // <- Create request builder
            .header("User-Agent", "Actix-web")
            .send_body(Bytes::from(serialize(&txn))) // <- Send http request
            .await;
        match response {
            Ok(response) => println!("submit_transaction Status: {:?}", response.status()),
            Err(e) => eprintln!("Error response: {:?}", e),
        }
    }

    // request to peer to fetch pending transaction
    pub async fn fetch_pending_transaction(&self, txn_hash: &Hash) {
        let mut url: String = self.url.clone();
        url.extend("client/fetch_pending_transaction".chars());
        let result = self
            .client
            .get(url) // <- Create request builder
            .header("User-Agent", "Actix-web")
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
                            let signed_transaction: SignedTransaction = deserialize(&txnbody);
                            println!("{:#?}", signed_transaction.txn);
                        }
                        Err(e) => println!("Error body: {:?}", e),
                    }
                }
            }
            Err(e) => println!("Error response: {:?}", e),
        }
    }

    // request to peer to fetch public_address state
    pub async fn fetch_state(&self, public_address: &String) {
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
                        Ok(state) => {
                            let ret_txn: State = deserialize(&state);
                            println!("{:#?}", ret_txn);
                        }
                        Err(e) => println!("Error body: {:?}", e),
                    }
                }
            }
            Err(e) => println!("Error response: {:?}", e),
        }
    }

    // request to peer to fetch block
    pub async fn fetch_block(&self, block_index: &u64) {
        let mut url: String = self.url.clone();
        url.extend("client/fetch_block".chars());
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
                            let fetched_block: String = deserialize(&state);
                            println!("{:#?}", fetched_block);
                        }
                        Err(e) => println!("Error body: {:?}", e),
                    }
                }
            }
            Err(e) => println!("Error response: {:?}", e),
        }
    }

    // request to peer to fetch confirmed transaction
    pub async fn fetch_confirm_transaction(&self, txn_hash: &Hash) {
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
                            let signed_transaction: SignedTransaction = deserialize(&txnbody);
                            println!("{:#?}", signed_transaction.txn);
                        }
                        Err(e) => eprintln!("Error body: {:?}", e),
                    }
                }
            }
            Err(e) => eprintln!("Error response: {:?}", e),
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
                println!("fetch_block Status: {:?}", response.status());
                if response.status() == 200 {
                    match resp_body.await {
                        Ok(state) => {
                            let fetched_block: String = deserialize(&state);
                            eprintln!("{:#?}", fetched_block);
                        }
                        Err(e) => eprintln!("Error body: {:?}", e),
                    }
                }
            }
            Err(e) => eprintln!("Error response: {:?}", e),
        }
    }
}
