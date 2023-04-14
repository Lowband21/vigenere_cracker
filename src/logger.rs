// src/logger.rs

pub const LOG_LEVEL: LogState = LogState::DEBUG;

#[derive(Debug, PartialEq)]
pub enum LogState {
    DEBUG,
    TIMING,
    INFO,
}

pub fn log_debug(message: String) {
    if LOG_LEVEL == LogState::DEBUG {
        println!("DEBUG: {}", message)
    }
}

pub fn log_timing(message: String) {
    if LOG_LEVEL == LogState::TIMING || LOG_LEVEL == LogState::DEBUG {
        println!("TIMING: {}", message)
    }
}

pub fn log_info(message: String) {
    println!("INFO: {}", message)
}
