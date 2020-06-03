use exonum_merkledb::{Database, DbOptions, Fork, RocksDB, Snapshot};
use utils::configreader;
use utils::configreader::Configuration;

fn create_db_instance() -> RocksDB {
    let db_options: DbOptions = Default::default();
    let config: &Configuration = &configreader::GLOBAL_CONFIG;
    match RocksDB::open(&config.db.dbpath, &db_options) {
        Ok(connection) => return connection,
        Err(_) => panic!("can't able to create new db instance"),
    };
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
        println!("error occurred in patch_db process {:?}", error);
    }
}

#[cfg(test)]
mod test_db_layer {
    use super::*;
    use utils::configreader::initialize_config;

    #[test]
    #[should_panic]
    fn test_create_db_instance() {
        initialize_config("../../config.toml");
        fork_db();
        create_db_instance();
    }
}
