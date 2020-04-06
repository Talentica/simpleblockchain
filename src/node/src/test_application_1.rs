extern crate db_service;
extern crate doc_app;
use db_service::db_fork_ref::SchemaFork;
use db_service::db_layer::{fork_db, patch_db};
use db_service::db_snapshot_ref::SchemaSnap;
use doc_app::{
    state::{NFTToken, State},
    transaction::{SignedTransaction, STATE_KEY},
    user_messages::DataTypes,
};
use exonum_crypto::Hash;
use exonum_merkledb::ObjectHash;
use schema::transaction_pool::{TxnPool, TxnPoolKeyType, POOL};
use utils::keypair::{CryptoKeypair, Keypair};

fn test_document_transfer_valid_scenario() {
    println!("\n ***Document Transfer Demo***");
    let mut secret =
        hex::decode("97ba6f71a5311c4986e01798d525d0da8ee5c54acbf6ef7c3fadd1e2f624442f")
            .expect("invalid secret");
    let keypair = Keypair::generate_from(secret.as_mut_slice());
    let _public_key =
        String::from("2c8a35450e1d198e3834d933a35962600c33d1d0f8f6481d6e08f140791374d0");
    let fork = fork_db();
    // put genesis blockin database
    {
        let mut schema = SchemaFork::new(&fork);
        schema.initialize_db(&keypair);
    }
    patch_db(fork);

    // add multiple docs in state
    let mut payload: Vec<DataTypes> = Vec::new();
    let mut token = NFTToken::default();
    let mut vector: Vec<Hash> = Vec::new();
    for index in 0..3 {
        let token_hash = token.object_hash();
        vector.push(token_hash);
        token.pkg_no = index.to_string();
    }
    payload.push(DataTypes::VecHashVal(vector.clone()));
    let fxn_call = String::from("add_doc");
    let txn: SignedTransaction =
        SignedTransaction::generate_from(&keypair, payload.clone(), fxn_call);
    let self_address: String = match &txn.txn {
        Some(txn) => txn.from.clone(),
        None => String::default(),
    };
    let timestamp = txn
        .header
        .get(&String::from("timestamp"))
        .unwrap()
        .parse::<TxnPoolKeyType>()
        .unwrap();
    POOL.insert_op(&timestamp, &txn);
    let fork = fork_db();
    {
        let mut schema = SchemaFork::new(&fork);
        let block = schema.create_block(&keypair);
        println!("{:?}", block);
    }
    patch_db(fork);

    // checking doc is present or not in the state
    println!("\n *** state after adding 3 unique documents ***");
    let snapshot = fork_db();
    {
        let schema = SchemaSnap::new(&snapshot);
        match schema.get_state(STATE_KEY.to_string()) {
            Some(state) => {
                let state: State = utils::serializer::deserialize(state.as_slice());
                for index in 0..vector.len() {
                    let each: Hash = vector.get(index).unwrap().clone();
                    println!("token details {:?}", state.get_nft_token(each));
                }
            }
            None => println!("no state found"),
        }
    }

    // transferring docs for review
    let to_address: String = String::from(self_address.clone());
    payload.clear();
    payload.push(DataTypes::VecHashVal(vector.clone()));
    payload.push(DataTypes::StringVal(to_address.clone()));
    let fxn_call = String::from("transfer_sc");
    let txn = SignedTransaction::generate_from(&keypair, payload.clone(), fxn_call);
    let timestamp = txn
        .header
        .get(&String::from("timestamp"))
        .unwrap()
        .parse::<TxnPoolKeyType>()
        .unwrap();
    POOL.insert_op(&timestamp, &txn);
    let fork = fork_db();
    {
        let mut schema = SchemaFork::new(&fork);
        let block = schema.create_block(&keypair);
        println!("{:?}", block);
    }
    patch_db(fork);

    // check review waiting list
    println!("\n *** state after transferring previous documents to SC ***");
    let snapshot = fork_db();
    {
        let schema = SchemaSnap::new(&snapshot);
        match schema.get_state(STATE_KEY.to_string()) {
            Some(state) => {
                let state: State = utils::serializer::deserialize(state.as_slice());
                println!(
                    "waiting list of SC {:?}",
                    state.get_confirmation_waiting_list(&to_address)
                );
                for index in 0..vector.len() {
                    let each: Hash = vector.get(index).unwrap().clone();
                    println!("token details {:?}", state.get_nft_token(each));
                }
            }
            None => println!("no state found"),
        }
    }

    // set pkg no and watch changes
    let pkg_no: String = String::from("pkg_no_420");
    payload.clear();
    payload.push(DataTypes::VecHashVal(vector.clone()));
    payload.push(DataTypes::StringVal(pkg_no.clone()));
    let fxn_call = String::from("set_pkg_no");
    let txn = SignedTransaction::generate_from(&keypair, payload.clone(), fxn_call);
    let timestamp = txn
        .header
        .get(&String::from("timestamp"))
        .unwrap()
        .parse::<TxnPoolKeyType>()
        .unwrap();
    POOL.insert_op(&timestamp, &txn);
    let fork = fork_db();
    {
        let mut schema = SchemaFork::new(&fork);
        let block = schema.create_block(&keypair);
        println!("{:?}", block);
    }
    patch_db(fork);

    // check review waiting list
    println!("\n *** state after binding pkg_no with documents***");
    let snapshot = fork_db();
    {
        let schema = SchemaSnap::new(&snapshot);
        match schema.get_state(STATE_KEY.to_string()) {
            Some(state) => {
                let state: State = utils::serializer::deserialize(state.as_slice());
                println!(
                    "waiting list of  SC {:?}",
                    state.get_confirmation_waiting_list(&to_address)
                );
                println!(
                    "pkg_no \'{:?}\' contains doc {:?}",
                    pkg_no,
                    state.get_pkg_list(&pkg_no)
                );
                for index in 0..vector.len() {
                    let each: Hash = vector.get(index).unwrap().clone();
                    println!("token details {:?}", state.get_nft_token(each));
                }
            }
            None => println!("no state found"),
        }
    }

    // transfer pkg_no for review
    payload.clear();
    payload.push(DataTypes::StringVal(pkg_no.clone()));
    payload.push(DataTypes::StringVal(self_address.clone()));
    let fxn_call = String::from("transfer_for_review");
    let txn = SignedTransaction::generate_from(&keypair, payload.clone(), fxn_call);
    let timestamp = txn
        .header
        .get(&String::from("timestamp"))
        .unwrap()
        .parse::<TxnPoolKeyType>()
        .unwrap();
    POOL.insert_op(&timestamp, &txn);
    let fork = fork_db();
    {
        let mut schema = SchemaFork::new(&fork);
        let block = schema.create_block(&keypair);
        println!("{:?}", block);
    }
    patch_db(fork);

    // check review waiting list
    println!("\n *** state after binding pkg_no with documents***");
    let snapshot = fork_db();
    {
        let schema = SchemaSnap::new(&snapshot);
        match schema.get_state(STATE_KEY.to_string()) {
            Some(state) => {
                let state: State = utils::serializer::deserialize(state.as_slice());
                println!(
                    "waiting list of  SC {:?}",
                    state.get_confirmation_waiting_list(&to_address)
                );
                println!(
                    "pkg_no \'{:?}\' contains doc {:?}",
                    pkg_no,
                    state.get_pkg_list(&pkg_no)
                );
                println!(
                    "pending review list of  \'{:?}\' -> {:?}",
                    self_address,
                    state.get_pkg_review_pending_list(&self_address)
                );
                for index in 0..vector.len() {
                    let each: Hash = vector.get(index).unwrap().clone();
                    println!("token details {:?}", state.get_nft_token(each));
                }
            }
            None => println!("no state found"),
        }
    }

    // review pkg_no accept or reject
    payload.clear();
    payload.push(DataTypes::StringVal(pkg_no.clone()));
    payload.push(DataTypes::BoolVal(false));
    let fxn_call = String::from("review_docs");
    let txn = SignedTransaction::generate_from(&keypair, payload.clone(), fxn_call);
    let timestamp = txn
        .header
        .get(&String::from("timestamp"))
        .unwrap()
        .parse::<TxnPoolKeyType>()
        .unwrap();
    POOL.insert_op(&timestamp, &txn);
    let fork = fork_db();
    {
        let mut schema = SchemaFork::new(&fork);
        let block = schema.create_block(&keypair);
        println!("{:?}", block);
    }
    patch_db(fork);

    // check state review process
    println!("\n *** state after review done by reviewer ***");
    let snapshot = fork_db();
    {
        let schema = SchemaSnap::new(&snapshot);
        match schema.get_state(STATE_KEY.to_string()) {
            Some(state) => {
                let state: State = utils::serializer::deserialize(state.as_slice());
                println!(
                    "waiting list of  SC {:?}",
                    state.get_confirmation_waiting_list(&to_address)
                );
                println!(
                    "pkg_no \'{:?}\' contains doc {:?}",
                    pkg_no,
                    state.get_pkg_list(&pkg_no)
                );
                println!(
                    "pending review list of  \'{:?}\' -> {:?}",
                    self_address,
                    state.get_pkg_review_pending_list(&self_address)
                );
                for index in 0..vector.len() {
                    let each: Hash = vector.get(index).unwrap().clone();
                    println!("token details {:?}", state.get_nft_token(each));
                }
            }
            None => println!("no state found"),
        }
    }

    // Publishing approved pokg_no
    payload.clear();
    payload.push(DataTypes::StringVal(pkg_no.clone()));
    let fxn_call = String::from("publish_docs");
    let txn = SignedTransaction::generate_from(&keypair, payload.clone(), fxn_call);
    let timestamp = txn
        .header
        .get(&String::from("timestamp"))
        .unwrap()
        .parse::<TxnPoolKeyType>()
        .unwrap();
    POOL.insert_op(&timestamp, &txn);
    let fork = fork_db();
    {
        let mut schema = SchemaFork::new(&fork);
        let block = schema.create_block(&keypair);
        println!("{:?}", block);
    }
    patch_db(fork);

    // check review waiting list
    println!("\n *** state after package publication done ***");
    let snapshot = fork_db();
    {
        let schema = SchemaSnap::new(&snapshot);
        match schema.get_state(STATE_KEY.to_string()) {
            Some(state) => {
                let state: State = utils::serializer::deserialize(state.as_slice());
                println!(
                    "waiting list of  SC {:?}",
                    state.get_confirmation_waiting_list(&to_address)
                );
                println!(
                    "pkg_no \'{:?}\' contains doc {:?}",
                    pkg_no,
                    state.get_pkg_list(&pkg_no)
                );
                println!(
                    "pending review list of  \'{:?}\' -> {:?}",
                    self_address,
                    state.get_pkg_review_pending_list(&self_address)
                );
                for index in 0..vector.len() {
                    let each: Hash = vector.get(index).unwrap().clone();
                    println!("token details {:?}", state.get_nft_token(each));
                }
            }
            None => println!("no state found"),
        }
    }
}

fn main() {
    test_document_transfer_valid_scenario();
}
