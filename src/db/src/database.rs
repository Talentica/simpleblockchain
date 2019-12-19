// Module responsible for serializing and deserializing
pub mod sbserde {

    use libp2p::multihash::{encode, Hash};
    use serde::{Deserialize, Serialize};

    // serialize a generic type T
    // Needs trait Serialize to be implemented
    // can be done directly by using macro
    // #[derive(Serialize)] defined in serde
    pub fn sb_ser<T>(to_ser: T) -> Vec<u8>
    where
        T: Serialize,
    {
        let servec = serde_cbor::to_vec(&to_ser).unwrap();
        servec
    }

    // returns the SHA3_256 hash of cbor value for
    // generic type T
    pub fn sb_ser_hash256<T>(to_ser: T) -> Vec<u8>
    where
        T: Serialize,
    {
        let to_hash = sb_ser(to_ser);
        encode(Hash::SHA3256, &to_hash).unwrap().to_vec()
    }

    // deserialize a vec<u8> slice and returns
    // generic type T
    // needs to implement Deserialze trait
    // with lifetime "a"
    pub fn sb_deser<'a, T>(slice: &'a [u8]) -> T
    where
        T: Deserialize<'a>,
    {
        serde_cbor::from_slice(&slice).unwrap()
    }
}

#[cfg(test)]
mod tests_sbserde {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::sbserde;
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
        let hash_of_ser = sbserde::sb_ser_hash256(&ferris);
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
        let serobj = sbserde::sb_ser(&ferris);

        let deserobj: Mascot = sbserde::sb_deser(&serobj);
        let hash_of_ser = sbserde::sb_ser_hash256(&ferris);
        let hash_of_ser_deser = sbserde::sb_ser_hash256(&deserobj);
        assert_eq!(hash_of_ser, hash_of_ser_deser);
        assert_eq!(deserobj.name, ferris.name);
        assert_eq!(deserobj.species, ferris.species);
        assert_eq!(deserobj.year_of_birth, ferris.year_of_birth);
    }
}

// module responsible for connecting to
// rocksdb
pub mod rdb_connection {
    use super::sbserde;
    use serde::{de::DeserializeOwned, Serialize};

    // enum with with Db or Nil object
    // TODO: see if this can be handled with
    // Option or Result
    pub enum DB {
        Db(rocksdb::DB),
        Nil,
    }
    // struct with actual connection object and
    // boolean to check if connected
    // TODO: see if connected is really needed
    pub struct Con {
        con_obj: DB,
        connected: bool,
    }

    impl Con {
        // initialize a new object to make connection
        pub fn new() -> Con {
            let obj = Con {
                con_obj: DB::Nil,
                connected: false,
            };
            obj
        }
        pub fn connect(&mut self) {
            // Todo :: handle if not able to connect
            // Take input name of DB(but review the
            //security pov with that functionality )
            self.con_obj = DB::Db(rocksdb::DB::open_default("rockdb/db").unwrap());
            self.connected = true;
        }

        // put a serializable object with its serialized value in
        // DB with key = hash of serialized value
        // returns None if Not connected to db
        pub fn put_in_db<T>(&self, object: T) -> Option<Vec<u8>>
        where
            T: Serialize,
        {
            match &self.con_obj {
                DB::Db(some) => {
                    let ser_obj = sbserde::sb_ser(&object);
                    let hash_obj = sbserde::sb_ser_hash256(&object);
                    let _res = some.put(&hash_obj, ser_obj);
                    Some(hash_obj)
                }
                DB::Nil => {
                    // println!("Not connected to db");
                    None
                }
            }
        }
        // input key as vec<u8> retrieves serialized value
        // of object from db , returns empty vector if
        // not connected to db
        pub fn getu8(self, slice: &[u8]) -> Vec<u8> {
            match self.con_obj {
                DB::Db(some) => {
                    //TODO: handle possible errors here
                    //some.get(slice) gives Result
                    // which can be Ok(Some(value)), Ok(None)
                    // or Err(e)
                    some.get(slice).unwrap().unwrap()
                }
                DB::Nil => vec![],
            }
        }
        // NOTE: WILL NOT WORK AS OF NOW
        // Expected functionality: take input key
        // retrive the corresponding value from db
        // use that value to get the deserialized object
        // if any error return None

        // ISSUES: Not able to get deserialized object using srde_cbor
        //     most likely issue with lifetime of the value
        //     (cannot be of lifetime that the caller function is to which the
        //     the object will be returned and eventually bound to a type
        pub fn get_from_db<T>(self, slice: &[u8]) -> Option<T>
        where
            T: DeserializeOwned,
        {
            match self.con_obj {
                DB::Db(some) => {
                    //let x = some.get(&slice);
                    ////TODO: rather than is_ok handle
                    //// all possibilities using match
                    //if x.is_ok(){
                    //    let v = &x.unwrap().unwrap();
                    //    // println!("slice is {:?}",v);
                    //    serde_cbor::from_slice(&v).unwrap()
                    //}
                    //None

                    // Following snippet can be used to
                    // handle all cases properly and
                    // prevent the thread from panicking later
                    let res = some.get(slice);
                    match res {
                        Ok(Some(value)) => {
                            serde_cbor::from_slice(&value).unwrap()
                            // let a = res.unwrap().unwrap();
                            //sbserde::sb_deser(res.unwrap()_)
                        }
                        Ok(None) => None,
                        Err(_e) => None,
                    }
                }
                DB::Nil => None,
            }
            // Ok(value)
        }
    }
}
