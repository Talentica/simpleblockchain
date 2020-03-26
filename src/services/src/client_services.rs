use actix_web::{web, HttpResponse, Responder};
use db_service::db_layer::snapshot_db;
use db_service::db_snapshot_ref::SchemaSnap;
use exonum_crypto::Hash;
use futures::channel::mpsc::*;
use p2plib::message_sender::MessageSender;
use p2plib::messages::MessageTypes;
use schema::transaction::SignedTransaction;
use schema::transaction_pool::{TxnPool, TxnPoolKeyType, POOL};
use utils::serializer::{deserialize, serialize};

pub struct ClientServices {}

impl ClientServices {
    pub fn submit_transaction_service(
        transaction: web::Bytes,
        sender: &mut Sender<Option<MessageTypes>>,
    ) -> impl Responder {
        let txn: SignedTransaction = deserialize(&transaction);
        // println!("submit_transaction {:?}", transaction);
        let timestamp = txn
            .header
            .get(&String::from("timestamp"))
            .unwrap()
            .parse::<TxnPoolKeyType>()
            .unwrap();
        POOL.insert_op(&timestamp, &txn);
        MessageSender::send_transaction_msg(sender, txn);
        HttpResponse::Ok().body("txn added in the pool")
    }

    pub fn fetch_pending_transaction_service(transaction_hash: web::Bytes) -> impl Responder {
        let txn_hash: Hash = deserialize(&transaction_hash);
        println!("fetch_pending_transaction {:?}", transaction_hash);
        match POOL.get(&txn_hash) {
            Some(transaction) => HttpResponse::Ok().body(serialize(&transaction)),
            None => HttpResponse::BadRequest().body("BadRequest"),
        }
    }

    pub fn fetch_confirm_transaction_service(transaction_hash: web::Bytes) -> impl Responder {
        let txn_hash: Hash = deserialize(&transaction_hash);
        println!("fetch_confirm_transaction {:?}", transaction_hash);
        let snapshot = snapshot_db();
        let schema = SchemaSnap::new(&snapshot);
        match schema.get_transaction(txn_hash) {
            Some(txn) => HttpResponse::Ok().body(serialize(&txn)),
            None => HttpResponse::BadRequest().body("BadRequest"),
        }
    }

    pub fn fetch_state_service(address: web::Bytes) -> impl Responder {
        let public_address: String = deserialize(&address);
        println!("fetch_state {:?}", public_address);
        let snapshot = snapshot_db();
        let schema = SchemaSnap::new(&snapshot);
        match schema.get_state(public_address) {
            Some(state) => HttpResponse::Ok().body(serialize(&state)),
            None => HttpResponse::BadRequest().body("BadRequest"),
        }
    }

    pub fn fetch_block_peer_service(address: web::Bytes) -> impl Responder {
        let block_index: u64 = deserialize(&address);
        println!("fetch_block {:?}", block_index);
        let snapshot = snapshot_db();
        let schema = SchemaSnap::new(&snapshot);
        match schema.get_block(block_index) {
            Some(block) => HttpResponse::Ok().body(serialize(&block)),
            None => HttpResponse::BadRequest().body("BadRequest"),
        }
    }

    pub fn fetch_latest_block_peer_service() -> impl Responder {
        let snapshot = snapshot_db();
        let schema = SchemaSnap::new(&snapshot);
        match schema.get_root_block() {
            Some(block) => HttpResponse::Ok().body(serialize(&block)),
            None => HttpResponse::BadRequest().body("BadRequest"),
        }
    }

    pub fn fetch_block_service(address: web::Bytes) -> impl Responder {
        let block_index: u64 = deserialize(&address);
        println!("fetch_block {:?}", block_index);
        let snapshot = snapshot_db();
        let schema = SchemaSnap::new(&snapshot);
        match schema.get_block(block_index) {
            Some(block) => {
                let block_string: String = block.to_string_format();
                HttpResponse::Ok().body(serialize(&block_string))
            }
            None => HttpResponse::BadRequest().body("BadRequest"),
        }
    }

    pub fn fetch_latest_block_service() -> impl Responder {
        let snapshot = snapshot_db();
        let schema = SchemaSnap::new(&snapshot);
        match schema.get_root_block() {
            Some(block) => {
                let block_string: String = block.to_string_format();
                HttpResponse::Ok().body(serialize(&block_string))
            }
            None => HttpResponse::BadRequest().body("BadRequest"),
        }
    }
}
