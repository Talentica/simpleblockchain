#[macro_use]
extern crate lazy_static;

mod cli_config;
mod client;

use crate::client::{ClientObj, SyncState};
use exonum_crypto::{Hash, PublicKey};
use std::io;
use utils::serializer::serialize;

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
fn main() {
    let mut client: ClientObj = ClientObj::new();
    let mut end_flag: bool = false;
    let mut invalid_opt_count: u8 = 0;
    while !end_flag {
        println!();
        println!("Application CLI support following operations:");
        println!("1:) reset client");
        println!("2:) fetch pending transaction");
        println!("3:) fetch confirmed transaction");
        println!("4:) fetch state details");
        println!("5:) fetch block");
        println!("6:) fetch latest block");
        println!("7:) fetch blockchain length");
        println!("8:) fetch sync states");
        println!("9:) exit");
        let mut input = String::new();
        println!("Please select Option:");
        let is_string: bool = get_string_input(&mut input);
        if is_string {
            if input == String::from("1") {
                invalid_opt_count = 0;
                client = ClientObj::new();
            } else if input == String::from("2") {
                invalid_opt_count = 0;
                println!("Enter transaction Hash");
                let is_hash: Option<Hash> = get_hash_input(&mut input);
                match is_hash {
                    Some(txn_hash) => {
                        let output = client.fetch_pending_transaction(&txn_hash);
                        println!("{:#?}", output);
                    }
                    None => println!("error: invalid input for transaction hash"),
                }
            } else if input == String::from("3") {
                invalid_opt_count = 0;
                println!("Enter transaction Hash");
                let is_hash: Option<Hash> = get_hash_input(&mut input);
                match is_hash {
                    Some(txn_hash) => {
                        let output = client.fetch_confirm_transaction(&txn_hash);
                        println!("{:#?}", output);
                    }
                    None => println!("error: invalid input for transaction hash"),
                }
            } else if input == String::from("4") {
                invalid_opt_count = 0;
                println!("Enter public address");
                let is_pk: bool = get_public_key_input(&mut input);
                if is_pk {
                    let output = client.fetch_state(&input);
                    println!("{:#?}", output);
                } else {
                    println!("error: invalid public_address");
                }
            } else if input == String::from("5") {
                invalid_opt_count = 0;
                println!("Enter block index");
                let is_index: Option<u64> = get_integer_input(&mut input);
                match is_index {
                    Some(index) => {
                        let output = client.fetch_block(&index);
                        println!("{:#?}", output);
                    }
                    None => println!("error: invalid input for block index"),
                }
            } else if input == String::from("6") {
                let output = client.fetch_latest_block();
                println!("{:#?}", output);
                invalid_opt_count = 0;
            } else if input == String::from("7") {
                let output = client.fetch_blockchain_length();
                println!("{:#?}", output);
                invalid_opt_count = 0;
            } else if input == String::from("8") {
                let output: SyncState = client.fetch_sync_state(0);
                println!("{:?}", output);
                invalid_opt_count = 0;
            } else if input == String::from("9") {
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
