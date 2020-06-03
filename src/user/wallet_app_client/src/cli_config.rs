use super::*;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use toml;

#[derive(Debug)]
pub struct FilePath {
    path: Arc<std::sync::Mutex<String>>,
}

impl FilePath {
    pub fn new() -> FilePath {
        FilePath {
            path: Arc::new(Mutex::new(String::new())),
        }
    }

    pub fn get_file_path(&self) -> String {
        let locked_path = self.path.lock().unwrap();
        let file_path: String = String::from(locked_path.clone());
        file_path
    }

    pub fn set_file_path(&self, file_path: &String) {
        let mut locked_path = self.path.lock().unwrap();
        *locked_path = file_path.clone();
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Configuration {
    pub url: String,
    pub secret: String,
    pub public: String,
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
        let conf_data: cli_config::Configuration = toml::from_str(&config_file_str).unwrap();
        conf_data
    }
}

lazy_static! {
    pub static ref FILE_PATH: FilePath = FilePath::new();
}

lazy_static! {
    pub static ref GLOBAL_CONFIG: cli_config::Configuration = Configuration::new();
}

#[cfg(test)]
mod tests {
    use super::*;
    use utils::keypair::{CryptoKeypair, Keypair};

    #[test]
    fn test_config() {
        &FILE_PATH.set_file_path(&String::from("../../../cli_config.toml"));
        info!("conf data = {:?}", cli_config::GLOBAL_CONFIG.url);
        let mut secret = hex::decode(GLOBAL_CONFIG.secret.clone()).expect("invalid secret");
        let keypair = Keypair::generate_from(secret.as_mut_slice());
        assert_eq!(hex::encode(keypair.public().encode()), GLOBAL_CONFIG.public);
    }
}
