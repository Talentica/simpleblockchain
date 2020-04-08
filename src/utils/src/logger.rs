extern crate chrono;
extern crate env_logger;

use chrono::Local;
use env_logger::Builder;
use log::LevelFilter;
use log4rs::append::rolling_file::policy::compound::roll::fixed_window::FixedWindowRoller;
use log4rs::append::rolling_file::policy::compound::{trigger::size::SizeTrigger, CompoundPolicy};
use log4rs::append::rolling_file::RollingFileAppender;
use log4rs::config::{Appender, Config, Root};
use log4rs::encode::pattern::PatternEncoder;
use log4rs::init_file;

use std::io::Write;

pub fn file_logger_init_from_yml(file_path: &String) {
    init_file(file_path, Default::default()).unwrap();
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
        debug!("Logger configuration initialized");
    } else {
        panic!("Error in log init_config function");
    }
}

pub fn console_logger_init(file_path: &String) {
    init_file(file_path, Default::default()).unwrap();
}
