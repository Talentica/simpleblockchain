extern crate futures;
use doc_app::transaction::{SignedTransaction, TransactionTrait};
use doc_app::user_messages::{CryptoTransaction, DataTypes};
use exonum_crypto::{Hash, PublicKey};
use std::collections::HashMap;
use std::io;
use std::time::SystemTime;
use utils::crypto::keypair::KeypairType;

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

fn validate_payload_for_set_hash() -> Option<Vec<DataTypes>> {
    let mut fxn_arguments: Vec<DataTypes> = Vec::new();
    let mut input = String::new();
    info!("Please enter NFT hash:");
    let option_txn_hash: Option<Hash> = get_hash_input(&mut input);
    match option_txn_hash {
        Some(txn_hash) => fxn_arguments.push(DataTypes::HashVal(txn_hash)),
        None => return None,
    }
    info!("Please enter NFT doc hash:");
    let option_txn_hash: Option<Hash> = get_hash_input(&mut input);
    match option_txn_hash {
        Some(txn_hash) => fxn_arguments.push(DataTypes::HashVal(txn_hash)),
        None => return None,
    }
    Some(fxn_arguments)
}

fn validate_payload_for_add_doc() -> Option<Vec<DataTypes>> {
    let mut fxn_arguments: Vec<DataTypes> = Vec::new();
    let mut input = String::new();
    info!("Please enter Doc hash count:");
    let is_count = get_integer_input(&mut input);
    match is_count {
        Some(count) => {
            let mut hash_vec: Vec<Hash> = Vec::new();
            for _ in 0..count {
                info!("Please enter NFT hash:");
                let option_txn_hash: Option<Hash> = get_hash_input(&mut input);
                match option_txn_hash {
                    Some(txn_hash) => hash_vec.push(txn_hash),
                    None => return None,
                }
            }
            fxn_arguments.push(DataTypes::VecHashVal(hash_vec));
        }
        None => return None,
    }
    Some(fxn_arguments)
}

fn validate_payload_for_transfer_sc() -> Option<Vec<DataTypes>> {
    let mut fxn_arguments: Vec<DataTypes> = Vec::new();
    let mut input = String::new();
    info!("Please enter Doc hash count:");
    let is_count = get_integer_input(&mut input);
    match is_count {
        Some(count) => {
            let mut hash_vec: Vec<Hash> = Vec::new();
            for _ in 0..count {
                info!("Please enter NFT hash:");
                let option_txn_hash: Option<Hash> = get_hash_input(&mut input);
                match option_txn_hash {
                    Some(txn_hash) => hash_vec.push(txn_hash),
                    None => return None,
                }
            }
            fxn_arguments.push(DataTypes::VecHashVal(hash_vec));
        }
        None => return None,
    }
    info!("Please enter SC public_key:");
    let is_pk: bool = get_public_key_input(&mut input);
    if is_pk {
        fxn_arguments.push(DataTypes::StringVal(input));
    } else {
        return None;
    }
    Some(fxn_arguments)
}

fn validate_payload_for_set_pkg_no() -> Option<Vec<DataTypes>> {
    let mut fxn_arguments: Vec<DataTypes> = Vec::new();
    let mut input = String::new();
    info!("Please enter Doc hash count:");
    let is_count = get_integer_input(&mut input);
    match is_count {
        Some(count) => {
            let mut hash_vec: Vec<Hash> = Vec::new();
            for _ in 0..count {
                info!("Please enter NFT hash:");
                let option_txn_hash: Option<Hash> = get_hash_input(&mut input);
                match option_txn_hash {
                    Some(txn_hash) => hash_vec.push(txn_hash),
                    None => return None,
                }
            }
            fxn_arguments.push(DataTypes::VecHashVal(hash_vec));
        }
        None => return None,
    }
    info!("Please enter package_no for given docs:");
    let is_pkg_no: bool = get_string_input(&mut input);
    if is_pkg_no {
        fxn_arguments.push(DataTypes::StringVal(input));
    } else {
        return None;
    }
    Some(fxn_arguments)
}

fn validate_payload_for_transfer_for_review() -> Option<Vec<DataTypes>> {
    let mut fxn_arguments: Vec<DataTypes> = Vec::new();
    let mut input = String::new();

    info!("Please enter pkg_no:");
    let is_pk: bool = get_string_input(&mut input);
    if is_pk {
        fxn_arguments.push(DataTypes::StringVal(input.clone()));
    } else {
        return None;
    }

    info!("Please enter SC public_key:");
    let is_pk: bool = get_public_key_input(&mut input);
    if is_pk {
        fxn_arguments.push(DataTypes::StringVal(input));
    } else {
        return None;
    }
    Some(fxn_arguments)
}

fn validate_payload_for_review_docs() -> Option<Vec<DataTypes>> {
    let mut fxn_arguments: Vec<DataTypes> = Vec::new();
    let mut input = String::new();

    info!("Please enter pkg_no:");
    let is_pk: bool = get_string_input(&mut input);
    if is_pk {
        fxn_arguments.push(DataTypes::StringVal(input.clone()));
    } else {
        return None;
    }

    info!("Please enter reviewed value (yes or no)(true or false) :");
    let is_bool: Option<bool> = get_bool_input(&mut input);
    match is_bool {
        Some(bool_val) => fxn_arguments.push(DataTypes::BoolVal(bool_val)),
        None => return None,
    }
    Some(fxn_arguments)
}

fn validate_payload_for_publish_docs() -> Option<Vec<DataTypes>> {
    let mut fxn_arguments: Vec<DataTypes> = Vec::new();
    let mut input = String::new();
    info!("Please enter pkg_no:");
    let is_string: bool = get_string_input(&mut input);
    if is_string {
        fxn_arguments.push(DataTypes::StringVal(input.clone()));
    } else {
        return None;
    }
    Some(fxn_arguments)
}

pub fn create_transaction(kp: &KeypairType) -> Option<SignedTransaction> {
    let mut crypto_transaction: CryptoTransaction = CryptoTransaction::generate(kp);
    crypto_transaction.payload.clear();
    let mut input = String::new();
    info!("1): Set_Hash transaction");
    info!("2): Add_Doc transaction");
    info!("3): Transfer_SC transaction");
    info!("4): Set_Pkg_No transaction");
    info!("5): Transfer_For_Review transaction");
    info!("6): Review_Docs transaction");
    info!("7): Publish_Docs transaction");
    info!("Please select transaction_type:");
    match io::stdin().read_line(&mut input) {
        Ok(_) => {
            remove_trailing_newline(&mut input);
            if input == String::from("1") {
                crypto_transaction.fxn_call = String::from("set_hash");
                match validate_payload_for_set_hash() {
                    Some(vec_data_types) => {
                        crypto_transaction.payload = vec_data_types;
                    }
                    None => return None,
                }
            } else if input == String::from("2") {
                crypto_transaction.fxn_call = String::from("add_doc");
                match validate_payload_for_add_doc() {
                    Some(vec_data_types) => {
                        crypto_transaction.payload = vec_data_types;
                    }
                    None => return None,
                }
            } else if input == String::from("3") {
                crypto_transaction.fxn_call = String::from("transfer_sc");
                match validate_payload_for_transfer_sc() {
                    Some(vec_data_types) => {
                        crypto_transaction.payload = vec_data_types;
                    }
                    None => return None,
                }
            } else if input == String::from("4") {
                crypto_transaction.fxn_call = String::from("set_pkg_no");
                match validate_payload_for_set_pkg_no() {
                    Some(vec_data_types) => {
                        crypto_transaction.payload = vec_data_types;
                    }
                    None => return None,
                }
            } else if input == String::from("5") {
                crypto_transaction.fxn_call = String::from("transfer_for_review");
                match validate_payload_for_transfer_for_review() {
                    Some(vec_data_types) => {
                        crypto_transaction.payload = vec_data_types;
                    }
                    None => return None,
                }
            } else if input == String::from("6") {
                crypto_transaction.fxn_call = String::from("review_docs");
                match validate_payload_for_review_docs() {
                    Some(vec_data_types) => {
                        crypto_transaction.payload = vec_data_types;
                    }
                    None => return None,
                }
            } else if input == String::from("7") {
                crypto_transaction.fxn_call = String::from("publish_docs");
                match validate_payload_for_publish_docs() {
                    Some(vec_data_types) => {
                        crypto_transaction.payload = vec_data_types;
                    }
                    None => return None,
                }
            } else {
                info!("invalid option");
                return None;
            }
        }
        Err(error) => {
            error!("error: {}", error);
            return None;
        }
    }
    let txn_sign = crypto_transaction.sign(&kp);
    let mut header = HashMap::default();
    let time_stamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_micros();
    header.insert("timestamp".to_string(), time_stamp.to_string());
    Some(SignedTransaction {
        txn: Some(crypto_transaction),
        signature: txn_sign,
        header,
    })
}
