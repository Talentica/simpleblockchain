extern crate utils;
use crate::state::State;
pub use exonum_merkledb::{
    impl_object_hash_for_binary_value, BinaryValue, ObjectAccess, ObjectHash, ProofMapIndex, RefMut,
};
use generic_traits::traits::{StateTraits, TransactionTrait};
use utils::keypair::{CryptoKeypair, Keypair, KeypairType};
use utils::serializer::{serialize, Deserialize, Serialize};
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum DataTypes {
    String,
    Vec(String),
    Number(u64),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CryptoTransaction {
    /* TODO:
    // Priority for a transaction. Additive. Higher is better.
    pub type TransactionPriority = u64;
    // Minimum number of blocks a transaction will remain valid for.
    // `TransactionLongevity::max_value()` means "forever".
    pub type TransactionLongevity = u64;
    // Tag for a transaction. No two transactions with the same tag should be placed on-chain.
    pub type TransactionTag = Vec<u8>;
    */
    pub nonce: u64,
    pub from: String,
    pub to: String,
    pub fxn_call: String,
    // TODO:: payload is for fxn_call variables
    // update payload type in future as per requirement
    pub payload: Vec<DataTypes>,
    pub amount: u64,
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
        let to_add_kp = Keypair::generate();
        let to: String = hex::encode(to_add_kp.public().encode());
        CryptoTransaction {
            nonce: 0,
            from,
            to,
            amount: 32,
            fxn_call: String::from("transfer"),
            payload: vec![],
        }
    }
}

impl<T: ObjectAccess> StateTraits<T, State> for CryptoTransaction {
    fn execute(&self, state_trie: &mut RefMut<ProofMapIndex<T, String, State>>) -> bool {
        if self.fxn_call == String::from("transfer") {
            self.transfer(state_trie)
        } else {
            false
        }
    }
}

pub trait ModuleTraits<T>
where
    T: ObjectAccess,
{
    fn transfer(&self, state_trie: &mut RefMut<ProofMapIndex<T, String, State>>) -> bool;
}

impl<T: ObjectAccess> ModuleTraits<T> for CryptoTransaction {
    fn transfer(&self, state_trie: &mut RefMut<ProofMapIndex<T, String, State>>) -> bool {
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
}
