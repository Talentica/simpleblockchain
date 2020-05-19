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
