// Module responsible for serializing and deserializing
use libp2p::multihash::{encode, Hash};
pub use serde::{Deserialize, Serialize};

/// serialize a generic type T
/// Needs trait Serialize to be implemented
/// can be done directly by using macro
/// #[derive(Serialize)] defined in serde
pub fn serialize<T>(to_ser: &T) -> Result<Vec<u8>, String>
where
    T: Serialize,
{
    match serde_cbor::to_vec(&to_ser) {
        Ok(value) => return Ok(value),
        Err(_) => return Err(String::from("couldn't to serialize")),
    };
}

/// returns the SHA3_256 hash of cbor value for
/// generic type T
pub fn serialize_hash256<T>(to_ser: &T) -> Result<Vec<u8>, String>
where
    T: Serialize,
{
    match serialize(&to_ser) {
        Result::Ok(to_hash) => {
            let encoded_value = match encode(Hash::SHA3256, &to_hash) {
                Ok(value) => value,
                Err(_) => return Err(String::from("couldn't able to compute serialize hash256")),
            };
            return Ok(encoded_value.to_vec());
        }
        Result::Err(_) => return Err(String::from("couldn't to serialize")),
    };
}

/// deserialize a vec<u8> slice and returns
/// generic type T
/// needs to implement Deserialze trait
/// with lifetime "a"
pub fn deserialize<'a, T>(slice: &'a [u8]) -> Result<T, String>
where
    T: Deserialize<'a>,
{
    let deserialize_value = serde_cbor::from_slice(&slice);
    match deserialize_value {
        Result::Ok(value) => return Result::Ok(value),
        Result::Err(_) => return Result::Err(String::from("couldn't to serialize")),
    };
}

#[cfg(test)]
mod tests_sbserde {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;
    use serde::{Deserialize, Serialize};

    // fn to test serializer
    #[test]
    fn test_ser() {
        // struct for unit testing
        #[derive(Debug, Serialize, Deserialize)]
        struct Friend {
            name: String,
        }
        // struct for unit testing
        #[derive(Debug, Serialize, Deserialize)]
        struct Mascot {
            name: String,
            species: String,
            year_of_birth: u32,
            f: Friend,
        }
        let ferris = Mascot {
            name: "Ferris".to_owned(),
            species: "crab".to_owned(),
            year_of_birth: 2015,
            f: Friend {
                name: "youtee".to_owned(),
            },
        };

        // check if hash of cbor is correct
        let hash_of_ser: Vec<u8> = match serialize_hash256(&ferris) {
            Result::Ok(value) => value,
            Result::Err(_) => vec![0],
        };
        assert_eq!(
            hash_of_ser,
            vec![
                22, 32, 243, 254, 107, 77, 41, 89, 227, 216, 65, 39, 75, 251, 101, 176, 236, 195,
                140, 255, 104, 236, 140, 34, 191, 18, 210, 4, 131, 108, 12, 184, 242, 73
            ]
        );
    }

    // fn to test deserializer
    #[test]
    fn test_ser_deser() {
        // struct for unit testing
        #[derive(Debug, Serialize, Deserialize)]
        struct Friend {
            name: String,
            // other: Box<Friend>;
        }
        // struct for unit testing
        #[derive(Debug, Serialize, Deserialize)]
        struct Mascot {
            name: String,
            species: String,
            year_of_birth: u32,
            f: Friend,
        }
        let ferris = Mascot {
            name: "Ferris".to_owned(),
            species: "crab".to_owned(),
            year_of_birth: 2015,
            f: Friend {
                name: "youtee".to_owned(),
            },
        };
        let serobj = serialize(&ferris).unwrap();

        let deserobj: Mascot = deserialize(&serobj).unwrap();
        let hash_of_ser = serialize_hash256(&ferris).unwrap();
        let hash_of_ser_deser = serialize_hash256(&deserobj).unwrap();
        assert_eq!(hash_of_ser, hash_of_ser_deser);
    }
}
