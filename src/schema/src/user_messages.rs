//// Auto-generated using build.rs and proto files. Don't edit by hand. //// 
#[derive(Clone, PartialEq, Serialize, Deserialize, ::prost::Message)]
pub struct Transaction {
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
