use super::*;
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::io::Read;
use toml;

use crypto::keypair::CryptoKeypair;
use crypto::keypair::Keypair;

use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub enum NODETYPE {
    FullNode,
    Validator,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct TomlReaderConfig {
    pub public: String,
    pub secret: String,
    node_type: String,
    genesis_block: bool,
    //p2p
    p2p_port: u16,
    //db config
    dbpath: String,
    //client config
    client_port: u32,
    client_host: String,
    client_apps: Vec<String>,
}

#[derive(Debug)]
pub struct Configuration {
    //Node
    pub node: Node,
    //peers list

    //Database config
    pub db: Database,
}

impl Configuration {
    fn new() -> Self {
        let tomlreader: TomlReaderConfig = Configuration::init_config();
        let mut secret = hex::decode(tomlreader.secret).expect("invalid secret");
        let keypair = Keypair::generate_from(secret.as_mut_slice());
        if hex::encode(keypair.public().encode()) != tomlreader.public {
            panic!("Secret and public key pair is invalid");
        }
        let mut node_type: NODETYPE = NODETYPE::Validator;
        if tomlreader.node_type.to_ascii_lowercase() == "fullnode" {
            node_type = NODETYPE::FullNode
        } else if tomlreader.node_type.to_ascii_lowercase() != "validator" {
            panic!("node type not defined properly");
        }
        let node_obj: Node = Node {
            public: Keypair::public(&keypair),
            hex_public: tomlreader.public,
            keypair: keypair,
            node_type,
            genesis_block: tomlreader.genesis_block,
            p2p_port: tomlreader.p2p_port,
            client_host: tomlreader.client_host,
            client_port: tomlreader.client_port,
            client_apps: tomlreader.client_apps.to_vec(),
        };
        let db_path: Database = Database {
            dbpath: "utils/rocksdb".to_string(),
        };
        let conf_obj = Configuration {
            node: node_obj,
            db: db_path,
        };
        conf_obj
    }
    pub fn init_config() -> TomlReaderConfig {
        // get Current Directory
        let cwd: String = match env::current_dir() {
            Ok(c) => c.display().to_string(),
            Err(e) => panic!(
                "Error processing envirnment variable of current_exe dir - Err: {}!",
                e
            ),
        };
        info!(">> Current Working Directory: {}", cwd);
        let config_file_path: String = cwd + &String::from("/config.toml");
        info!("path = {}", config_file_path);
        let mut config_file = match File::open(config_file_path) {
            Ok(f) => f,
            Err(e) => panic!("Error occurred opening config file:  Err: {}", e),
        };
        let mut config_file_str = String::new();
        config_file
            .read_to_string(&mut config_file_str)
            .expect("Error reading config");
        let conf_data: configreader::TomlReaderConfig = toml::from_str(&config_file_str).unwrap();
        conf_data
    }
}

#[derive(Debug)]
pub struct Node {
    //Node config
    pub public: crypto::keypair::PublicKeyType,
    pub hex_public: String,
    pub keypair: crypto::keypair::KeypairType,
    pub node_type: NODETYPE,
    pub genesis_block: bool,
    pub p2p_port: u16,
    pub client_host: String,
    pub client_port: u32,
    pub client_apps: Vec<String>,
}

#[derive(Debug)]
pub struct Database {
    pub dbpath: String,
}

lazy_static! {
    pub static ref GLOBAL_CONFIG: Configuration = Configuration::new();
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_config() {
        use super::*;
        info!("conf data = {:?}", configreader::GLOBAL_CONFIG.node);
        assert_eq!(
            hex::encode(configreader::GLOBAL_CONFIG.node.keypair.public().encode()),
            configreader::GLOBAL_CONFIG.node.hex_public
        );
    }
}
