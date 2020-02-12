extern crate futures;
extern crate schema;

use awc::Client;
use bytes::Bytes;
use prost::Message;
use std::io::Cursor;

use schema::user_messages::*;

//Sample Code to generate/serialize/deserialize the transactions using protobuf/schema::message.
pub fn create_transaction(nnce: u64, amt: u64) -> Transaction {
    Transaction {
        nonce: nnce,
        amount: amt,
        from: "ABCD1234".to_string(),
        to: "wxyz6789".to_string(),
        fxn_call: "".to_string(),
        payload: vec![],
    }
}

pub fn serialize_transaction(txn: &Transaction) -> Vec<u8> {
    let mut buf = Vec::new();
    buf.reserve(txn.encoded_len());
    txn.encode(&mut buf).unwrap();
    buf
}

pub fn deserialize_transaction(buf: &[u8]) -> Transaction {
    let txn = Transaction::decode(&mut Cursor::new(buf)).unwrap();
    println!(
        "Txn: Nonce:{0} Balance:{1} From:{2} To:{3}",
        txn.nonce, txn.amount, txn.from, txn.to
    );

    txn
}

//this attribute allows main to not need to return anything and still use async calls.
#[actix_rt::main]
async fn main() {
    std::env::set_var("RUST_BACKTRACE", "1");
    let txn = create_transaction(2, 190);

    let client = Client::default();
    let result = client
        .post("http://localhost:8089") // <- Create request builder
        .header("User-Agent", "Actix-web")
        //.send() // <- Send http request
        .send_body(Bytes::from(serialize_transaction(&txn)))
        .await
        .map_err(|_| ());

    match result {
        Ok(mut response) => {
            let resp_body = response.body();
            match resp_body.await {
                Ok(txnbody) => {
                    println!("Status: {:?}, Len: {:?}", response.status(), txnbody.len());
                    let ret_txn = deserialize_transaction(&txnbody);
                    println!("Ret Txn: \n{:?}", ret_txn);
                }
                Err(e) => println!("Error: {:?}", e),
            }
        }
        Err(e) => println!("Error: {:?}", e),
    }
}
