// Module responsible for serializing and deserializing
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

    // fn to test serializer deserializer
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
        assert_eq!(ferris.f.name, deserobj.f.name);
        assert_eq!(ferris.name, deserobj.name);
        assert_eq!(ferris.year_of_birth, deserobj.year_of_birth);
        assert_eq!(ferris.species, deserobj.species);
    }
}
