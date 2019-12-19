pub trait CryptoKeypair<T, U> {
    fn generate() -> T;
    ///Generate from secrete key as byte array
    fn generate_from(secret: &mut [u8]) -> T;
    fn public(keypair: &T) -> U;
    fn secret(keypair: &T) -> Vec<u8>;
    fn sign(keypair: &T, msg: &[u8]) -> Vec<u8>;
}

pub trait Sign<T> {
    fn sign(secret: &T, msg: &[u8]) -> Vec<u8>;
}

pub trait Verify<T> {
    fn verify(public: &T, msg: &[u8], signature: &[u8]) -> bool;
}

#[derive(Debug)]
pub struct Keypair {}

#[derive(Debug)]
pub struct PublicKey {}

#[derive(Debug)]
pub struct SecretKey {}

impl CryptoKeypair<libp2p::identity::ed25519::Keypair, libp2p::identity::ed25519::PublicKey>
    for Keypair
{
    fn generate() -> libp2p::identity::ed25519::Keypair {
        libp2p::identity::ed25519::Keypair::generate()
    }
    fn generate_from(secret_bytes: &mut [u8]) -> libp2p::identity::ed25519::Keypair {
        let secret_key = libp2p::identity::ed25519::SecretKey::from_bytes(secret_bytes);
        libp2p::identity::ed25519::Keypair::from(secret_key.unwrap())
    }
    fn public(
        keypair: &libp2p::identity::ed25519::Keypair,
    ) -> libp2p::identity::ed25519::PublicKey {
        keypair.public()
    }
    fn secret(keypair: &libp2p::identity::ed25519::Keypair) -> Vec<u8> {
        keypair.secret().as_ref().to_vec()
    }
    fn sign(keypair: &libp2p::identity::ed25519::Keypair, msg: &[u8]) -> Vec<u8> {
        keypair.sign(msg)
    }
}

impl Verify<libp2p::identity::ed25519::PublicKey> for PublicKey {
    fn verify(public: &libp2p::identity::ed25519::PublicKey, msg: &[u8], signature: &[u8]) -> bool {
        public.verify(msg, signature)
    }
}
