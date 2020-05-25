extern crate chrono;

use log::LevelFilter;
use log4rs::append::rolling_file::policy::compound::roll::fixed_window::FixedWindowRoller;
use log4rs::append::rolling_file::policy::compound::{trigger::size::SizeTrigger, CompoundPolicy};
use log4rs::append::rolling_file::RollingFileAppender;
use log4rs::config::{Appender, Config, Root};
use log4rs::encode::pattern::PatternEncoder;
use log4rs::init_file;

pub fn logger_init_from_yml(file_path: &str) {
    init_file(&String::from(file_path), Default::default()).unwrap();
}

pub fn file_logger_init() {
    let size_limit = 5 * 1024 * 1024; // 5MB as max log file size to roll
    let size_trigger = SizeTrigger::new(size_limit);

    let window_size = 3; // log0, log1, log2
    let fixed_window_roller = FixedWindowRoller::builder()
        .build("log/log{}", window_size)
        .unwrap();

    let compound_policy =
        CompoundPolicy::new(Box::new(size_trigger), Box::new(fixed_window_roller));

    let is_logfile = RollingFileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{d} {l} {t} - {m}{n}")))
        .build("log/output.log", Box::new(compound_policy));
    let logfile = match is_logfile {
        Ok(logfile) => logfile,
        Err(e) => panic!("{:?}", e),
    };

    let is_config = Config::builder()
        .appender(Appender::builder().build("logfile", Box::new(logfile)))
        .build(Root::builder().appender("logfile").build(LevelFilter::Info));
    let config = match is_config {
        Ok(config) => config,
        Err(e) => panic!("{:?}", e),
    };

    let is_init_config = log4rs::init_config(config);
    if is_init_config.is_ok() {
        info!("Logger configuration initialized");
    } else {
        panic!("Error in log init_config function");
    }
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

    // fn to test file_defined static logger
    #[test]
    fn test_file_defined_logger() {
        // file_logger_init();
        info!("file defined logger is working");
    }
}
