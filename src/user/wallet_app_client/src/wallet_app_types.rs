extern crate utils;
use exonum_crypto::Hash;
use exonum_merkledb::ObjectHash;
pub use sdk::signed_transaction::SignedTransaction;
use std::collections::HashMap;
use std::convert::AsRef;
use std::time::SystemTime;
use utils::keypair::{CryptoKeypair, Keypair, KeypairType, PublicKey, Verify};
use utils::serializer::{deserialize, serialize, Deserialize, Serialize};
const APPNAME: &str = "Cryptocurrency";

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, BinaryValue, ObjectHash)]
#[binary_value(codec = "bincode")]
pub struct CryptoState {
    pub nonce: u64,
    pub balance: u64,
    pub storage_root: Hash,
    pub code_hash: Hash,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinaryValue, ObjectHash)]
#[binary_value(codec = "bincode")]
pub struct CryptoTransaction {
    pub nonce: u64,
    pub from: std::string::String,
    pub to: std::string::String,
    pub fxn_call: std::string::String,
    pub amount: u64,
}

pub trait TransactionTrait<T> {
    // generate trait is only for testing purpose
    fn generate(kp: &KeypairType) -> T;
    fn validate(&self) -> bool;
    fn sign(&self, kp: &KeypairType) -> Vec<u8>;
    fn get_hash(&self) -> exonum_crypto::Hash;
}

impl TransactionTrait<CryptoTransaction> for CryptoTransaction {
    fn validate(&self) -> bool {
        // add application validation logic if needed
        true
    }

    fn sign(&self, kp: &KeypairType) -> Vec<u8> {
        let ser_txn: Vec<u8> = match serialize(&self) {
            Result::Ok(value) => value,
            Result::Err(_) => vec![0],
        };
        let sign = Keypair::sign(&kp, &ser_txn);
        sign
    }

    fn generate(kp: &KeypairType) -> CryptoTransaction {
        let from: String = hex::encode(kp.public().encode());
        let to_add_kp = Keypair::generate();
        let to: String = hex::encode(to_add_kp.public().encode());
        CryptoTransaction {
            nonce: 0,
            from,
            to,
            amount: 32,
            fxn_call: String::from("transfer"),
        }
    }

    fn get_hash(&self) -> Hash {
        self.object_hash()
    }
}

impl TransactionTrait<SignedTransaction> for SignedTransaction {
    fn validate(&self) -> bool {
        let txn: CryptoTransaction = match deserialize(&self.txn) {
            Result::Ok(value) => value,
            Result::Err(_) => return false,
        };
        PublicKey::verify_from_encoded_pk(&txn.from, &self.txn, &self.signature.as_ref())
    }

    fn sign(&self, kp: &KeypairType) -> Vec<u8> {
        Keypair::sign(&kp, &self.txn)
    }

    fn generate(kp: &KeypairType) -> SignedTransaction {
        let from: String = hex::encode(kp.public().encode());
        let to_add_kp = Keypair::generate();
        let to: String = hex::encode(to_add_kp.public().encode());
        let txn: CryptoTransaction = CryptoTransaction {
            nonce: 0,
            from,
            to,
            amount: 32,
            fxn_call: String::from("transfer"),
        };
        let txn_sign = txn.sign(&kp);
        let mut header = HashMap::default();
        let time_stamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_micros();
        header.insert("timestamp".to_string(), time_stamp.to_string());
        let serialized_txn: Vec<u8> = match serialize(&txn) {
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

    fn get_hash(&self) -> Hash {
        self.object_hash()
    }
}
