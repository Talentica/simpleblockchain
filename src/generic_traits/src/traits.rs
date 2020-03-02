use exonum_merkledb::{access::Access, ProofMapIndex};
use utils::keypair::KeypairType;

pub trait TransactionTrait<T> {
    // generate trait is only for testing purpose
    fn generate(kp: &KeypairType) -> T;
    fn validate(&self) -> bool;
    fn sign(&self, kp: &KeypairType) -> Vec<u8>;
}

pub trait StateTraits<T: Access, S> {
    fn execute(&self, fork: &mut ProofMapIndex<T::Base, String, S>) -> bool;
}
