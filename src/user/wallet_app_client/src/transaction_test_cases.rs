use crate::client::ClientObj;
use crate::wallet_app_types::{CryptoTransaction, SignedTransaction, TransactionTrait};
use exonum_merkledb::ObjectHash;
use std::collections::HashMap;
use std::time::SystemTime;
use utils::crypto::keypair::{CryptoKeypair, Keypair, KeypairType};
use utils::serializer::serialize;

const APPNAME: &str = "Cryptocurrency";

async fn create_transaction(client: &mut ClientObj) -> SignedTransaction {
    let kp: KeypairType = Keypair::generate();
    let mut crypto_transaction: CryptoTransaction = CryptoTransaction::generate(&kp);
    client.set_keypair(kp);
    let nonce: u64 = match client.get_nonce().await {
        Some(nonce) => nonce,
        None => 0,
    };
    crypto_transaction.nonce = nonce;
    crypto_transaction.fxn_call = String::from("mint");
    crypto_transaction.to = String::from("");
    crypto_transaction.amount = 11;

    let txn_sign = crypto_transaction.sign(&client.get_keypair());
    let mut header = HashMap::default();
    let time_stamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_micros();
    header.insert("timestamp".to_string(), time_stamp.to_string());
    let serialized_txn: Vec<u8> = match serialize(&crypto_transaction) {
        Result::Ok(value) => value,
        Result::Err(_) => vec![0],
    };
    SignedTransaction {
        txn: serialized_txn,
        app_name: String::from(APPNAME),
        signature: txn_sign,
        header,
    }
}

pub async fn create_transactions_set(mut client: &mut ClientObj) {
    // this loop will take 100 seconds
    let mut count: u64 = 100;
    while count > 0 {
        count = count - 1;
        let mint_txn: SignedTransaction = create_transaction(&mut client).await;
        info!("txn hash {:?}", mint_txn.object_hash());
        client.submit_transaction(&mint_txn).await;
        std::thread::sleep(std::time::Duration::from_millis(1000));
    }
    // this loop will take 80 seconds
    let mut count: u64 = 100;
    while count > 0 {
        count = count - 1;
        let mint_txn: SignedTransaction = create_transaction(&mut client).await;
        info!("txn hash {:?}", mint_txn.object_hash());
        client.submit_transaction(&mint_txn).await;
        std::thread::sleep(std::time::Duration::from_millis(800));
    }
    // this loop will take 60 seconds
    let mut count: u64 = 100;
    while count > 0 {
        count = count - 1;
        let mint_txn: SignedTransaction = create_transaction(&mut client).await;
        info!("txn hash {:?}", mint_txn.object_hash());
        client.submit_transaction(&mint_txn).await;
        std::thread::sleep(std::time::Duration::from_millis(600));
    }
    // this loop will take 40 seconds
    let mut count: u64 = 100;
    while count > 0 {
        count = count - 1;
        let mint_txn: SignedTransaction = create_transaction(&mut client).await;
        info!("txn hash {:?}", mint_txn.object_hash());
        client.submit_transaction(&mint_txn).await;
        std::thread::sleep(std::time::Duration::from_millis(400));
    }

    // this loop will take 20 seconds
    let mut count: u64 = 100;
    while count > 0 {
        count = count - 1;
        let mint_txn: SignedTransaction = create_transaction(&mut client).await;
        info!("txn hash {:?}", mint_txn.object_hash());
        client.submit_transaction(&mint_txn).await;
        std::thread::sleep(std::time::Duration::from_millis(200));
    }
}
