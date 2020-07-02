use super::*;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use toml;

#[derive(Debug)]
struct FilePath {
    path: Arc<std::sync::Mutex<String>>,
}

impl FilePath {
    fn new() -> FilePath {
        FilePath {
            path: Arc::new(Mutex::new(String::new())),
        }
    }

    fn get_file_path(&self) -> String {
        let locked_path = self.path.lock().unwrap();
        String::from(locked_path.clone())
    }

    fn set_file_path(&self, file_path: &String) {
        let mut locked_path = self.path.lock().unwrap();
        *locked_path = file_path.clone();
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Configuration {
    pub validator_set: Vec<String>,
    pub block_list_size: usize,
    pub step_time: u64,
    pub validator_ids: Vec<u64>,
    pub force_sealing: bool,
    pub start_time: u64,
    pub round_number: u64,
}

impl Configuration {
    fn new() -> Self {
        let tomlreader: Configuration = Configuration::init_config();
        tomlreader
    }

    pub fn init_config() -> Configuration {
        // get Current Directory
        let cwd: String = match env::current_dir() {
            Ok(c) => c.display().to_string(),
            Err(e) => panic!(
                "Error processing envirnment variable of current_exe dir - Err: {}!",
                e
            ),
        };
        let cwd: &Path = Path::new(&cwd);
        info!(">> Current Working Directory: {}", cwd.to_string_lossy());
        let config_file_path: PathBuf = cwd.join(&FILE_PATH.get_file_path());
        info!("path = {}", config_file_path.to_string_lossy());
        let mut config_file = match File::open(config_file_path) {
            Ok(f) => f,
            Err(e) => panic!("Error occurred opening config file:  Err: {}", e),
        };
        let mut config_file_str = String::new();
        config_file
            .read_to_string(&mut config_file_str)
            .expect("Error reading config");
        let conf_data: Configuration = toml::from_str(&config_file_str).unwrap();
        conf_data
    }
}

pub fn initialize_config(file_path: &str) {
    &FILE_PATH.set_file_path(&String::from(file_path));
    lazy_static::initialize(&AURA_CONFIG);
}

lazy_static! {
    static ref FILE_PATH: FilePath = FilePath::new();
}

lazy_static! {
    pub static ref AURA_CONFIG: Configuration = Configuration::new();
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_aura_config() {
        use super::*;
        initialize_config("../../../config.toml");
        info!("conf data = {:?}", AURA_CONFIG.validator_set);
    }
}
