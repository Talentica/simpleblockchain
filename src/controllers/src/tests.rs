#[cfg(test)]
mod test_controller_services {
    use crate::client_controller::*;
    use client::client::ClientObj;
    use db_service::{
        db_fork_ref::SchemaFork,
        db_layer::{fork_db, patch_db},
    };
    use exonum_crypto::Hash;
    use exonum_merkledb::ObjectHash;
    use futures::channel::mpsc::*;
    use libp2p::core::{Multiaddr, PeerId};
    use message_handler::messages::MessageTypes;
    use schema::signed_transaction::SignedTransaction;
    use schema::state::State;
    use schema::transaction_pool::TxnPoolKeyType;
    use sdk::traits::StateContext;
    use std::collections::HashMap;
    use std::net::IpAddr;
    use std::net::Ipv4Addr;
    use std::time::SystemTime;
    use std::{thread, time::Duration};
    use utils::configreader::initialize_config;
    use utils::global_peer_data::*;
    use utils::serializer::serialize;

    fn test_submit_transaction_controller(client: &ClientObj) {
        let mut header = HashMap::default();
        let time_stamp: TxnPoolKeyType = 6565656565;
        header.insert("timestamp".to_string(), time_stamp.to_string());
        let signed_transaction: SignedTransaction = SignedTransaction {
            txn: vec![0],
            app_name: String::from("app_name"),
            header,
            signature: vec![0],
        };
        let transaction: Vec<u8> = serialize(&signed_transaction).unwrap();
        match client.submit_transaction(transaction) {
            Ok(value) => {
                if !value {
                    panic!("error in transaction submit process");
                }
            }
            Err(_) => panic!("error in transaction submit process"),
        };
    }

    fn test_fetch_pending_transaction_controller(client: &ClientObj) {
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
        match client.fetch_pending_transaction(&txn_hash) {
            Ok(is_value) => {
                if None == is_value {
                    panic!("error in fetch pending transaction process");
                }
            }
            Err(_) => panic!("http_response not equal to 200"),
        };
    }

    fn test_fetch_confirm_transaction_controller(client: &ClientObj) {
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
            if !schema.contains_txn(&txn_hash) {
                schema.put_txn(&txn_hash, signed_transaction.clone());
            }
        }
        patch_db(fork);
        match client.fetch_confirm_transaction(&txn_hash) {
            Ok(is_value) => {
                if None == is_value {
                    panic!("error in fetch confirm transaction process");
                }
            }
            Err(_) => panic!("http_response not equal to 200"),
        };
    }

    fn test_fetch_state_controller(client: &ClientObj) {
        let state_key: String = String::from("dkcnjsdcnosdvnvsfv");
        let state: State = State::new();
        let fork = fork_db();
        {
            let mut schema = SchemaFork::new(&fork);
            if !schema.contains(&state_key) {
                schema.put(&state_key, state.clone());
            }
        }
        patch_db(fork);
        match client.fetch_state(&state_key) {
            Ok(is_value) => {
                if None == is_value {
                    panic!("error in fetch state process");
                }
            }
            Err(_) => panic!("http_response not equal to 200"),
        };
    }

    fn test_fetch_block_controller(client: &ClientObj) {
        let fork = fork_db();
        {
            let mut schema = SchemaFork::new(&fork);
            schema.initialize_db(Vec::new());
        }
        patch_db(fork);
        match client.fetch_block(&0) {
            Ok(is_value) => {
                if None == is_value {
                    panic!("error in fetch block process");
                }
            }
            Err(_) => panic!("http_response not equal to 200"),
        };
    }

    fn test_fetch_latest_block_controller(client: &ClientObj) {
        match client.fetch_latest_block() {
            Ok(is_value) => {
                if None == is_value {
                    panic!("error in fetch latest block process");
                }
            }
            Err(_) => panic!("http_response not equal to 200"),
        };
    }

    fn test_fetch_blockchain_length_controller(client: &ClientObj) {
        match client.fetch_blockchain_length() {
            Ok(_) => {}
            Err(_) => panic!("http_response not equal to 200"),
        };
    }

    #[test]
    fn test_controller_functionality() {
        initialize_config("../../config.toml");
        let peer_id: String = "127.0.0.1".to_string();
        let time_stamp: u128 = 123445;
        let addr: Multiaddr = Multiaddr::from(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)));
        let id: PeerId = PeerId::random();
        let peer_data: PeerData = PeerData::new(id, time_stamp, addr);
        {
            GLOBALDATA.lock().unwrap().peers.insert(peer_id, peer_data);
        }
        let (sender, _) = channel::<Option<MessageTypes>>(4194304);
        let host: String = String::from("127.0.0.1");
        let mut client_controller: ClientController = ClientController::new(&host, 8089);
        thread::spawn(move || {
            client_controller.start_validator_controller(sender);
        });
        println!("thread for server started");
        thread::sleep(Duration::from_millis(2000));
        let client: ClientObj = ClientObj::new();
        test_submit_transaction_controller(&client);
        test_fetch_pending_transaction_controller(&client);
        test_fetch_confirm_transaction_controller(&client);
        test_fetch_state_controller(&client);
        test_fetch_block_controller(&client);
        test_fetch_latest_block_controller(&client);
        test_fetch_blockchain_length_controller(&client);
        std::process::exit(0);
    }
}
