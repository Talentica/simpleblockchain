use super::signed_transaction::SignedTransaction;
use super::state::State;
use exonum_merkledb::access::{Access, RawAccessMut};
use generic_traits::traits::AppHandler;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
pub struct AppData {
    pub appdata: HashMap<String, Arc<Mutex<Box<dyn AppHandler + Send>>>>,
}

impl AppData {
    pub fn new() -> AppData {
        AppData {
            appdata: HashMap::new(),
        }
    }
}

lazy_static! {
    pub static ref APPDATA: Arc<Mutex<AppData>> = Arc::new(Mutex::new(AppData::new()));
}
