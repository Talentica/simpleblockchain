#[macro_use]
extern crate exonum_derive;

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate log;

extern crate futures;
mod doc_app_types;
use crate::doc_app_types::{SignedTransaction, TransactionTrait};
use exonum_crypto::Hash;
mod client;
use client::ClientObj;
mod build_transaction;
use build_transaction::*;
use utils::logger::logger_init_from_yml;
mod cli_config;
use clap::{App, Arg};

//this attribute allows main to not need to return anything and still use async calls.
#[actix_rt::main]
async fn main() {
    let matches = App::new("SimpleBlockchain Document Review App Client")
        .version("0.1.0")
        .author("gaurav agarwal <gaurav.agarwal@talentica.com>")
        .about("Document Review app client command line arguent parser")
        .arg(
            Arg::with_name("cli_config_path")
                .short("c")
                .long("config")
                .takes_value(true)
                .help("config file"),
        )
        .arg(
            Arg::with_name("logger_file_path")
                .short("l")
                .long("logger")
                .takes_value(true)
                .help("logger file path"),
        )
        .get_matches();
    let config_file_path = matches
        .value_of("cli_config_path")
        .unwrap_or("cli_config.toml");
    let logger_file_path = matches
        .value_of("logger_file_path")
        .unwrap_or("client_log.yml");
    cli_config::initialize_config(config_file_path);
    logger_init_from_yml(logger_file_path);
    info!("Document Application CLient Bootstrapping");
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
                let is_txn: Option<SignedTransaction> = create_transaction(client.get_keypair());
                match is_txn {
                    Some(txn) => {
                        let txn_hash = txn.get_hash();
                        let string_txn_hash: String = txn_hash.to_hex();
                        info!("txn_hash {:?}", string_txn_hash);
                        client.submit_transaction(&txn).await;
                    }
                    None => error!("error: invalid input for submit transaction"),
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
