use libloading::Library;
use sdk::traits::AppHandler;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub struct AppData {
    pub appdata: HashMap<String, Arc<Mutex<Box<dyn AppHandler + Send>>>>,
    pub lib: Vec<Arc<Library>>,
}

impl AppData {
    pub fn new() -> AppData {
        AppData {
            appdata: HashMap::new(),
            lib: Vec::new(),
        }
    }
}

lazy_static! {
    pub static ref APPDATA: Arc<Mutex<AppData>> = Arc::new(Mutex::new(AppData::new()));
}

#[cfg(test)]
mod tests_app_data {

    use super::*;
    use sdk::signed_transaction::SignedTransaction;
    use sdk::traits::StateContext;
    const APPNAME: &str = "MockApp";

    pub struct MockApp {
        name: String,
    }

    impl MockApp {
        pub fn new(s: &String) -> MockApp {
            MockApp { name: s.clone() }
        }
    }

    impl AppHandler for MockApp {
        fn execute(&self, _txn: &SignedTransaction, _state_context: &mut dyn StateContext) -> bool {
            true
        }

        fn name(&self) -> String {
            self.name.clone()
        }
    }

    pub fn register_app() -> Box<dyn AppHandler + Send> {
        Box::new(MockApp::new(&String::from(APPNAME)))
    }

    #[test]
    pub fn test_app_data_insertion() {
        {
            let app_handle = Arc::new(Mutex::new(register_app()));
            let app_name = app_handle.lock().unwrap().name();
            println!("Loaded app {:?}", app_name);
            let mut locked_app_data = APPDATA.lock().unwrap();
            // locked_app_data.lib.push(Arc::new(applib));
            locked_app_data
                .appdata
                .insert(app_name.clone(), app_handle.clone());
        }
        assert_eq!(
            APPDATA.lock().unwrap().appdata.len(),
            1,
            "Issue with appdata insert"
        );
    }
}
