//// Auto-generated using build.rs and proto files. Don't edit by hand. //// 
#[derive(Clone, PartialEq, Serialize, Deserialize, ::prost::Message)]
pub struct CryptoTransaction {
    #[prost(uint64, tag="1")]
    pub nonce: u64,
    #[prost(string, tag="2")]
    pub from: std::string::String,
    #[prost(string, tag="3")]
    pub to: std::string::String,
    #[prost(string, tag="4")]
    pub fxn_call: std::string::String,
    #[prost(bytes, tag="5")]
    pub payload: std::vec::Vec<u8>,
    #[prost(uint64, tag="6")]
    pub amount: u64,
}
//// Auto-generated using build.rs and proto files. Don't edit by hand. //// 
#[derive(Clone, PartialEq, Serialize, Deserialize, ::prost::Message)]
pub struct SignedTransaction {
    #[prost(message, optional, tag="1")]
    pub txn: ::std::option::Option<CryptoTransaction>,
    #[prost(map="string, string", tag="2")]
    pub header: ::std::collections::HashMap<std::string::String, std::string::String>,
    #[prost(bytes, tag="3")]
    pub signature: std::vec::Vec<u8>,
}
