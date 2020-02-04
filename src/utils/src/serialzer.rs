// Module responsible for serializing and deserializing
use libp2p::multihash::{encode, Hash};
use serde::{Deserialize, Serialize};

/// serialize a generic type T
/// Needs trait Serialize to be implemented
/// can be done directly by using macro
/// #[derive(Serialize)] defined in serde
pub fn serialize<T>(to_ser: &T) -> Vec<u8>
where
    T: Serialize,
{
    let servec = serde_cbor::to_vec(&to_ser).unwrap();
    servec
}

/// returns the SHA3_256 hash of cbor value for
/// generic type T
pub fn serialize_hash256<T>(to_ser: &T) -> Vec<u8>
where
    T: Serialize,
{
    let to_hash = serialize(to_ser);
    encode(Hash::SHA3256, &to_hash).unwrap().to_vec()
}

/// deserialize a vec<u8> slice and returns
/// generic type T
/// needs to implement Deserialze trait
/// with lifetime "a"
pub fn deserialize<'a, T>(slice: &'a [u8]) -> T
where
    T: Deserialize<'a>,
{
    serde_cbor::from_slice(&slice).unwrap()
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
        let hash_of_ser = serialize_hash256(&ferris);
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
        let serobj = serialize(&ferris);

        let deserobj: Mascot = deserialize(&serobj);
        let hash_of_ser = serialize_hash256(&ferris);
        let hash_of_ser_deser = serialize_hash256(&deserobj);
        assert_eq!(hash_of_ser, hash_of_ser_deser);
    }
}
