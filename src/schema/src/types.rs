use exonum_crypto::Hash;

pub type SimpleHash = exonum_crypto::Hash;
pub type GetHash = fn(&[u8]) -> SimpleHash;

pub const GETHASH: GetHash = exonum_crypto::hash;
