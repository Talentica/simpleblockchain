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

impl PublicKey {
    pub fn from_string(public: &String) -> Option<PublicKeyType> {
        let decode_public_key = match hex::decode(public) {
            Ok(decode_public_key) => decode_public_key,
            Err(_) => return None,
        };

        match PublicKeyType::decode(&decode_public_key) {
            Ok(public_key) => return Some(public_key),
            Err(_) => return None,
        };
    }
}

impl Verify<PublicKeyType> for PublicKey {
    fn verify(public: &PublicKeyType, msg: &[u8], signature: &[u8]) -> bool {
        public.verify(msg, signature)
    }

    fn verify_from_encoded_pk(public: &String, msg: &[u8], signature: &[u8]) -> bool {
        match PublicKey::from_string(public) {
            Some(public_key) => public_key.verify(msg, signature),
            None => return false,
        }
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
        let s = "5f7c3fadd1e2f6225d0da8ee5c5a4435acbf6ef7c3fa543dd16e2544f624442f";
        let mut secret_bytes = hex::decode(s).expect("invalid secret");
        let kp = Keypair::generate_from(secret_bytes.as_mut_slice());
        info!("pub : {:?}", hex::encode(Keypair::public(&kp).encode()));
        // info!("secrete {:?}", hex::encode(kp.secret().as_ref()));
        let sign = Keypair::sign(&kp, b"Hello World");
        assert_eq!(
            true,
            PublicKey::verify(&Keypair::public(&kp), b"Hello World", sign.as_ref())
        );
    }
}
