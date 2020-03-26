#[macro_use]
extern crate lazy_static;

extern crate futures;
use doc_app::transaction::{SignedTransaction, TransactionTrait};
use exonum_crypto::Hash;
mod client;
use client::ClientObj;
mod build_transaction;
use build_transaction::*;
mod cli_config;

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
                let is_txn: Option<SignedTransaction> = create_transaction(client.get_keypair());
                match is_txn {
                    Some(txn) => {
                        let txn_hash = txn.get_hash();
                        let string_txn_hash: String = txn_hash.to_hex();
                        println!("txn_hash {:?}", string_txn_hash);
                        client.submit_transaction(&txn).await;
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
