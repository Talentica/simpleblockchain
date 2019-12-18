pub struct SimpleKeypair<T> {
    //P2PEd25519(libp2p::core::identity::ed25519::Keypair),
    pub keypair: Option<T>,
}

pub struct PublicKey<T> {
    //P2PPublic(libp2p::core::identity::PublicKey),
    pub public: Option<T>,
}

pub struct SecretKey<T> {
    //P2PED25519Secret(libp2p::identity::ed25519::SecretKey),
    pub secret: Option<T>,
}

pub trait CryptoKeypair<T> {
    fn generate() -> T;
    ///Generate from secrete key as byte array
    fn generate_from(secret: &mut [u8]) -> T;
    fn public(keypair: &T) -> Vec<u8>;
    fn secret(keypair: &T) -> Vec<u8>;
    fn sign(keypair: &T, msg: &[u8]) -> Vec<u8>;
}

pub trait Sign<T> {
    fn sign(secret: &SecretKey<T>, msg: &[u8]) -> Vec<u8>;
}

pub trait Verify<T> {
    fn verify(public: &PublicKey<T>, msg: &[u8], signature: &[u8]) -> bool;
}

#[derive(Debug)]
pub struct Keypair {}

impl CryptoKeypair<libp2p::identity::ed25519::Keypair> for Keypair {
    fn generate() -> libp2p::identity::ed25519::Keypair {
        libp2p::identity::ed25519::Keypair::generate()
    }
    fn generate_from(secret_bytes: &mut [u8]) -> libp2p::identity::ed25519::Keypair {
        let secret_key = libp2p::identity::ed25519::SecretKey::from_bytes(secret_bytes);
        libp2p::identity::ed25519::Keypair::from(secret_key.unwrap())
    }
    fn public(keypair: &libp2p::identity::ed25519::Keypair) -> Vec<u8> {
        keypair.public().encode().to_vec()
    }
    fn secret(keypair: &libp2p::identity::ed25519::Keypair) -> Vec<u8> {
        keypair.secret().as_ref().to_vec()
    }
    fn sign(keypair: &libp2p::identity::ed25519::Keypair, msg: &[u8]) -> Vec<u8> {
        keypair.sign(msg)
    }
}

impl Verify<libp2p::identity::ed25519::PublicKey> for libp2p::identity::ed25519::PublicKey {
    fn verify(
        public: &PublicKey<libp2p::identity::ed25519::PublicKey>,
        msg: &[u8],
        signature: &[u8],
    ) -> bool {
        public.public.as_ref().unwrap().verify(msg, signature)
    }
}
// pub use libp2p::core::identity::ed25519::Keypair as SimpleKeypair;

// pub struct SimpleKeypair {
//     pub keypair: libp2p::core::identity::ed25519::Keypair,
// }
