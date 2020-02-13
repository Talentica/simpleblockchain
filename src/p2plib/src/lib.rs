#[macro_use]
extern crate lazy_static;
pub mod constants;
pub mod messages;
pub mod p2pbehaviour;
pub mod simpleswarm;
pub mod txn_pool_p2p;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
