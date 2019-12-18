pub mod crypto;
use crypto::keypair::CryptoKeypair;
use crypto::keypair::Keypair;
use hex;

#[cfg(test)]
mod tests {
    #[test]
    fn test_secret() {
        use super::*;
        let s = "97ba6f71a5311c4986e01798d525d0da8ee5c54acbf6ef7c3fadd1e2f624442f";
        let mut secret_bytes = hex::decode(s).expect("invalid secret");
        let kp = crypto::keypair::Keypair::generate_from(secret_bytes.as_mut_slice());
        println!("pub : {:?}", hex::encode(kp.public().encode()));
        // println!("secrete {:?}", hex::encode(kp.secret().as_ref()));\\
        assert_eq!(
            hex::encode(kp.public().encode()),
            "2c8a35450e1d198e3834d933a35962600c33d1d0f8f6481d6e08f140791374d0"
        );
    }
}
