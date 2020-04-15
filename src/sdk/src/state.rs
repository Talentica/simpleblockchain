extern crate utils;
use exonum_crypto::Hash;
use std::convert::AsRef;

use utils::serializer::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default, BinaryValue, ObjectHash)]
#[binary_value(codec = "bincode")]
pub struct State {
    data: ::std::vec::Vec<u8>,
    storage_root: Hash,
    code_hash: Hash,
}

impl State {
    pub fn new() -> State {
        State {
            data: Vec::new(),
            storage_root: Hash::zero(),
            code_hash: Hash::zero(),
        }
    }

    pub fn get_data(&self) -> &Vec<u8> {
        &self.data
    }

    pub fn get_storage_root(&self) -> Hash {
        self.storage_root
    }

    pub fn get_code_hash(&self) -> Hash {
        self.code_hash
    }

    pub fn set_data(&mut self, new_data: &Vec<u8>) {
        self.data = new_data.clone();
    }

    pub fn set_storage_root(&mut self, new_storage_root: Hash) {
        self.storage_root = new_storage_root;
    }

    pub fn set_code_hash(&mut self, new_code_hash: Hash) {
        self.code_hash = new_code_hash;
    }
}
