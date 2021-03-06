extern crate utils;
use crate::state::{DocStatus, NFTToken, State as DocState};
use crate::user_messages::{CryptoTransaction, DataTypes};
use exonum_crypto::Hash;
use exonum_merkledb::ObjectHash;
pub use sdk::signed_transaction::SignedTransaction;
use sdk::state::State;
use sdk::traits::{AppHandler, StateContext};
use std::collections::HashMap;
use std::convert::AsRef;
use std::time::SystemTime;
use utils::keypair::{CryptoKeypair, Keypair, KeypairType, PublicKey, Verify};
use utils::logger::logger_init_from_yml;
use utils::serializer::{deserialize, serialize};

const APPNAME: &str = "Document_Review";

pub const STATE_KEY: &str = "34132aec80149c4538bad4a15995ddf6a89d4ed5e39f0060e8466f6ba4dc9ceb";

trait StateTraits {
    fn execute(&self, state_context: &mut dyn StateContext) -> bool;
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

impl StateTraits for SignedTransaction {
    fn execute(&self, state_context: &mut dyn StateContext) -> bool {
        let mut flag: bool = false;
        if self.validate() {
            let txn: CryptoTransaction = match deserialize(&self.txn) {
                Result::Ok(value) => value,
                Result::Err(_) => return false,
            };
            let crypto_txn = &txn.clone() as &dyn ModuleTraits;
            if txn.fxn_call == String::from("set_hash") {
                flag = crypto_txn.set_hash(state_context);
            } else if txn.fxn_call == String::from("add_doc") {
                flag = crypto_txn.add_doc(state_context);
            } else if txn.fxn_call == String::from("transfer_sc") {
                flag = crypto_txn.transfer_sc(state_context);
            } else if txn.fxn_call == String::from("set_pkg_no") {
                flag = crypto_txn.set_pkg_no(state_context);
            } else if txn.fxn_call == String::from("transfer_for_review") {
                flag = crypto_txn.transfer_for_review(state_context);
            } else if txn.fxn_call == String::from("review_docs") {
                flag = crypto_txn.review_docs(state_context);
            } else if txn.fxn_call == String::from("publish_docs") {
                flag = crypto_txn.publish_docs(state_context);
            } else {
            }
        }
        state_context.put_txn(&self.get_hash(), self.clone());
        flag
    }
}

pub trait ModuleTraits {
    fn set_hash(&self, state_context: &mut dyn StateContext) -> bool;
    fn add_doc(&self, state_context: &mut dyn StateContext) -> bool;
    fn transfer_sc(&self, state_context: &mut dyn StateContext) -> bool;
    fn set_pkg_no(&self, state_context: &mut dyn StateContext) -> bool;
    fn transfer_for_review(&self, state_context: &mut dyn StateContext) -> bool;
    fn review_docs(&self, state_context: &mut dyn StateContext) -> bool;
    fn publish_docs(&self, state_context: &mut dyn StateContext) -> bool;
}

impl ModuleTraits for CryptoTransaction {
    fn set_hash(&self, state_context: &mut dyn StateContext) -> bool {
        let expected_payload_size: usize = 2;
        let mut expected_payload: Vec<DataTypes> = Vec::with_capacity(expected_payload_size);
        expected_payload.push(DataTypes::HashVal(Hash::zero()));
        expected_payload.push(DataTypes::HashVal(Hash::zero()));
        if self.payload.len() != expected_payload_size {
            return false;
        }
        for index in 0..expected_payload_size {
            let required_type = expected_payload.get(index).unwrap();
            let given_type = self.payload.get(index).unwrap();
            match (required_type, given_type) {
                (DataTypes::HashVal(_a), DataTypes::HashVal(_d)) => {}
                _ => return false,
            };
        }
        let token_id: Hash = match self.payload.get(0).unwrap() {
            DataTypes::HashVal(value) => value.clone(),
            _ => return false,
        };
        let file_hash: Hash = match self.payload.get(1).unwrap() {
            DataTypes::HashVal(value) => value.clone(),
            _ => return false,
        };
        let mut app_state: State = match state_context.get(&STATE_KEY.to_string()) {
            Some(state) => state,
            None => {
                let doc_state: DocState = DocState::new();
                let mut state: State = State::new();
                let serialized_doc_state: Vec<u8> = match serialize(&doc_state) {
                    Result::Ok(value) => value,
                    Result::Err(_) => return false,
                };
                state.set_data(&serialized_doc_state);
                state
            }
        };
        let mut state: DocState = match deserialize(app_state.get_data().as_slice()) {
            Result::Ok(value) => value,
            Result::Err(_) => return false,
        };
        let flag: bool = state.set_hash(token_id, file_hash);
        if !flag {
            info!("operation set_hash failed");
            return false;
        }
        let serialized_state: Vec<u8> = match serialize(&state) {
            Result::Ok(value) => value,
            Result::Err(_) => return false,
        };
        app_state.set_data(&serialized_state);
        state_context.put(&STATE_KEY.to_string(), app_state);
        info!("operation set_hash done");
        true
    }

    fn add_doc(&self, state_context: &mut dyn StateContext) -> bool {
        let expected_payload_size: usize = 1;
        let mut expected_payload: Vec<DataTypes> = Vec::with_capacity(expected_payload_size);
        expected_payload.push(DataTypes::VecHashVal(vec![]));
        if self.payload.len() != expected_payload_size {
            return false;
        }
        for index in 0..expected_payload_size {
            let required_type = expected_payload.get(index).unwrap();
            let given_type = self.payload.get(index).unwrap();
            match (required_type, given_type) {
                (DataTypes::IntVal(_a), DataTypes::IntVal(_d)) => {}
                (DataTypes::HashVal(_a), DataTypes::HashVal(_d)) => {}
                (DataTypes::StringVal(_a), DataTypes::StringVal(_d)) => {}
                (DataTypes::VecHashVal(_a), DataTypes::VecHashVal(_d)) => {}
                (DataTypes::VecStringVal(_a), DataTypes::VecStringVal(_d)) => {}
                _ => return false,
            };
        }
        let token_ids: Vec<Hash> = match self.payload.get(0).unwrap() {
            DataTypes::VecHashVal(value) => value.clone(),
            _ => return false,
        };
        let mut app_state: State = match state_context.get(&STATE_KEY.to_string()) {
            Some(state) => state,
            None => {
                let doc_state: DocState = DocState::new();
                let mut state: State = State::new();
                let serialized_doc_state: Vec<u8> = match serialize(&doc_state) {
                    Result::Ok(value) => value,
                    Result::Err(_) => return false,
                };
                state.set_data(&serialized_doc_state);
                state
            }
        };
        let mut state: DocState = match deserialize(app_state.get_data().as_slice()) {
            Result::Ok(value) => value,
            Result::Err(_) => return false,
        };
        for each in token_ids.iter() {
            let token: NFTToken = NFTToken {
                super_owner: self.from.clone(),
                owner: self.from.clone(),
                pkg_no: String::from(""),
                status: DocStatus::Created,
            };
            let flag: bool = state.add_nft_token(each.clone(), token);
            if !flag {
                return false;
            }
        }
        let serialized_state: Vec<u8> = match serialize(&state) {
            Result::Ok(value) => value,
            Result::Err(_) => return false,
        };
        app_state.set_data(&serialized_state);
        state_context.put(&STATE_KEY.to_string(), app_state);
        true
    }

    fn transfer_sc(&self, state_context: &mut dyn StateContext) -> bool {
        let expected_payload_size: usize = 2;
        let mut expected_payload: Vec<DataTypes> = Vec::with_capacity(expected_payload_size);
        expected_payload.push(DataTypes::VecHashVal(vec![]));
        expected_payload.push(DataTypes::StringVal(String::default()));
        if self.payload.len() != expected_payload_size {
            return false;
        }
        for index in 0..expected_payload_size {
            let required_type = expected_payload.get(index).unwrap();
            let given_type = self.payload.get(index).unwrap();
            match (required_type, given_type) {
                (DataTypes::StringVal(_a), DataTypes::StringVal(_d)) => {}
                (DataTypes::VecHashVal(_a), DataTypes::VecHashVal(_d)) => {}
                _ => return false,
            };
        }
        let token_ids: Vec<Hash> = match self.payload.get(0).unwrap() {
            DataTypes::VecHashVal(value) => value.clone(),
            _ => return false,
        };
        let to_address: String = match self.payload.get(1).unwrap() {
            DataTypes::StringVal(value) => value.clone(),
            _ => return false,
        };
        let mut app_state: State = match state_context.get(&STATE_KEY.to_string()) {
            Some(state) => state,
            None => return false,
        };
        let mut state: DocState = match deserialize(app_state.get_data().as_slice()) {
            Result::Ok(value) => value,
            Result::Err(_) => return false,
        };
        for each in token_ids.iter() {
            match state.get_nft_token(each.clone()) {
                Some(token) => {
                    if token.owner != self.from {
                        return false;
                    }
                }
                None => return false,
            }
        }
        state.add_into_confirmation_list(&to_address, &token_ids);
        let serialized_state: Vec<u8> = match serialize(&state) {
            Result::Ok(value) => value,
            Result::Err(_) => return false,
        };
        app_state.set_data(&serialized_state);
        state_context.put(&STATE_KEY.to_string(), app_state);
        true
    }

    fn set_pkg_no(&self, state_context: &mut dyn StateContext) -> bool {
        let expected_payload_size: usize = 2;
        let mut expected_payload: Vec<DataTypes> = Vec::with_capacity(expected_payload_size);
        expected_payload.push(DataTypes::VecHashVal(vec![]));
        expected_payload.push(DataTypes::StringVal(String::default()));
        if self.payload.len() != expected_payload_size {
            return false;
        }
        for index in 0..expected_payload_size {
            let required_type = expected_payload.get(index).unwrap();
            let given_type = self.payload.get(index).unwrap();
            match (required_type, given_type) {
                (DataTypes::StringVal(_a), DataTypes::StringVal(_d)) => {}
                (DataTypes::VecHashVal(_a), DataTypes::VecHashVal(_d)) => {}
                _ => return false,
            };
        }
        let token_ids: Vec<Hash> = match self.payload.get(0).unwrap() {
            DataTypes::VecHashVal(value) => value.clone(),
            _ => return false,
        };
        let pkg_no: String = match self.payload.get(1).unwrap() {
            DataTypes::StringVal(value) => value.clone(),
            _ => return false,
        };
        let mut app_state: State = match state_context.get(&STATE_KEY.to_string()) {
            Some(state) => state,
            None => return false,
        };
        let mut state: DocState = match deserialize(app_state.get_data().as_slice()) {
            Result::Ok(value) => value,
            Result::Err(_) => return false,
        };
        let mut waiting_list: Vec<Hash> = match state.get_confirmation_waiting_list(&self.from) {
            Some(list) => list.clone(),
            None => return false,
        };
        let mut token_map: HashMap<Hash, NFTToken> = HashMap::new();
        for each in token_ids.iter() {
            if !waiting_list.contains(each) {
                return false;
            }
            let token: NFTToken = match state.get_nft_token(each.clone()) {
                Some(token) => {
                    if token.status != DocStatus::Created {
                        return false;
                    }
                    token.clone()
                }
                None => return false,
            };
            token_map.insert(each.clone(), token);
        }
        for (token_hash, token) in token_map.into_iter() {
            let mut token: NFTToken = token.clone();
            token.status = DocStatus::Submitted;
            token.pkg_no = pkg_no.clone();
            state.replace_nft_token(token_hash.clone(), token);
            let index = waiting_list.iter().position(|&r| r == token_hash).unwrap();
            waiting_list.remove(index);
        }
        state.set_pkg_list(&pkg_no, &token_ids);
        state.update_confirmation_list(&self.from, &waiting_list);
        let serialized_state: Vec<u8> = match serialize(&state) {
            Result::Ok(value) => value,
            Result::Err(_) => return false,
        };
        app_state.set_data(&serialized_state);
        state_context.put(&STATE_KEY.to_string(), app_state);
        true
    }

    fn transfer_for_review(&self, state_context: &mut dyn StateContext) -> bool {
        let expected_payload_size: usize = 2;
        let mut expected_payload: Vec<DataTypes> = Vec::with_capacity(expected_payload_size);
        expected_payload.push(DataTypes::StringVal(String::default()));
        expected_payload.push(DataTypes::StringVal(String::default()));
        if self.payload.len() != expected_payload_size {
            return false;
        }
        for index in 0..expected_payload_size {
            let required_type = expected_payload.get(index).unwrap();
            let given_type = self.payload.get(index).unwrap();
            match (required_type, given_type) {
                (DataTypes::StringVal(_a), DataTypes::StringVal(_d)) => {}
                _ => return false,
            };
        }
        let pkg_no: String = match self.payload.get(0).unwrap() {
            DataTypes::StringVal(value) => value.clone(),
            _ => return false,
        };
        let reviewer_address: String = match self.payload.get(1).unwrap() {
            DataTypes::StringVal(value) => value.clone(),
            _ => return false,
        };
        let mut app_state: State = match state_context.get(&STATE_KEY.to_string()) {
            Some(state) => state,
            None => return false,
        };
        let mut state: DocState = match deserialize(app_state.get_data().as_slice()) {
            Result::Ok(value) => value,
            Result::Err(_) => return false,
        };
        let pkg_doc_list: Vec<Hash> = match state.get_pkg_list(&pkg_no) {
            Some(list) => list.clone(),
            None => return false,
        };
        for each in pkg_doc_list.iter() {
            match state.get_nft_token(each.clone()) {
                Some(token) => {
                    if token.status != DocStatus::Submitted {
                        return false;
                    }
                    if token.owner != self.from {
                        return false;
                    }
                }
                None => return false,
            };
        }
        state.add_pkg_no_for_review(&reviewer_address, &pkg_no);
        let serialized_state: Vec<u8> = match serialize(&state) {
            Result::Ok(value) => value,
            Result::Err(_) => return false,
        };
        app_state.set_data(&serialized_state);
        state_context.put(&STATE_KEY.to_string(), app_state);
        true
    }

    fn review_docs(&self, state_context: &mut dyn StateContext) -> bool {
        let expected_payload_size: usize = 2;
        let mut expected_payload: Vec<DataTypes> = Vec::with_capacity(expected_payload_size);
        expected_payload.push(DataTypes::StringVal(String::default()));
        expected_payload.push(DataTypes::BoolVal(false));
        if self.payload.len() != expected_payload_size {
            return false;
        }
        for index in 0..expected_payload_size {
            let required_type = expected_payload.get(index).unwrap();
            let given_type = self.payload.get(index).unwrap();
            match (required_type, given_type) {
                (DataTypes::StringVal(_a), DataTypes::StringVal(_d)) => {}
                (DataTypes::BoolVal(_a), DataTypes::BoolVal(_d)) => {}
                _ => return false,
            };
        }
        let pkg_no: String = match self.payload.get(0).unwrap() {
            DataTypes::StringVal(value) => value.clone(),
            _ => return false,
        };
        let response_bool: bool = match self.payload.get(1).unwrap() {
            DataTypes::BoolVal(value) => value.clone(),
            _ => return false,
        };
        let mut app_state: State = match state_context.get(&STATE_KEY.to_string()) {
            Some(state) => state,
            None => return false,
        };
        let mut state: DocState = match deserialize(app_state.get_data().as_slice()) {
            Result::Ok(value) => value,
            Result::Err(_) => return false,
        };
        match state.get_pkg_review_pending_list(&self.from) {
            Some(list) => {
                if !list.contains(&pkg_no) {
                    return false;
                }
            }
            None => return false,
        }
        let pkg_doc_list: Vec<Hash> = match state.get_pkg_list(&pkg_no) {
            Some(list) => list.clone(),
            None => return false,
        };
        for each in pkg_doc_list.iter() {
            match state.get_nft_token(each.clone()) {
                Some(token) => {
                    if token.status != DocStatus::Submitted {
                        return false;
                    }
                }
                None => return false,
            };
        }
        if response_bool {
            for each in pkg_doc_list.iter() {
                let mut token: NFTToken = state.get_nft_token(each.clone()).unwrap().clone();
                token.status = DocStatus::Approved;
                state.replace_nft_token(each.clone(), token);
            }
        } else {
            for each in pkg_doc_list.iter() {
                let mut token: NFTToken = state.get_nft_token(each.clone()).unwrap().clone();
                token.status = DocStatus::Rejected;
                state.replace_nft_token(each.clone(), token);
            }
        }
        state.remove_pkg_no_from_review_list(&self.from, &pkg_no);
        let serialized_state: Vec<u8> = match serialize(&state) {
            Result::Ok(value) => value,
            Result::Err(_) => return false,
        };
        app_state.set_data(&serialized_state);
        state_context.put(&STATE_KEY.to_string(), app_state);
        true
    }

    fn publish_docs(&self, state_context: &mut dyn StateContext) -> bool {
        let expected_payload_size: usize = 1;
        let mut expected_payload: Vec<DataTypes> = Vec::with_capacity(expected_payload_size);
        expected_payload.push(DataTypes::StringVal(String::default()));
        if self.payload.len() != expected_payload_size {
            return false;
        }
        for index in 0..expected_payload_size {
            let required_type = expected_payload.get(index).unwrap();
            let given_type = self.payload.get(index).unwrap();
            match (required_type, given_type) {
                (DataTypes::StringVal(_a), DataTypes::StringVal(_d)) => {}
                _ => return false,
            };
        }
        let pkg_no: String = match self.payload.get(0).unwrap() {
            DataTypes::StringVal(value) => value.clone(),
            _ => return false,
        };
        let mut app_state: State = match state_context.get(&STATE_KEY.to_string()) {
            Some(state) => state,
            None => return false,
        };
        let mut state: DocState = match deserialize(app_state.get_data().as_slice()) {
            Result::Ok(value) => value,
            Result::Err(_) => return false,
        };
        let pkg_doc_list: Vec<Hash> = match state.get_pkg_list(&pkg_no) {
            Some(list) => list.clone(),
            None => return false,
        };
        for each in pkg_doc_list.iter() {
            match state.get_nft_token(each.clone()) {
                Some(token) => {
                    if token.status != DocStatus::Approved {
                        return false;
                    }
                    if token.owner != self.from {
                        return false;
                    }
                }
                None => return false,
            };
        }
        for each in pkg_doc_list.iter() {
            let mut token: NFTToken = state.get_nft_token(each.clone()).unwrap().clone();
            token.status = DocStatus::Publish;
            state.replace_nft_token(each.clone(), token);
        }
        let serialized_state: Vec<u8> = match serialize(&state) {
            Result::Ok(value) => value,
            Result::Err(_) => return false,
        };
        app_state.set_data(&serialized_state);
        state_context.put(&STATE_KEY.to_string(), app_state);
        true
    }
}

pub struct CryptoApp {
    name: String,
}

impl CryptoApp {
    pub fn new(s: &String) -> CryptoApp {
        logger_init_from_yml("log.yml");
        CryptoApp { name: s.clone() }
    }
}

impl AppHandler for CryptoApp {
    fn execute(&self, txn: &SignedTransaction, state_context: &mut dyn StateContext) -> bool {
        let st = txn as &dyn StateTraits;
        st.execute(state_context)
    }

    fn name(&self) -> String {
        self.name.clone()
    }
}

#[no_mangle]
pub fn register_app() -> Box<dyn AppHandler + Send> {
    Box::new(CryptoApp::new(&String::from(APPNAME)))
}

#[cfg(test)]
mod test_state {

    #[test]
    fn check_tzn() {
        use super::*;
        let to_add_kp = Keypair::generate();
        let _to: String = hex::encode(to_add_kp.public().encode());

        let signed_transaction: SignedTransaction = SignedTransaction::generate(&to_add_kp);
        info!("{:?}", signed_transaction);
        info!("{:?}", signed_transaction.validate());
    }
}
