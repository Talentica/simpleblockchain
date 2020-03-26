extern crate utils;
use crate::state::State;
pub use crate::user_messages::{CryptoTransaction, SignedTransaction};
use exonum_crypto::Hash;
use exonum_merkledb::{
    access::{Access, RawAccessMut},
    ObjectHash, ProofMapIndex,
};
pub use generic_traits::traits::{StateTraits, TransactionTrait};
use std::collections::HashMap;
use std::convert::AsRef;
use std::time::SystemTime;
use utils::keypair::{CryptoKeypair, Keypair, KeypairType, PublicKey, Verify};
use utils::serializer::serialize;

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
        match &self.txn {
            Some(txn) => {
                let ser_txn = serialize(&txn);
                PublicKey::verify_from_encoded_pk(&txn.from, &ser_txn, &self.signature.as_ref())
            }
            None => false,
        }
    }

    fn sign(&self, kp: &KeypairType) -> Vec<u8> {
        match &self.txn {
            Some(txn) => {
                let ser_txn = serialize(&txn);
                let sign = Keypair::sign(&kp, &ser_txn);
                sign
            }
            None => Vec::new(),
        }
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
        SignedTransaction {
            txn: Some(txn),
            signature: txn_sign,
            header,
        }
    }

    fn get_hash(&self) -> Hash {
        self.object_hash()
    }
}

impl<T: Access> StateTraits<T, State, SignedTransaction> for SignedTransaction
where
    T::Base: RawAccessMut,
{
    fn execute(
        &self,
        state_trie: &mut ProofMapIndex<T::Base, String, State>,
        txn_trie: &mut ProofMapIndex<T::Base, Hash, SignedTransaction>,
    ) -> bool {
        let mut flag: bool = false;
        if self.validate() {
            match &self.txn {
                Some(txn) => {
                    let crypto_txn = &txn.clone() as &dyn ModuleTraits<T>;
                    if txn.fxn_call == String::from("transfer") {
                        flag = crypto_txn.transfer(state_trie);
                    } else if txn.fxn_call == String::from("mint") {
                        flag = crypto_txn.mint(state_trie);
                    } else {
                    }
                }
                None => {}
            }
        }
        if flag {
            txn_trie.put(&self.get_hash(), self.clone());
            flag
        } else {
            false
        }
    }
}

pub trait ModuleTraits<T: Access>
where
    T::Base: RawAccessMut,
{
    fn transfer(&self, state_trie: &mut ProofMapIndex<T::Base, String, State>) -> bool;
    fn mint(&self, state_trie: &mut ProofMapIndex<T::Base, String, State>) -> bool;
}

impl<T: Access> ModuleTraits<T> for CryptoTransaction
where
    T::Base: RawAccessMut,
{
    fn transfer(&self, state_trie: &mut ProofMapIndex<T::Base, String, State>) -> bool {
        if self.validate() {
            if state_trie.contains(&self.from) {
                let mut from_wallet: State = state_trie.get(&self.from).unwrap();
                if from_wallet.get_balance() > self.amount {
                    if state_trie.contains(&self.to) {
                        let mut to_wallet: State = state_trie.get(&self.to).unwrap();
                        to_wallet.add_balance(self.amount);
                        state_trie.put(&self.to.clone(), to_wallet);
                    } else {
                        let mut to_wallet = State::new();
                        to_wallet.add_balance(self.amount);
                        state_trie.put(&self.to.clone(), to_wallet);
                    }
                    from_wallet.deduct_balance(self.amount);
                    from_wallet.increase_nonce();
                    state_trie.put(&self.from.clone(), from_wallet);
                    return true;
                }
            }
        }
        false
    }

    fn mint(&self, state_trie: &mut ProofMapIndex<T::Base, String, State>) -> bool {
        if self.validate() {
            if state_trie.contains(&self.to) {
                let mut to_wallet: State = state_trie.get(&self.to).unwrap();
                to_wallet.add_balance(self.amount);
                state_trie.put(&self.to.clone(), to_wallet);
            } else {
                let mut to_wallet = State::new();
                to_wallet.add_balance(self.amount);
                state_trie.put(&self.to.clone(), to_wallet);
            }
            return true;
        }
        false
    }
}
