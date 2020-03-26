use exonum_crypto::Hash;
use exonum_merkledb::{access::Access, ProofMapIndex};
use utils::keypair::KeypairType;

pub trait TransactionTrait<T> {
    // generate trait is only for testing purpose
    fn generate(kp: &KeypairType) -> T;
    fn validate(&self) -> bool;
    fn sign(&self, kp: &KeypairType) -> Vec<u8>;
    fn get_hash(&self) -> exonum_crypto::Hash;
}

pub trait StateTraits<T: Access, StateObj, TransactionObj> {
    fn execute(
        &self,
        state_trie: &mut ProofMapIndex<T::Base, String, StateObj>,
        txn_trie: &mut ProofMapIndex<T::Base, Hash, TransactionObj>,
    ) -> bool;
}

pub trait PoolTrait<T: Access, StateObj, TransactionObj> {
    fn execute_transactions(
        &self,
        state_trie: &mut ProofMapIndex<T::Base, String, StateObj>,
        txn_trie: &mut ProofMapIndex<T::Base, Hash, TransactionObj>,
    ) -> Vec<Hash>;

    fn update_transactions(
        &self,
        state_trie: &mut ProofMapIndex<T::Base, String, StateObj>,
        txn_trie: &mut ProofMapIndex<T::Base, Hash, TransactionObj>,
        hash_vec: &Vec<Hash>,
    ) -> bool;
}
