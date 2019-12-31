pub mod block;
pub fn hello_world() {
    println!("Hello World ",);
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
