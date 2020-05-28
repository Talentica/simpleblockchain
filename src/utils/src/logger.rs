extern crate chrono;

use log4rs::init_file;

pub fn logger_init_from_yml(file_path: &str) {
    init_file(&String::from(file_path), Default::default()).unwrap();
}

#[cfg(test)]
mod tests_logger {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;
    // fn to test logger functionality
    #[test]
    fn test_yml_logger() {
        logger_init_from_yml("log.yml");
        info!("yml logger is working");
    }
}
