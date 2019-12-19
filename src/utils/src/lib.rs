pub mod crypto;
use crypto::keypair::CryptoKeypair;
use crypto::keypair::Keypair;
use crypto::keypair::PublicKey;
use crypto::keypair::Verify;
use hex;

#[cfg(test)]
mod tests {
    #[test]
    fn test_secret() {
        use super::*;
        let s = "97ba6f71a5311c4986e01798d525d0da8ee5c54acbf6ef7c3fadd1e2f624442f";
        let mut secret_bytes = hex::decode(s).expect("invalid secret");
        let kp = crypto::keypair::Keypair::generate_from(secret_bytes.as_mut_slice());
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
