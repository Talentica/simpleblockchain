#[macro_use]
extern crate lazy_static;

extern crate futures;
use std::io;

use awc::Client;
use bytes::Bytes;
use exonum_crypto::{Hash, PublicKey};
use schema::block::SignedBlock;
use utils::serializer::{deserialize, serialize};
mod cli_config;

pub struct ClientObj {
    client: Client,
    url: String,
}

pub fn remove_trailing_newline(input: &mut String) {
    while input.ends_with('\n') {
        input.pop();
        while input.ends_with('\r') {
            input.pop();
        }
    }
}

pub fn get_hash_input(input: &mut String) -> Option<Hash> {
    input.clear();
    match io::stdin().read_line(input) {
        Ok(_) => {
            remove_trailing_newline(input);
            let decoding_result = hex::decode(input.as_bytes());
            match decoding_result {
                Ok(slice) => {
                    let option_txn_hash: Option<Hash> = Hash::from_slice(&slice);
                    return option_txn_hash;
                }
                Err(_) => return None,
            }
        }
        Err(_) => return None,
    };
}

pub fn get_integer_input(input: &mut String) -> Option<u64> {
    input.clear();
    match io::stdin().read_line(input) {
        Ok(_) => {
            remove_trailing_newline(input);
            let is_index: Result<u64, std::num::ParseIntError> = input.parse::<u64>();
            match is_index {
                Ok(inetger) => return Some(inetger),
                Err(_) => return None,
            }
        }
        Err(_) => return None,
    };
}

pub fn get_string_input(input: &mut String) -> bool {
    input.clear();
    match io::stdin().read_line(input) {
        Ok(_) => {
            remove_trailing_newline(input);
            if input.len() > 0 {
                return true;
            } else {
                return false;
            }
        }
        Err(_) => return false,
    };
}

pub fn get_public_key_input(input: &mut String) -> bool {
    input.clear();
    match io::stdin().read_line(input) {
        Ok(_) => {
            remove_trailing_newline(input);
            let decoding_result = hex::decode(input.as_bytes());
            match decoding_result {
                Ok(slice) => {
                    let option_txn_hash: Option<PublicKey> = PublicKey::from_slice(&slice);
                    match option_txn_hash {
                        Some(_) => return true,
                        None => return false,
                    }
                }
                Err(_) => return false,
            }
        }
        Err(_) => return false,
    };
}

pub fn get_bool_input(input: &mut String) -> Option<bool> {
    input.clear();
    match io::stdin().read_line(input) {
        Ok(_) => {
            remove_trailing_newline(input);
            let lower_case = input.to_ascii_lowercase();
            if lower_case == "y" || lower_case == "yes" || lower_case == "t" || lower_case == "true"
            {
                return Some(true);
            } else if lower_case == "n"
                || lower_case == "no"
                || lower_case == "f"
                || lower_case == "false"
            {
                return Some(false);
            } else {
                return None;
            }
        }
        Err(_) => return None,
    };
}

impl ClientObj {
    pub fn new(config: &cli_config::Configuration) -> ClientObj {
        std::env::set_var("RUST_BACKTRACE", "1");
        ClientObj {
            client: Client::default(),
            url: config.url.clone(),
        }
    }

    // request for transaction submission to validator
    async fn submit_transaction(&self, serialized_txn: Vec<u8>) {
        let mut url: String = self.url.clone();
        url.extend("client/submit_transaction".chars());
        let response = self
            .client
            .post(url) // <- Create request builder
            .header("User-Agent", "Actix-web")
            .send_body(Bytes::from(serialized_txn)) // <- Send http request
            .await;
        match response {
            Ok(response) => println!("submit_transaction Status: {:?}", response.status()),
            Err(e) => println!("Error response: {:?}", e),
        }
    }

    // request to peer to fetch pending transaction
    async fn fetch_pending_transaction(&self, txn_hash: &Hash) {
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
                            println!("{:#?}", txnbody);
                        }
                        Err(e) => println!("Error body: {:?}", e),
                    }
                }
            }
            Err(e) => println!("Error response: {:?}", e),
        }
    }

    // request to peer to fetch public_address state
    async fn fetch_state(&self, public_address: &String) {
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
                            println!("{:#?}", serialized_state);
                        }
                        Err(e) => println!("Error body: {:?}", e),
                    }
                }
            }
            Err(e) => println!("Error response: {:?}", e),
        }
    }

    // request to peer to fetch block
    async fn fetch_block(&self, block_index: &u64) {
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
    async fn fetch_confirm_transaction(&self, txn_hash: &Hash) {
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
                            println!("{:#?}", txnbody);
                        }
                        Err(e) => println!("Error body: {:?}", e),
                    }
                }
            }
            Err(e) => println!("Error response: {:?}", e),
        }
    }

    // request for fetching latest block
    async fn fetch_latest_block(&self) {
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
                            println!("{:#?}", fetched_block);
                        }
                        Err(e) => println!("Error body: {:?}", e),
                    }
                }
            }
            Err(e) => println!("Error response: {:?}", e),
        }
    }
}

pub fn get_vec_input(input: &mut String) -> Option<Vec<u8>> {
    input.clear();
    match io::stdin().read_line(input) {
        Ok(_) => {
            remove_trailing_newline(input);
            let vec: Vec<u8> = serialize(&input);
            return Some(vec);
        }
        Err(_) => return None,
    };
}

//this attribute allows main to not need to return anything and still use async calls.
#[actix_rt::main]
async fn main() {
    let cli_configuration: &cli_config::Configuration = &cli_config::GLOBAL_CONFIG;
    let client: ClientObj = ClientObj::new(cli_configuration);
    let mut end_flag: bool = false;
    let mut invalid_opt_count: u8 = 0;
    while !end_flag {
        println!();
        println!("Application CLI support following operations:");
        println!("1:) submit transaction");
        println!("2:) fetch pending transaction");
        println!("3:) fetch confirmed transaction");
        println!("4:) fetch state details");
        println!("5:) fetch block");
        println!("6:) fetch latest block");
        println!("7:) exit");
        let mut input = String::new();
        println!("Please select Option:");
        let is_string: bool = get_string_input(&mut input);
        if is_string {
            if input == String::from("1") {
                invalid_opt_count = 0;
                let is_txn_vec: Option<Vec<u8>> = get_vec_input(&mut input);
                match is_txn_vec {
                    Some(txn_vec) => {
                        client.submit_transaction(txn_vec).await;
                    }
                    None => println!("error: invalid input for submit transaction"),
                }
            } else if input == String::from("2") {
                invalid_opt_count = 0;
                println!("Enter transaction Hash");
                let is_hash: Option<Hash> = get_hash_input(&mut input);
                match is_hash {
                    Some(txn_hash) => client.fetch_pending_transaction(&txn_hash).await,
                    None => println!("error: invalid input for transaction hash"),
                }
            } else if input == String::from("3") {
                invalid_opt_count = 0;
                println!("Enter transaction Hash");
                let is_hash: Option<Hash> = get_hash_input(&mut input);
                match is_hash {
                    Some(txn_hash) => client.fetch_confirm_transaction(&txn_hash).await,
                    None => println!("error: invalid input for transaction hash"),
                }
            } else if input == String::from("4") {
                invalid_opt_count = 0;
                println!("Enter public address");
                let is_pk: bool = get_public_key_input(&mut input);
                if is_pk {
                    client.fetch_state(&input).await;
                } else {
                    println!("error: invalid public_address");
                }
            } else if input == String::from("5") {
                invalid_opt_count = 0;
                println!("Enter block index");
                let is_index: Option<u64> = get_integer_input(&mut input);
                match is_index {
                    Some(index) => client.fetch_block(&index).await,
                    None => println!("error: invalid input for block index"),
                }
            } else if input == String::from("6") {
                client.fetch_latest_block().await;
                invalid_opt_count = 0;
            } else if input == String::from("7") {
                end_flag = true;
            } else {
                println!("invalid option");
                invalid_opt_count = invalid_opt_count + 1;
            }
        } else {
            invalid_opt_count = invalid_opt_count + 1;
        }
        if invalid_opt_count > 2 {
            end_flag = true;
        }
    }
}
