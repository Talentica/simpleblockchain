extern crate utils;
mod block;
mod transaction;
use std::collections::BTreeMap;
use crate::transaction::{SignedTransaction, Txn};
use std::time::{Instant};
use utils::keypair::{CryptoKeypair, Keypair};

fn test_btreemap(){
    let mut btreemap = BTreeMap::<Instant, SignedTransaction>::new();
    let instant_one = Instant::now();
    let kp = Keypair::generate();
    let one = SignedTransaction::generate(&kp);
    let instant_two = Instant::now();
    println!("{:?} {:?}", instant_one, instant_two);
    btreemap.insert(instant_two, one.clone());
    btreemap.insert(instant_one, one.clone());

    for (key, _value) in btreemap.iter() {
        println!("{:?}: ", key);
    }
    // println!("{:?}", btreemap.;
}
fn main() {
    println!("Hello, world!");
    block::poa_with_sep_th();
    test_btreemap();
}
