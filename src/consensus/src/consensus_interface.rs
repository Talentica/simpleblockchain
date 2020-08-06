extern crate aura;
extern crate message_handler;
extern crate utils;

use aura::aura_interface;
use futures::channel::mpsc::*;
use message_handler::messages::MessageTypes;
use poa::poa_interface;
use std::sync::{Arc, Mutex};
use utils::configreader::Configuration;

pub struct Consensus {}

impl Consensus {
    pub fn init_consensus(
        config: &Configuration,
        consensus_file_path: &str,
        sender: &mut Sender<Option<MessageTypes>>,
        msg_receiver: Arc<Mutex<Receiver<Option<Vec<u8>>>>>,
    ) {
        if config.node.consensus_name == "aura" {
            aura_interface::Aura::init_aura_consensus(
                config,
                &consensus_file_path,
                sender,
                msg_receiver,
            );
        } else if config.node.consensus_name == "poa" {
            poa_interface::Consensus::init_poa_consensus(
                config,
                &consensus_file_path,
                sender,
                msg_receiver,
            );
        } else {
            println!("kindly provide predefined consensus name string");
            std::process::exit(1);
        }
    }
}
