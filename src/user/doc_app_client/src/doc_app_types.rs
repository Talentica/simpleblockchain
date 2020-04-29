extern crate utils;
use exonum_crypto::Hash;
use exonum_merkledb::ObjectHash;
pub use sdk::signed_transaction::SignedTransaction;
use std::collections::{BTreeMap, HashMap};
use std::convert::AsRef;
use std::time::SystemTime;
use utils::keypair::{CryptoKeypair, Keypair, KeypairType, PublicKey, Verify};
use utils::serializer::{deserialize, serialize, Deserialize, Serialize};
pub const APPNAME: &str = "Document_Review";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, BinaryValue, ObjectHash)]
#[binary_value(codec = "bincode")]
pub struct CryptoTransaction {
    pub from: std::string::String,
    pub fxn_call: std::string::String,
    pub payload: std::vec::Vec<DataTypes>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, BinaryValue, ObjectHash)]
#[binary_value(codec = "bincode")]
pub enum DataTypes {
    BoolVal(bool),
    IntVal(i32),
    HashVal(Hash),
    StringVal(String),
    VecHashVal(Vec<Hash>),
    VecStringVal(Vec<String>),
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
        let ser_txn = serialize(&self);
        let sign = Keypair::sign(&kp, &ser_txn);
        sign
    }

    fn generate(kp: &KeypairType) -> CryptoTransaction {
        let from: String = hex::encode(kp.public().encode());
        let mut payload: Vec<DataTypes> = Vec::new();
        let token = NFTToken::default();
        let token_hash = token.object_hash();
        payload.push(DataTypes::HashVal(token_hash));
        payload.push(DataTypes::HashVal(Hash::zero()));
        CryptoTransaction {
            from,
            fxn_call: String::from("set_hash"),
            payload,
        }
    }

    fn get_hash(&self) -> Hash {
        self.object_hash()
    }
}

impl TransactionTrait<SignedTransaction> for SignedTransaction {
    fn validate(&self) -> bool {
        let txn = deserialize::<CryptoTransaction>(&self.txn);
        PublicKey::verify_from_encoded_pk(&txn.from, &self.txn, &self.signature.as_ref())
    }

    fn sign(&self, kp: &KeypairType) -> Vec<u8> {
        Keypair::sign(&kp, &self.txn)
    }

    fn generate(kp: &KeypairType) -> SignedTransaction {
        let from: String = hex::encode(kp.public().encode());
        let mut payload: Vec<DataTypes> = Vec::new();
        let token = NFTToken::default();
        let token_hash = token.object_hash();
        payload.push(DataTypes::HashVal(token_hash));
        payload.push(DataTypes::HashVal(Hash::zero()));
        let txn = CryptoTransaction {
            from,
            fxn_call: String::from("set_hash"),
            payload,
        };
        let txn_sign = txn.sign(&kp);
        let mut header = HashMap::default();
        let time_stamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_micros();
        header.insert("timestamp".to_string(), time_stamp.to_string());
        SignedTransaction {
            txn: serialize(&txn),
            app_name: String::from(APPNAME),
            signature: txn_sign,
            header,
        }
    }

    fn get_hash(&self) -> Hash {
        self.object_hash()
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinaryValue, ObjectHash)]
#[binary_value(codec = "bincode")]
pub enum DocStatus {
    Approved,
    Rejected,
    Submitted,
    Created,
    Publish,
}

impl Default for DocStatus {
    fn default() -> Self {
        DocStatus::Created
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default, BinaryValue, ObjectHash)]
#[binary_value(codec = "bincode")]
pub struct NFTToken {
    pub super_owner: String,
    pub owner: String,
    pub pkg_no: String,
    pub status: DocStatus,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default, BinaryValue, ObjectHash)]
#[binary_value(codec = "bincode")]
pub struct DocState {
    // NFT_TOKEN_HASH => NFT_TOKEN
    tokens: BTreeMap<Hash, NFTToken>,
    // PKG_NO => NFT_TOKEN_HASH_LIST
    pkg_no: BTreeMap<String, Vec<Hash>>,
    // NFT_TOKEN => FILE_HASH
    file_hash: BTreeMap<Hash, Hash>,
    // PUBLIC_ADDRESS => PKG_NO_LIST
    pending_view: BTreeMap<String, Vec<String>>,
    // PUBLIC_ADDRESS => NFT_LIST (NEDDED ONLY FOR BUSINESS LOGIC ALGORITHM)
    confirmation_list: BTreeMap<String, Vec<Hash>>,
}
