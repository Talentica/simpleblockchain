#[cfg(test)]
mod test_controller_services {
    use crate::client_services::ClientServices;
    use actix_web::{
        dev::{Body, ResponseBody},
        web, HttpResponse,
    };
    use db_service::{
        db_fork_ref::SchemaFork,
        db_layer::{fork_db, patch_db},
    };
    use exonum_crypto::Hash;
    use exonum_merkledb::ObjectHash;
    use futures::channel::mpsc::*;
    use message_handler::messages::MessageTypes;
    use schema::block::SignedBlock;
    use schema::signed_transaction::SignedTransaction;
    use schema::state::State;
    use schema::transaction_pool::TxnPoolKeyType;
    use sdk::traits::StateContext;
    use std::collections::HashMap;
    use std::time::SystemTime;
    use std::{thread, time::Duration};
    use utils::configreader::{FILE_PATH, GLOBAL_CONFIG};
    use utils::crypto::keypair::{CryptoKeypair, Keypair, KeypairType};
    use utils::serializer::{deserialize, serialize};

    fn test_submit_transaction_service() {
        let (mut sender, mut receiver) = channel::<Option<MessageTypes>>(4194304);
        let mut header = HashMap::default();
        let time_stamp: TxnPoolKeyType = 6565656565;
        header.insert("timestamp".to_string(), time_stamp.to_string());
        let signed_transaction: SignedTransaction = SignedTransaction {
            txn: vec![0],
            app_name: String::from("app_name"),
            header,
            signature: vec![0],
        };
        let transaction: web::Bytes = web::Bytes::from(serialize(&signed_transaction).unwrap());
        let mut http_response: HttpResponse =
            ClientServices::submit_transaction_service(transaction, &mut sender);
        if http_response.status() == 200 {
            let response_body: ResponseBody<Body> = http_response.take_body();
            let body_ref = response_body.as_ref().unwrap();
            let body_vec: Vec<u8> = match body_ref {
                Body::None => panic!("invalid response body type"),
                Body::Empty => panic!("invalid response body type"),
                Body::Bytes(ref b) => b.to_vec(),
                Body::Message(_) => panic!("invalid response body type"),
            };
            let output: String = deserialize(&body_vec).unwrap();
            assert_eq!(output, String::from("txn added in the pool"));
        } else {
            panic!("http_response not equal to 200");
        }
        thread::sleep(Duration::from_millis(100));
        let received_data = receiver.try_next().unwrap();
        let data = received_data.unwrap();
        if let None = data {
            panic!("transaction couldn't able to submit");
        }
    }

    fn test_fetch_pending_transaction_service() {
        let time_stamp: TxnPoolKeyType = 6565656565;
        let mut header = HashMap::default();
        header.insert("timestamp".to_string(), time_stamp.to_string());
        let signed_transaction: SignedTransaction = SignedTransaction {
            txn: vec![0],
            app_name: String::from("app_name"),
            header,
            signature: vec![0],
        };
        let txn_hash: Hash = signed_transaction.object_hash();
        let transaction_hash: web::Bytes = web::Bytes::from(serialize(&txn_hash).unwrap());
        let mut http_response: HttpResponse =
            ClientServices::fetch_pending_transaction_service(transaction_hash);
        if http_response.status() == 200 {
            let response_body: ResponseBody<Body> = http_response.take_body();
            let body_ref = response_body.as_ref().unwrap();
            let body_vec: Vec<u8> = match body_ref {
                Body::None => panic!("invalid response body type"),
                Body::Empty => panic!("invalid response body type"),
                Body::Bytes(ref b) => b.to_vec(),
                Body::Message(_) => panic!("invalid response body type"),
            };
            let output: SignedTransaction = deserialize(&body_vec).unwrap();
            assert_eq!(txn_hash, output.object_hash());
        } else {
            panic!("http_response not equal to 200");
        }
    }

    fn test_fetch_confirm_transaction_service() {
        let mut header = HashMap::default();
        let time_stamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_micros();
        header.insert("timestamp".to_string(), time_stamp.to_string());
        let signed_transaction: SignedTransaction = SignedTransaction {
            txn: vec![0],
            app_name: String::from("app_name"),
            header,
            signature: vec![0],
        };
        let txn_hash: Hash = signed_transaction.object_hash();
        let fork = fork_db();
        {
            let mut schema = SchemaFork::new(&fork);
            schema.put_txn(&txn_hash, signed_transaction.clone());
        }
        patch_db(fork);
        let transaction_hash: web::Bytes = web::Bytes::from(serialize(&txn_hash).unwrap());
        let mut http_response: HttpResponse =
            ClientServices::fetch_confirm_transaction_service(transaction_hash);
        if http_response.status() == 200 {
            let response_body: ResponseBody<Body> = http_response.take_body();
            let body_ref = response_body.as_ref().unwrap();
            let body_vec: Vec<u8> = match body_ref {
                Body::None => panic!("invalid response body type"),
                Body::Empty => panic!("invalid response body type"),
                Body::Bytes(ref b) => b.to_vec(),
                Body::Message(_) => panic!("invalid response body type"),
            };
            let output: SignedTransaction = deserialize(&body_vec).unwrap();
            assert_eq!(txn_hash, output.object_hash())
        } else {
            panic!("http_response not equal to 200");
        }
    }

    fn test_fetch_state_service() {
        let state_key: String = String::from("dkcnjsdcnosdvnvsfv");
        let state: State = State::new();
        let fork = fork_db();
        {
            let mut schema = SchemaFork::new(&fork);
            schema.put(&state_key, state.clone());
        }
        patch_db(fork);
        let state_key_bytes: web::Bytes = web::Bytes::from(serialize(&state_key).unwrap());
        let mut http_response: HttpResponse = ClientServices::fetch_state_service(state_key_bytes);
        if http_response.status() == 200 {
            let response_body: ResponseBody<Body> = http_response.take_body();
            let body_ref = response_body.as_ref().unwrap();
            let body_vec: Vec<u8> = match body_ref {
                Body::None => panic!("invalid response body type"),
                Body::Empty => panic!("invalid response body type"),
                Body::Bytes(ref b) => b.to_vec(),
                Body::Message(_) => panic!("invalid response body type"),
            };
            let output: State = deserialize(&body_vec).unwrap();
            assert_eq!(state.get_data(), output.get_data());
        } else {
            panic!("http_response not equal to 200");
        }
    }

    fn fetch_block_peer_service() {
        let kp: KeypairType = Keypair::generate();
        let fork = fork_db();
        {
            let mut schema = SchemaFork::new(&fork);
            schema.initialize_db(&kp);
        }
        patch_db(fork);
        let index_byes: web::Bytes = web::Bytes::from(serialize(&0).unwrap());
        let mut http_response: HttpResponse = ClientServices::fetch_block_peer_service(index_byes);
        if http_response.status() == 200 {
            let response_body: ResponseBody<Body> = http_response.take_body();
            let body_ref = response_body.as_ref().unwrap();
            let body_vec: Vec<u8> = match body_ref {
                Body::None => panic!("invalid response body type"),
                Body::Empty => panic!("invalid response body type"),
                Body::Bytes(ref b) => b.to_vec(),
                Body::Message(_) => panic!("invalid response body type"),
            };
            let output: SignedBlock = deserialize(&body_vec).unwrap();
            assert_eq!(output.validate(&output.block.peer_id), true);
            assert_eq!(0, output.block.id);
        } else {
            panic!("http_response not equal to 200");
        }
    }

    fn fetch_latest_block_peer_service() {
        let mut http_response: HttpResponse = ClientServices::fetch_latest_block_peer_service();
        if http_response.status() == 200 {
            let response_body: ResponseBody<Body> = http_response.take_body();
            let body_ref = response_body.as_ref().unwrap();
            let body_vec: Vec<u8> = match body_ref {
                Body::None => panic!("invalid response body type"),
                Body::Empty => panic!("invalid response body type"),
                Body::Bytes(ref b) => b.to_vec(),
                Body::Message(_) => panic!("invalid response body type"),
            };
            let output: SignedBlock = deserialize(&body_vec).unwrap();
            assert_eq!(output.validate(&output.block.peer_id), true);
        } else {
            panic!("http_response not equal to 200");
        }
    }

    fn test_fetch_block_service() {
        let index_byes: web::Bytes = web::Bytes::from(serialize(&0).unwrap());
        let mut http_response: HttpResponse = ClientServices::fetch_block_service(index_byes);
        if http_response.status() == 200 {
            let response_body: ResponseBody<Body> = http_response.take_body();
            let body_ref = response_body.as_ref().unwrap();
            let body_vec: Vec<u8> = match body_ref {
                Body::None => panic!("invalid response body type"),
                Body::Empty => panic!("invalid response body type"),
                Body::Bytes(ref b) => b.to_vec(),
                Body::Message(_) => panic!("invalid response body type"),
            };
            deserialize::<String>(&body_vec).unwrap();
        } else {
            panic!("http_response not equal to 200");
        }
    }

    fn test_fetch_latest_block_servic() {
        let mut http_response: HttpResponse = ClientServices::fetch_latest_block_service();
        if http_response.status() == 200 {
            let response_body: ResponseBody<Body> = http_response.take_body();
            let body_ref = response_body.as_ref().unwrap();
            let body_vec: Vec<u8> = match body_ref {
                Body::None => panic!("invalid response body type"),
                Body::Empty => panic!("invalid response body type"),
                Body::Bytes(ref b) => b.to_vec(),
                Body::Message(_) => panic!("invalid response body type"),
            };
            deserialize::<String>(&body_vec).unwrap();
        } else {
            panic!("http_response not equal to 200");
        }
    }

    fn test_fetch_blockchain_length_peer_service() {
        let mut http_response: HttpResponse =
            ClientServices::fetch_blockchain_length_peer_service();
        if http_response.status() == 200 {
            let response_body: ResponseBody<Body> = http_response.take_body();
            let body_ref = response_body.as_ref().unwrap();
            let body_vec: Vec<u8> = match body_ref {
                Body::None => panic!("invalid response body type"),
                Body::Empty => panic!("invalid response body type"),
                Body::Bytes(ref b) => b.to_vec(),
                Body::Message(_) => panic!("invalid response body type"),
            };
            deserialize::<u64>(&body_vec).unwrap();
        } else {
            panic!("http_response not equal to 200");
        }
    }

    fn test_fetch_blockchain_length_service() {
        let mut http_response: HttpResponse = ClientServices::fetch_blockchain_length_service();
        if http_response.status() == 200 {
            let response_body: ResponseBody<Body> = http_response.take_body();
            let body_ref = response_body.as_ref().unwrap();
            let body_vec: Vec<u8> = match body_ref {
                Body::None => panic!("invalid response body type"),
                Body::Empty => panic!("invalid response body type"),
                Body::Bytes(ref b) => b.to_vec(),
                Body::Message(_) => panic!("invalid response body type"),
            };
            deserialize::<u64>(&body_vec).unwrap();
        } else {
            panic!("http_response not equal to 200");
        }
    }

    #[test]
    fn test_controller_services() {
        &FILE_PATH.set_file_path(&String::from("../../config.toml"));
        lazy_static::initialize(&GLOBAL_CONFIG);
        test_submit_transaction_service();
        test_fetch_pending_transaction_service();
        test_fetch_confirm_transaction_service();
        test_fetch_state_service();
        fetch_block_peer_service();
        fetch_latest_block_peer_service();
        test_fetch_block_service();
        test_fetch_latest_block_servic();
        test_fetch_blockchain_length_peer_service();
        test_fetch_blockchain_length_service();
    }
}
