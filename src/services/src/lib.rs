pub mod transaction_service;

pub trait Service {
    //fn new() -> Self;
    fn start(&mut self) -> bool;
    fn stop(&self);
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
