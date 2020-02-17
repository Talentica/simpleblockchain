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
    DB_INSTANCE.merge(fork.into_patch()).unwrap();
}

#[cfg(test)]
mod tests_db_layer {
    use super::*;
    use exonum_crypto::Hash;
    use exonum_merkledb::{
        impl_object_hash_for_binary_value, BinaryValue, ObjectHash, ProofMapIndex,
    };
    use failure::Error;
    use std::{borrow::Cow, convert::AsRef};
    use utils::serializer::{Deserialize, Serialize};

    #[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
    pub struct Txn {
        nonce: u64,
        from: String,
    }

    impl Txn {
        fn new() -> Txn {
            Txn {
                nonce: 231,
                from: String::from("ddd"),
            }
        }
    }

    impl_object_hash_for_binary_value! {Txn}

    impl BinaryValue for Txn {
        fn to_bytes(&self) -> Vec<u8> {
            bincode::serialize(self).unwrap()
        }

        fn from_bytes(bytes: Cow<'_, [u8]>) -> Result<Self, Error> {
            bincode::deserialize(bytes.as_ref()).map_err(From::from)
        }
    }

    #[test]
    pub fn test_db_operations() {
        let fork = fork_db();
        let name = "name";
        {
            let mut mut_index: ProofMapIndex<_, Hash, Txn> = ProofMapIndex::new(name, &fork);
            let value = Txn::new();
            let key = value.object_hash();
            mut_index.put(&key, value);
            println!("added in database {}", key);
            // mut_index.clear();
        }
        patch_db(fork);
        let snapshot = snapshot_db();
        {
            let mut_index: ProofMapIndex<_, Hash, Txn> = ProofMapIndex::new(name, &snapshot);
            println!(" data from snapshot");
            for (_key, _value) in mut_index.iter() {
                println!("{} {:?} ", _key, _value);
            }
        }
    }
}
