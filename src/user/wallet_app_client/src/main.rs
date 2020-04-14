#[macro_use]
extern crate exonum_derive;

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate log;

extern crate futures;

mod wallet_app_types;
use crate::wallet_app_types::{CryptoTransaction, SignedTransaction, TransactionTrait};
use exonum_crypto::{Hash, PublicKey};
use std::collections::HashMap;
use std::io;
use utils::crypto::keypair::KeypairType;
mod client;
use client::ClientObj;
use std::time::SystemTime;
use utils::logger::*;
use utils::serializer::serialize;

mod cli_config;
const APPNAME: &str = "Cryptocurrency";

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

pub fn create_transaction(kp: &KeypairType, nonce: u64) -> Option<SignedTransaction> {
    let mut crypto_transaction: CryptoTransaction = CryptoTransaction::generate(kp);
    crypto_transaction.nonce = nonce;
    let mut input = String::new();
    info!("1): transfer transaction");
    info!("2): mint transaction");
    info!("Please select transaction_type:");
    let is_string: bool = get_string_input(&mut input);
    if is_string {
        if input == String::from("1") {
            crypto_transaction.fxn_call = String::from("transfer");
            info!("Please enter to_address:");
            let is_public_key: bool = get_public_key_input(&mut input);
            if is_public_key {
                crypto_transaction.to = input.clone();
            } else {
                return None;
            }
        } else if input == String::from("2") {
            crypto_transaction.fxn_call = String::from("mint");
            crypto_transaction.to = String::default();
        } else {
            info!("invalid option");
            return None;
        }
    } else {
        return None;
    }
    let mut input = String::new();
    info!("Please enter amount:");
    let is_amount: Option<u64> = get_integer_input(&mut input);
    match is_amount {
        Some(amount) => crypto_transaction.amount = amount,
        None => return None,
    };
    let txn_sign = crypto_transaction.sign(&kp);
    let mut header = HashMap::default();
    let time_stamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_micros();
    header.insert("timestamp".to_string(), time_stamp.to_string());
    Some(SignedTransaction {
        txn: serialize(&crypto_transaction),
        app_name: String::from(APPNAME),
        signature: txn_sign,
        header,
    })
}

//this attribute allows main to not need to return anything and still use async calls.
#[actix_rt::main]
async fn main() {
    file_logger_init_from_yml(&String::from("log.yml"));
    info!("Wallet Application CLient Bootstrapping");
    let cli_configuration: &cli_config::Configuration = &cli_config::GLOBAL_CONFIG;
    let client: ClientObj = ClientObj::new(cli_configuration);
    let mut end_flag: bool = false;
    let mut invalid_opt_count: u8 = 0;
    while !end_flag {
        info!("");
        info!("Application CLI support following operations:");
        info!("1:) submit transaction");
        info!("2:) fetch pending transaction");
        info!("3:) fetch confirmed transaction");
        info!("4:) fetch state details");
        info!("5:) fetch block");
        info!("6:) fetch latest block");
        info!("7:) exit");
        let mut input = String::new();
        info!("Please select Option:");
        let is_string: bool = get_string_input(&mut input);
        if is_string {
            if input == String::from("1") {
                invalid_opt_count = 0;
                let is_nonce = client.get_nonce().await;
                match is_nonce {
                    Some(nonce) => {
                        let is_txn: Option<SignedTransaction> =
                            create_transaction(client.get_keypair(), nonce);
                        match is_txn {
                            Some(txn) => {
                                let txn_hash = txn.get_hash();
                                let string_txn_hash: String = txn_hash.to_hex();
                                info!("txn_hash {:?}", string_txn_hash);
                                client.submit_transaction(&txn).await;
                            }
                            None => error!("error: invalid input for submit transaction"),
                        }
                    }
                    None => error!("SomeThing Wrong happened. Check error"),
                }
            } else if input == String::from("2") {
                invalid_opt_count = 0;
                info!("Enter transaction Hash");
                let is_hash: Option<Hash> = get_hash_input(&mut input);
                match is_hash {
                    Some(txn_hash) => client.fetch_pending_transaction(&txn_hash).await,
                    None => error!("error: invalid input for transaction hash"),
                }
            } else if input == String::from("3") {
                invalid_opt_count = 0;
                info!("Enter transaction Hash");
                let is_hash: Option<Hash> = get_hash_input(&mut input);
                match is_hash {
                    Some(txn_hash) => client.fetch_confirm_transaction(&txn_hash).await,
                    None => error!("error: invalid input for transaction hash"),
                }
            } else if input == String::from("4") {
                invalid_opt_count = 0;
                info!("Enter public address");
                let is_pk: bool = get_public_key_input(&mut input);
                if is_pk {
                    client.fetch_state(&input).await;
                } else {
                    error!("error: invalid public_address");
                }
            } else if input == String::from("5") {
                invalid_opt_count = 0;
                info!("Enter block index");
                let is_index: Option<u64> = get_integer_input(&mut input);
                match is_index {
                    Some(index) => client.fetch_block(&index).await,
                    None => error!("error: invalid input for block index"),
                }
            } else if input == String::from("6") {
                client.fetch_latest_block().await;
                invalid_opt_count = 0;
            } else if input == String::from("7") {
                end_flag = true;
            } else {
                info!("invalid option");
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
