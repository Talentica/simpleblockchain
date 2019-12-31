pub trait CryptoKeypair<T, U> {
    fn generate() -> T;
    ///Generate from secrete key as byte array
    fn generate_from(secret: &mut [u8]) -> T;
    fn public(keypair: &T) -> U;
    fn secret(keypair: &T) -> Vec<u8>;
    fn sign(keypair: &T, msg: &[u8]) -> Vec<u8>;
}

pub type KeypairType = libp2p::identity::ed25519::Keypair;
pub type PublicKeyType = libp2p::identity::ed25519::PublicKey;
pub type SecretKeyType = libp2p::identity::ed25519::SecretKey;

pub trait Sign<T> {
    fn sign(secret: &T, msg: &[u8]) -> Vec<u8>;
}

pub trait Verify<T> {
    fn verify(public: &T, msg: &[u8], signature: &[u8]) -> bool;
    fn verify_from_encoded_pk(public: &String, msg: &[u8], signature: &[u8]) -> bool;
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
        keypair.public()
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

    fn verify_from_encoded_pk(public: &String, msg: &[u8], signature: &[u8]) -> bool {
        // println!("decoding");
        let decode_public_key = match hex::decode(public) {
            Ok(decode_public_key) => decode_public_key,
            Err(_e) => return false,
        };

        let public_key = match PublicKeyType::decode(&decode_public_key) {
            Ok(public_key) => public_key,
            Err(_e) => return false,
        };
        // println!("decoded");
        public_key.verify(msg, signature)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_secret() {
        use super::*;
        let s = "97ba6f71a5311c4986e01798d525d0da8ee5c54acbf6ef7c3fadd1e2f624442f";
        let mut secret_bytes = hex::decode(s).expect("invalid secret");
        let kp = Keypair::generate_from(secret_bytes.as_mut_slice());
        assert_eq!(
            hex::encode(kp.public().encode()),
            "2c8a35450e1d198e3834d933a35962600c33d1d0f8f6481d6e08f140791374d0"
        );
    }
    #[test]
    fn test_verify() {
        use super::*;
        let s = "97ba6f71a5311c4986e01798d525d0da8ee5c54acbf6ef7c3fadd1e2f624442f";
        let mut secret_bytes = hex::decode(s).expect("invalid secret");
        let kp = Keypair::generate_from(secret_bytes.as_mut_slice());
        println!("pub : {:?}", hex::encode(Keypair::public(&kp).encode()));
        // println!("secrete {:?}", hex::encode(kp.secret().as_ref()));
        let sign = Keypair::sign(&kp, b"Hello World");
        assert_eq!(
            true,
            PublicKey::verify(&Keypair::public(&kp), b"Hello World", sign.as_ref())
        );
    }
}
