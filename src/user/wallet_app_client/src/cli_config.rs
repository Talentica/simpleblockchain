use super::*;
use std::env;
use std::fs::File;
use std::io::prelude::*;
use toml;

use serde::{Deserialize, Serialize};

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
        println!(">> Current Working Directory: {}", cwd);
        let config_file_path: String = cwd + &String::from("/cli_config.toml");
        println!("path = {}", config_file_path);
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
    pub static ref GLOBAL_CONFIG: cli_config::Configuration = Configuration::new();
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_config() {
        use super::*;
        println!("conf data = {:?}", cli_config::GLOBAL_CONFIG.url);
    }
}
