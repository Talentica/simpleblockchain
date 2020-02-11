pub trait CryptoKeypair<T, U> {
    fn generate() -> T;
    ///Generate from secrete key as byte array
    fn generate_from(secret: &mut [u8]) -> T;
    fn public(keypair: &T) -> U;
    fn secret(keypair: &T) -> Vec<u8>;
    fn sign(keypair: &T, msg: &[u8]) -> Vec<u8>;
}

pub type KeypairType = libp2p::identity::ed25519::Keypair;
pub type PublicKeyType = libp2p::identity::PublicKey;
pub type SecretKeyType = libp2p::identity::ed25519::SecretKey;

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

impl CryptoKeypair<KeypairType, PublicKeyType> for Keypair {
    fn generate() -> KeypairType {
        KeypairType::generate()
    }
    fn generate_from(secret_bytes: &mut [u8]) -> KeypairType {
        let secret_key = SecretKeyType::from_bytes(secret_bytes);
        KeypairType::from(secret_key.unwrap())
    }
    fn public(keypair: &KeypairType) -> PublicKeyType {
        libp2p::identity::PublicKey::Ed25519(keypair.public())
    }
    fn secret(keypair: &KeypairType) -> Vec<u8> {
        keypair.secret().as_ref().to_vec()
    }
    fn sign(keypair: &KeypairType, msg: &[u8]) -> Vec<u8> {
        keypair.sign(msg)
    }
}

impl Verify<PublicKeyType> for PublicKey {
    fn verify(public: &PublicKeyType, msg: &[u8], signature: &[u8]) -> bool {
        public.verify(msg, signature)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_generate() {
        use super::*;
        let kp = Keypair::generate();
        println!("pub : {:?}", hex::encode(kp.public().encode()));
        println!("secret : {:?}", hex::encode(Keypair::secret(&kp)));
        assert_eq!(true, true);
    }
    #[test]
    fn test_secret() {
        use super::*;
        let s = "728421f50913fefe04724bfad213624cb47ce9666445d2148bdbae0ba38e1884";
        let mut secret_bytes = hex::decode(s).expect("invalid secret");
        let kp = Keypair::generate_from(secret_bytes.as_mut_slice());
        assert_eq!(
            hex::encode(kp.public().encode()),
            "a986f7e76ec53d46d78f69fb2d76ae9378b439a6db43afba7a6761a01ccf3a7d"
        );
    }
    #[test]
    fn test_verify() {
        use super::*;
        let s = "97ba6f71a5311c4986e01798d525d0da8ee5c54acbf6ef7c3fadd1e2f624442f";
        let mut secret_bytes = hex::decode(s).expect("invalid secret");
        let kp = Keypair::generate_from(secret_bytes.as_mut_slice());
        println!("pub : {:?}", hex::encode(kp.public().encode()));
        // println!("secrete {:?}", hex::encode(kp.secret().as_ref()));
        let sign = Keypair::sign(&kp, b"Hello World");
        assert_eq!(
            true,
            PublicKey::verify(&Keypair::public(&kp), b"Hello World", sign.as_ref())
        );
    }
}
