#[cfg(test)]
mod test_db_service {
    use crate::db_fork_ref::*;
    use crate::db_layer::{fork_db, patch_db, snapshot_db};
    use crate::db_snapshot_ref::*;
    use exonum_crypto::Hash;
    use exonum_merkledb::{Fork, ObjectHash, Snapshot};
    use schema::block::{Block, BlockTraits, SignedBlock};
    use schema::signed_transaction::SignedTransaction;
    use schema::state::State;
    use sdk::traits::StateContext;
    use std::collections::HashMap;
    use std::time::SystemTime;
    use utils::configreader;
    use utils::configreader::BlockConfig;
    use utils::keypair::{CryptoKeypair, Keypair, KeypairType};

    fn test_db_initialization_check() {
        // reset_db_state
        let kp: KeypairType = Keypair::generate();
        let pk: String = hex::encode(kp.public().encode());
        let block: Block = Block::genesis_block(Vec::new(), 0);
        #[allow(unused_assignments)]
        let mut signed_block: SignedBlock = SignedBlock::create_block(block, vec![0], Vec::new());
        let fork: Fork = fork_db();
        {
            let mut schema = SchemaFork::new(&fork);
            signed_block = schema.initialize_db(Vec::new(), 0);
            assert_eq!(
                signed_block.block.validate(&pk, &signed_block.signature),
                false
            );
        }
        patch_db(fork);
        // not using patch_db so that we can update this block
        let fork: Fork = fork_db();
        {
            let mut schema = SchemaFork::new(&fork);
            assert_eq!(false, schema.update_block(&signed_block));
        }

        // check db state to check whether fork and patch are working using snapshot
        let empty_hash: Hash = signed_block.block.header[0].clone();
        let snapshot: Box<dyn Snapshot> = snapshot_db();
        {
            let schema = SchemaSnap::new(&snapshot);
            assert_eq!(schema.get_blockchain_length(), 1);
            assert_eq!(schema.get_state_trie_hash(), empty_hash);
            assert_eq!(schema.get_storage_trie_hash(), empty_hash);
            assert_eq!(schema.get_transaction_trie_hash(), empty_hash);
        }
    }

    fn test_db_read_write_check() {
        // reset_db_state
        let kp: KeypairType = Keypair::generate();
        let snapshot: Box<dyn Snapshot> = snapshot_db();
        {
            let schema = SchemaSnap::new(&snapshot);
            if !schema.is_db_initialized() {}
        }
        // db is initialized create one block and verify it with snapshot
        let fork: Fork = fork_db();
        let block: Block = Block::genesis_block(Vec::new(), 0);
        #[allow(unused_assignments)]
        let mut signed_block: SignedBlock = SignedBlock::create_block(block, vec![0], Vec::new());
        {
            let mut schema = SchemaFork::new(&fork);
            signed_block = schema.create_block(&kp, Vec::new());
        }
        // not using patch_db so that we can update this block
        let fork: Fork = fork_db();
        #[allow(unused_assignments)]
        let mut update_flag: bool = false;
        {
            let mut schema = SchemaFork::new(&fork);
            update_flag = schema.update_block(&signed_block);
        }
        assert_eq!(update_flag, true);
        if update_flag {
            patch_db(fork);
        }
        // check db state to check whether fork and patch are working using snapshot
        let snapshot: Box<dyn Snapshot> = snapshot_db();
        {
            let schema = SchemaSnap::new(&snapshot);
            assert_eq!(schema.get_root_block_hash(), signed_block.get_hash());
        }
    }

    fn test_db_state_context() {
        let kp: KeypairType = Keypair::generate();
        let pk: String = hex::encode(kp.public().encode());
        let fork: Fork = fork_db();
        let state: State = State::new();
        let txn: SignedTransaction = SignedTransaction {
            txn: vec![0],
            app_name: String::from("app_name"),
            header: HashMap::new(),
            signature: vec![0],
        };
        let txn_hash: Hash = txn.object_hash();
        {
            let mut schema = SchemaFork::new(&fork);
            // let state_context = schema as &mut dyn StateContext;
            schema.put(&pk, state.clone());
            schema.put_txn(&txn_hash, txn.clone());
        }
        patch_db(fork);
        let fork: Fork = fork_db();
        {
            let schema = SchemaFork::new(&fork);
            let is_contains: bool = schema.contains(&pk);
            println!("{:?}", is_contains);
            assert_eq!(is_contains, true);
            assert_eq!(schema.get(&pk).unwrap(), state);

            let is_contains: bool = schema.contains_txn(&txn_hash);
            println!("{:?}", is_contains);
            assert_eq!(is_contains, true);
            assert_eq!(schema.get_txn(&txn_hash).unwrap(), txn);
        }
    }

    fn test_db_sync_state() {
        // since it is unit test case sync-state should return zero-state not error
        let fork: Fork = fork_db();
        {
            let mut schema = SchemaFork::new(&fork);
            assert_eq!(schema.sync_state(), false);
        }
    }

    fn test_failed_scenarios() {
        let kp: KeypairType = Keypair::generate();
        // db is initialized create one block and verify it with snapshot
        let fork: Fork = fork_db();
        let block: Block = Block::genesis_block(Vec::new(), 0);
        #[allow(unused_assignments)]
        let mut signed_block: SignedBlock = SignedBlock::create_block(block, vec![0], Vec::new());
        {
            let mut schema = SchemaFork::new(&fork);
            signed_block = schema.create_block(&kp, Vec::new());
        }
        // signature error
        let mut wrong_block: SignedBlock = signed_block.clone();
        let fork: Fork = fork_db();
        {
            let mut schema = SchemaFork::new(&fork);
            wrong_block.block.header[2] = Hash::zero();
            assert_eq!(false, schema.update_block(&wrong_block));
        }
        {
            let mut schema = SchemaFork::new(&fork);
            wrong_block.block.header[1] = Hash::zero();
            assert_eq!(false, schema.update_block(&wrong_block));
        }
        {
            let mut schema = SchemaFork::new(&fork);
            wrong_block.block.header[0] = Hash::zero();
            assert_eq!(false, schema.update_block(&wrong_block));
        }
        {
            let mut schema = SchemaFork::new(&fork);
            wrong_block.block.prev_hash = Hash::zero();
            assert_eq!(false, schema.update_block(&wrong_block));
        }
        {
            let mut schema = SchemaFork::new(&fork);
            wrong_block.block.id = schema.blockchain_length() - 1;
            assert_eq!(false, schema.update_block(&wrong_block));
        }
        {
            let mut schema = SchemaFork::new(&fork);
            wrong_block.signature = vec![0];
            assert_eq!(false, schema.update_block(&wrong_block));
        }
        {
            let mut schema = SchemaFork::new(&fork);
            assert_eq!(true, schema.update_block(&signed_block));
        }
        patch_db(fork);
        // check db state to check whether fork and patch are working using snapshot
        let snapshot: Box<dyn Snapshot> = snapshot_db();
        {
            let schema = SchemaSnap::new(&snapshot);
            assert_eq!(
                schema.get_block_hash(signed_block.block.id),
                signed_block.get_hash()
            );
        }
    }

    fn test_block_creation_config() {
        let kp: KeypairType = Keypair::generate();
        let block_config: &BlockConfig = &configreader::GLOBAL_CONFIG.block_config;
        let fork = fork_db();
        {
            let schema = SchemaFork::new(&fork);
            let mut timestamp: u128 = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_micros();
            timestamp = timestamp + block_config.block_creation_time_limit;
            let (_fork_instance, _signed_block) = schema.forge_new_block(&kp, Vec::new());
            let current_timestamp: u128 = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_micros();
            assert_eq!(
                current_timestamp > timestamp,
                true,
                "it should take extra time since there is no transaction to fullfill first cut-off"
            );
        }
        {
            let mut schema = SchemaFork::new(&fork);
            let mut timestamp: u128 = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_micros();
            timestamp = timestamp + block_config.block_creation_time_limit;
            let _signed_block = schema.create_block(&kp, Vec::new());
            let current_timestamp: u128 = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_micros();
            assert_eq!(
                current_timestamp > timestamp,
                false,
                "it should take no extra time since it does not consider block creation config"
            );
        }
    }

    #[test]
    fn test_db_services_checks() {
        std::thread::sleep(std::time::Duration::from_millis(100));
        test_db_initialization_check();
        test_db_read_write_check();
        test_db_state_context();
        test_db_sync_state();
        test_failed_scenarios();
        test_block_creation_config();
    }
}
