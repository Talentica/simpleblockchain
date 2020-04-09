extern crate utils;
use exonum_crypto::Hash;
use std::collections::BTreeMap;
use std::convert::AsRef;
use utils::serializer::{Deserialize, Serialize};

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
pub struct State {
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

impl State {
    pub fn new() -> State {
        State {
            tokens: BTreeMap::new(),
            pkg_no: BTreeMap::new(),
            file_hash: BTreeMap::new(),
            pending_view: BTreeMap::new(),
            confirmation_list: BTreeMap::new(),
        }
    }

    pub fn set_hash(&mut self, token_hash: Hash, file_hash: Hash) -> bool {
        if !self.file_hash.contains_key(&token_hash) {
            self.file_hash.insert(token_hash, file_hash);
            return true;
        }
        false
    }

    pub fn check_hash(&self, token_hash: Hash, file_hash: Hash) -> bool {
        if let Some(hash_value) = self.file_hash.get(&token_hash) {
            if hash_value.clone() == file_hash {
                return true;
            }
        }
        false
    }

    pub fn add_nft_token(&mut self, token_hash: Hash, token: NFTToken) -> bool {
        if !self.tokens.contains_key(&token_hash) {
            self.tokens.insert(token_hash, token);
            return true;
        }
        false
    }

    pub fn replace_nft_token(&mut self, token_hash: Hash, token: NFTToken) {
        self.tokens.insert(token_hash, token);
    }

    pub fn get_nft_token(&self, token_hash: Hash) -> Option<&NFTToken> {
        self.tokens.get(&token_hash)
    }

    pub fn set_pkg_list(&mut self, pkg_no: &String, doc_list: &Vec<Hash>) -> bool {
        match self.pkg_no.get(pkg_no) {
            Some(_list) => false,
            None => {
                self.pkg_no.insert(pkg_no.clone(), doc_list.clone());
                true
            }
        }
    }

    pub fn get_pkg_list(&self, pkg_no: &String) -> Option<&Vec<Hash>> {
        self.pkg_no.get(pkg_no)
    }

    pub fn add_into_confirmation_list(
        &mut self,
        to_address: &String,
        doc_list: &Vec<Hash>,
    ) -> bool {
        if let Some(list) = self.confirmation_list.get(to_address) {
            let mut mut_list: Vec<Hash> = list.clone();
            info!("Matched {:?}!", list);
            for each in doc_list {
                mut_list.push(each.clone());
            }
            self.confirmation_list.insert(to_address.clone(), mut_list);
        } else {
            let mut mut_list: Vec<Hash> = Vec::new();
            for each in doc_list {
                mut_list.push(each.clone());
            }
            self.confirmation_list.insert(to_address.clone(), mut_list);
        }
        true
    }

    pub fn update_confirmation_list(&mut self, to_address: &String, doc_list: &Vec<Hash>) {
        self.confirmation_list
            .insert(to_address.clone(), doc_list.clone());
    }

    pub fn get_confirmation_waiting_list(&self, to_address: &String) -> Option<&Vec<Hash>> {
        self.confirmation_list.get(to_address)
    }

    pub fn add_pkg_no_for_review(&mut self, to_address: &String, pkg_no: &String) {
        if let Some(list) = self.pending_view.get(to_address) {
            let mut mut_list: Vec<String> = list.clone();
            mut_list.push(pkg_no.clone());
            self.pending_view.insert(to_address.clone(), mut_list);
        } else {
            let mut mut_list: Vec<String> = Vec::new();
            mut_list.push(pkg_no.clone());
            self.pending_view.insert(to_address.clone(), mut_list);
        }
    }

    pub fn remove_pkg_no_from_review_list(&mut self, to_address: &String, pkg_no: &String) -> bool {
        match self.pending_view.get(to_address) {
            Some(list) => {
                let mut list = list.clone();
                let index = match list.iter().position(|r| r == pkg_no) {
                    Some(index) => index,
                    None => return false,
                };
                list.remove(index);
                self.pending_view.insert(to_address.clone(), list);
                true
            }
            None => false,
        }
    }

    pub fn get_pkg_review_pending_list(&self, to_address: &String) -> Option<&Vec<String>> {
        self.pending_view.get(to_address)
    }
}

#[cfg(test)]
mod test_state {

    #[test]
    pub fn test_states() {
        use super::*;
        use exonum_merkledb::ObjectHash;
        let mut state = State::new();
        let default_token = NFTToken::default();
        info!("{:?}", state);
        let token_hash = default_token.object_hash();
        state.tokens.insert(token_hash, default_token);
        info!("{:?}", state);
        info!("{:?}", state.set_hash(token_hash.clone(), Hash::zero()));
        info!(
            "{:?}",
            state.check_hash(token_hash.clone(), token_hash.clone())
        );
    }
}
