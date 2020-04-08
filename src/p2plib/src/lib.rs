#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate log;

pub mod constants;
pub mod message_sender;
pub mod messages;
pub mod p2pbehaviour;
pub mod simpleswarm;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
