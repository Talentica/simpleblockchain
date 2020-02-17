use super::*;
use std::env;
use std::fs;
use std::fs::{File, OpenOptions};
use std::io;
use std::io::prelude::*;
use toml;

use serde::{Deserialize, Serialize};

use toml::Value;

#[derive(Debug, Serialize, Deserialize)]
pub struct Configuration {
    pub public_keys: Vec<String>,
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
        println!(">> Current Working Directory: {}", cwd);
        let config_file_path: String = cwd + &String::from("/config.toml");
        println!("path = {}", config_file_path);
        let mut config_file = match File::open(config_file_path) {
            Ok(f) => f,
            Err(e) => panic!("Error occurred opening config file:  Err: {}", e),
        };
        let mut config_file_str = String::new();
        config_file
            .read_to_string(&mut config_file_str)
            .expect("Error reading config");
        let conf_data: consensus_config::Configuration = toml::from_str(&config_file_str).unwrap();
        conf_data
    }
}

lazy_static! {
    pub static ref GLOBAL_CONFIG: consensus_config::Configuration = Configuration::new();
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_config() {
        use super::*;
        println!("conf data = {:?}", consensus_config::GLOBAL_CONFIG.public_keys);
    }
}
