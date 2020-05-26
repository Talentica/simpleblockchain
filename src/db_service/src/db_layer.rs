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
