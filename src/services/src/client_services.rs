use actix_web::{web, HttpResponse, Responder};
use db_service::db_layer::snapshot_db;
use db_service::db_snapshot_ref::SchemaSnap;
use exonum_crypto::Hash;
use futures::channel::mpsc::*;
use message_handler::message_sender::MessageSender;
use message_handler::messages::MessageTypes;
use schema::signed_transaction::SignedTransaction;
use schema::transaction_pool::{TxnPool, TxnPoolKeyType, POOL};
use utils::serializer::{deserialize, serialize};

pub struct ClientServices {}

impl ClientServices {
    pub fn submit_transaction_service(
        transaction: web::Bytes,
        sender: &mut Sender<Option<MessageTypes>>,
    ) -> impl Responder {
        if let Ok(txn) = deserialize::<SignedTransaction>(&transaction) {
            debug!("submit_transaction {:?}", txn);
            if let Some(string) = txn.header.get(&String::from("timestamp")) {
                if let Ok(timestamp) = string.parse::<TxnPoolKeyType>() {
                    POOL.insert_op(&timestamp, &txn);
                    MessageSender::send_transaction_msg(sender, txn);
                    return HttpResponse::Ok().body("txn added in the pool");
                }
            }
        }
        HttpResponse::BadRequest().body("txn couldn't deserialize")
    }

    pub fn fetch_pending_transaction_service(transaction_hash: web::Bytes) -> impl Responder {
        if let Ok(txn_hash) = deserialize::<Hash>(&transaction_hash) {
            debug!("fetch_pending_transaction {:?}", txn_hash);
            if let Some(transaction) = POOL.get(&txn_hash) {
                if let Ok(serialized_transaction) = serialize(&transaction) {
                    return HttpResponse::Ok().body(serialized_transaction);
                };
            }
            return HttpResponse::BadRequest().body("BadRequest");
        }
        HttpResponse::BadRequest().body("txn_hash couldn't deserialize")
    }

    pub fn fetch_confirm_transaction_service(transaction_hash: web::Bytes) -> impl Responder {
        if let Ok(txn_hash) = deserialize::<Hash>(&transaction_hash) {
            debug!("fetch_confirm_transaction {:?}", txn_hash);
            let snapshot = snapshot_db();
            let schema = SchemaSnap::new(&snapshot);
            if let Some(transaction) = schema.get_transaction(txn_hash) {
                if let Ok(serialized_transaction) = serialize(&transaction) {
                    return HttpResponse::Ok().body(serialized_transaction);
                };
            }
            return HttpResponse::BadRequest().body("BadRequest");
        }
        HttpResponse::BadRequest().body("txn_hash couldn't deserialize")
    }

    pub fn fetch_state_service(address: web::Bytes) -> impl Responder {
        if let Ok(public_address) = deserialize::<String>(&address) {
            debug!("fetch_state {:?}", public_address);
            let snapshot = snapshot_db();
            let schema = SchemaSnap::new(&snapshot);
            if let Some(state) = schema.get_state(public_address) {
                if let Ok(serialized_state) = serialize(&state) {
                    return HttpResponse::Ok().body(serialized_state);
                }
            }
            return HttpResponse::BadRequest().body("BadRequest");
        }
        HttpResponse::BadRequest().body("string couldn't deserialize")
    }

    pub fn fetch_block_peer_service(address: web::Bytes) -> impl Responder {
        if let Ok(block_index) = deserialize::<u64>(&address) {
            debug!("fetch_block {:?}", block_index);
            let snapshot = snapshot_db();
            let schema = SchemaSnap::new(&snapshot);
            if let Some(block) = schema.get_block(block_index) {
                if let Ok(serialized_block) = serialize(&block) {
                    return HttpResponse::Ok().body(serialized_block);
                }
            }
            return HttpResponse::BadRequest().body("BadRequest");
        }
        HttpResponse::BadRequest().body("block index couldn't deserialize")
    }

    pub fn fetch_latest_block_peer_service() -> impl Responder {
        let snapshot = snapshot_db();
        let schema = SchemaSnap::new(&snapshot);
        if let Some(block) = schema.get_root_block() {
            if let Ok(serialized_block) = serialize(&block) {
                return HttpResponse::Ok().body(serialized_block);
            }
        }
        return HttpResponse::BadRequest().body("BadRequest");
    }

    pub fn fetch_block_service(address: web::Bytes) -> impl Responder {
        if let Ok(block_index) = deserialize::<u64>(&address) {
            debug!("fetch_block {:?}", block_index);
            let snapshot = snapshot_db();
            let schema = SchemaSnap::new(&snapshot);
            if let Some(block) = schema.get_block(block_index) {
                let block_string: String = block.to_string_format();
                if let Ok(serialized_block) = serialize(&block_string) {
                    return HttpResponse::Ok().body(serialized_block);
                }
            }
            return HttpResponse::BadRequest().body("BadRequest");
        }
        HttpResponse::BadRequest().body("block index couldn't deserialize")
    }

    pub fn fetch_latest_block_service() -> impl Responder {
        let snapshot = snapshot_db();
        let schema = SchemaSnap::new(&snapshot);
        if let Some(block) = schema.get_root_block() {
            let block_string: String = block.to_string_format();
            if let Ok(serialized_block) = serialize(&block_string) {
                return HttpResponse::Ok().body(serialized_block);
            }
        }
        return HttpResponse::BadRequest().body("BadRequest");
    }

    pub fn fetch_blockchain_length_peer_service() -> impl Responder {
        let snapshot = snapshot_db();
        let schema = SchemaSnap::new(&snapshot);
        if let Ok(serialized_length) = serialize(&schema.get_blockchain_length()) {
            return HttpResponse::Ok().body(serialized_length);
        }
        HttpResponse::BadRequest().body("BadRequest")
    }

    pub fn fetch_blockchain_length_service() -> impl Responder {
        let snapshot = snapshot_db();
        let schema = SchemaSnap::new(&snapshot);
        if let Ok(serialized_length) = serialize(&schema.get_blockchain_length()) {
            return HttpResponse::Ok().body(serialized_length);
        }
        HttpResponse::BadRequest().body("BadRequest")
    }
}
