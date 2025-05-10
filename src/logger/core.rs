//! Logger module for the `rust_logger` library.
//!
//! This module provides core logging functionalities, including
//! structured logging with optional buffering and configurable outputs.
//!
//! Use [`log`] to write logs and [`init_logger`] to initialize the logger
//! with a [`Config`](crate::config::FinalConfig) object.

use crate::logger::buffer::{LogBuffer, init_buffer};
use crate::logger::config::FinalConfig;
use crate::logger::model::{LogEntry, LogLevel};
use once_cell::sync::Lazy;
use std::sync::Mutex;

/// Global singleton logger, protected by a Mutex
static LOGGER: Lazy<Mutex<Option<Logger>>> = Lazy::new(|| Mutex::new(None));

/// Internal logger struct holding config and log buffer
pub struct Logger {
    pub config: FinalConfig,
    pub buffer: Option<LogBuffer>, // optional if logging to stdout only
}

/// Initializes the global logger
pub fn init_logger(config: FinalConfig) {
    let buffer = if config.to_file {
        let file_path = match config.log_type.as_str() {
            "json" => config.file_name.clone() + ".json",
            "txt" => config.file_name.clone() + ".txt",
            _ => panic!("Unknown log type"),
        };
        Some(init_buffer(file_path, config.log_type == "json"))
    } else {
        None
    };

    let logger = Logger { config, buffer };

    // Store logger in singleton
    let mut global_logger = LOGGER.lock().unwrap();
    *global_logger = Some(logger);
}

/// Logs a new message using the configured output
pub fn log(level: LogLevel, message: String) {
    let logger = LOGGER.lock().unwrap();
    if let Some(ref logger) = *logger {
        let entry = LogEntry::new(level, message);
        match &logger.buffer {
            Some(buffer) => {
                // Send to background thread
                buffer.sender.send(entry).unwrap();
            }
            None => {
                // Immediate stdout output (no file logging)
                if logger.config.log_type == "json" {
                    println!("{}", serde_json::to_string(&entry).unwrap());
                } else {
                    println!("{}", entry.format());
                }
            }
        }
    }
}
