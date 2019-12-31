extern crate utils;

use utils::keypair::{CryptoKeypair, Keypair, KeypairType, PublicKey, Verify};
use utils::serializer::{serialize, serialize_hash256, Deserialize, Serialize};

pub trait Txn<T, U> {
    fn generate() -> T;
    fn validate(&self) -> bool;
    fn sign(&self, kp: &U) -> Vec<u8>;
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Transaction {
    party_a: String,
    party_b: String,
    amount: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SignedTransaction {
    txn: Transaction,
    signature: Vec<u8>,
}

impl Txn<Transaction, KeypairType> for Transaction {
    fn validate(&self) -> bool {
        unimplemented!();
    }

    fn sign(&self, kp: &KeypairType) -> Vec<u8> {
        let ser_txn = serialize(&self);
        let sign = Keypair::sign(&kp, &ser_txn);
        sign
    }

    fn generate() -> Transaction {
        let party_a_kp = Keypair::generate();
        let party_a: String = hex::encode(party_a_kp.public().encode());
        let party_b_kp = Keypair::generate();
        let party_b: String = hex::encode(party_b_kp.public().encode());
        Transaction {
            party_a,
            party_b,
            amount: 32,
        }
    }
}

impl Txn<SignedTransaction, KeypairType> for SignedTransaction {
    fn validate(&self) -> bool {
        let ser_txn = serialize(&self.txn);
        PublicKey::verify_from_encoded_pk(&self.txn.party_a, &ser_txn, &self.signature.as_ref())
        // PublicKey::verify_from_encoded_pk(&self.txn.party_a, signing_string.as_bytes(), &self.signature.as_ref())
    }

    fn sign(&self, kp: &KeypairType) -> Vec<u8> {
        let ser_txn = serialize(&self.txn);
        let sign = Keypair::sign(&kp, &ser_txn);
        sign
    }

    fn generate() -> SignedTransaction {
        let party_a_kp = Keypair::generate();
        let party_a: String = hex::encode(party_a_kp.public().encode());
        let party_b_kp = Keypair::generate();
        let party_b: String = hex::encode(party_b_kp.public().encode());
        let txn: Transaction = Transaction {
            party_a,
            party_b,
            amount: 32,
        };
        let txn_sign = txn.sign(&party_a_kp);
        SignedTransaction {
            txn,
            signature: txn_sign,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TransactionPool {
    pub pool: Vec<SignedTransaction>,
}

pub trait TxnPool {
    fn new() -> Self;
    fn execute(&mut self) -> Vec<SignedTransaction>;
    fn add(&mut self, signed_txn: SignedTransaction);
    fn pop(&mut self) -> SignedTransaction;
    fn length(&self) -> usize;
}

impl TxnPool for TransactionPool {
    fn new() -> Self {
        Self {
            pool: Vec::<SignedTransaction>::new(),
        }
    }

    fn execute(&mut self) -> Vec<SignedTransaction> {
        let mut temp_vec = Vec::<SignedTransaction>::with_capacity(10);
        while temp_vec.len() < 10 && self.pool.len() > 0 {
            let txn = self.pop();
            temp_vec.push(txn);
        }
        temp_vec
    }

    fn add(&mut self, signed_txn: SignedTransaction) {
        if signed_txn.validate() {
            self.pool.push(signed_txn);
        }
        // unimplemented!();
    }

    fn pop(&mut self) -> SignedTransaction {
        if self.pool.len() > 0 {
            self.pool.pop().unwrap()
        } else {
            panic!("poping txn from empty txn_pool");
        }
        // unimplemented!();
    }

    fn length(&self) -> usize {
        self.pool.len()
    }
}

#[cfg(test)]
mod tests_transactions {

    #[test]
    pub fn main_transaction() {
        let mut transaction_pool = TransactionPool::new();
        transaction_pool.add(SignedTransaction::generate());
        transaction_pool.add(SignedTransaction::generate());

        let exexuted_pool: Vec<SignedTransaction> = transaction_pool.execute();
        println!("{:?}", exexuted_pool);
    }
}
