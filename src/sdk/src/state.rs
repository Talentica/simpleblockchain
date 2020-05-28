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

#[cfg(test)]
mod test_sdk_state {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;
    use exonum_merkledb::ObjectHash;
    use utils::serializer::{deserialize, serialize};

    #[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default, BinaryValue, ObjectHash)]
    #[binary_value(codec = "bincode")]
    pub struct MockData {
        pub mock_field: u64,
    }

    // fn to test state operations
    #[test]
    fn test_state_operations() {
        let mut state: State = State::new();
        let state_data: MockData = MockData { mock_field: 11 };
        let state_storage: MockData = MockData { mock_field: 12 };
        let state_code: MockData = MockData { mock_field: 13 };

        state.set_data(&serialize(&state_data).unwrap());
        state.set_storage_root(state_storage.object_hash());
        state.set_code_hash(state_code.object_hash());

        assert_eq!(
            deserialize::<MockData>(state.get_data()).unwrap(),
            state_data
        );
        assert_eq!(state.get_storage_root(), state_storage.object_hash());
        assert_eq!(state.get_code_hash(), state_code.object_hash());
    }
}
