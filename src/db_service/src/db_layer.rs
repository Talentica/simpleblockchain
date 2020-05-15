use anyhow::{self, ensure, format_err};
use exonum_merkledb::{Database, DbOptions, Fork, RocksDB, Snapshot};

fn create_db_instance() -> RocksDB {
    let db_options: DbOptions = Default::default();
    RocksDB::open("dbtest/rocksdb", &db_options).unwrap()
}

lazy_static! {
    pub static ref DB_INSTANCE: RocksDB = create_db_instance();
}

pub fn fork_db() -> Fork {
    DB_INSTANCE.fork()
    // db.fork()
}

pub fn snapshot_db() -> Box<dyn Snapshot> {
    DB_INSTANCE.snapshot()
}

pub fn patch_db(fork: Fork) {
    if let Err(error) = DB_INSTANCE.merge(fork.into_patch()) {
        error!("error occurred in patch_db process {:?}", error);
    }
}

#[cfg(test)]
mod tests_db_layer {
    use super::*;
    use exonum_merkledb::{access::CopyAccessExt, ProofMapIndex};

    #[test]
    pub fn test_db_operations() {
        let fork = fork_db();
        let name = "name";
        {
            let mut mut_index: ProofMapIndex<_, String, String> = fork.get_proof_map(name);
            let value: String = String::from("value_string");
            let key: String = String::from("key_string");
            mut_index.put(&key, value.clone());
            info!("added in database {}", key);
            // mut_index.clear();
        }
        patch_db(fork);
        let snapshot = snapshot_db();
        {
            let mut_index: ProofMapIndex<_, String, String> = snapshot.get_proof_map(name);
            info!(" data from snapshot");
            for (_key, _value) in mut_index.iter() {
                info!("{} {:?} ", _key, _value);
            }
        }
    }
}
