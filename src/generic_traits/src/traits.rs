use exonum_merkledb::{ObjectAccess, ProofMapIndex, RefMut};
use utils::keypair::KeypairType;

pub trait TransactionTrait<T> {
    // generate trait is only for testing purpose
    fn generate(kp: &KeypairType) -> T;
    fn validate(&self) -> bool;
    fn sign(&self, kp: &KeypairType) -> Vec<u8>;
}

pub trait StateTraits<T, S>
where
    T: ObjectAccess,
{
    fn execute(&self, fork: &mut RefMut<ProofMapIndex<T, String, S>>) -> bool;
}
