use super::signed_transaction::SignedTransaction;
use super::state::State;
use exonum_crypto::Hash;
use exonum_merkledb::access::Access;

pub trait StateContext {
    fn put(&mut self, key: &String, state: State);
    fn get(&self, key: &String) -> Option<State>;
    fn contains(&self, key: &String) -> bool;
    fn put_txn(&mut self, key: &Hash, txn: SignedTransaction);
    fn get_txn(&self, key: &Hash) -> Option<SignedTransaction>;
    fn contains_txn(&self, key: &Hash) -> bool;
}

pub trait AppHandler {
    fn execute(&self, txn: &SignedTransaction, state_context: &mut dyn StateContext) -> bool;
    fn name(&self) -> String;
}

pub trait PoolTrait<T: Access, StateObj, TransactionObj> {
    fn execute_transactions(&self, state_context: &mut dyn StateContext) -> Vec<Hash>;
    fn update_transactions(
        &self,
        state_context: &mut dyn StateContext,
        hash_vec: &Vec<Hash>,
    ) -> bool;
}
