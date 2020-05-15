use super::constants;
use super::message_traits::Message;
use std::convert::AsRef;
use utils::serializer::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, BinaryValue, ObjectHash)]
#[binary_value(codec = "bincode")]
pub struct SignedTransaction {
    pub txn: ::std::vec::Vec<u8>,
    pub app_name: String,
    pub header: ::std::collections::HashMap<std::string::String, std::string::String>,
    pub signature: std::vec::Vec<u8>,
}

impl Message for SignedTransaction {
    const TOPIC: &'static str = constants::NODE_MSG_TOPIC_STR[0];
    const MODULE_TOPIC: &'static str = constants::NODE;
    fn handler(&self) {
        // info!("i am SignedTransaction handler");
    }
}
