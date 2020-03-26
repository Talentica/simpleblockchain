use exonum_crypto::Hash;
//// Auto-generated using build.rs and proto files. Don't edit by hand. ////
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, BinaryValue, ObjectHash)]
#[binary_value(codec = "bincode")]
pub struct CryptoTransaction {
    pub from: std::string::String,
    pub fxn_call: std::string::String,
    pub payload: std::vec::Vec<DataTypes>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, BinaryValue, ObjectHash)]
#[binary_value(codec = "bincode")]
pub enum DataTypes {
    BoolVal(bool),
    IntVal(i32),
    HashVal(Hash),
    StringVal(String),
    VecHashVal(Vec<Hash>),
    VecStringVal(Vec<String>),
}

//// Auto-generated using build.rs and proto files. Don't edit by hand. ////
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, BinaryValue, ObjectHash)]
#[binary_value(codec = "bincode")]
pub struct SignedTransaction {
    pub txn: ::std::option::Option<CryptoTransaction>,
    pub header: ::std::collections::HashMap<std::string::String, std::string::String>,
    pub signature: std::vec::Vec<u8>,
}
