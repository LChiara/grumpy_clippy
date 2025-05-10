use serde::{Deserialize, Serialize};
use std::fmt;

/// Represents a log entry used in the logger.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub level: LogLevel,
    pub message: String,
    pub timestamp: String,
}

impl LogEntry {
    pub fn new(level: LogLevel, message: String) -> Self {
        LogEntry {
            level,
            message,
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }

    pub fn format(&self) -> String {
        format!("[{}] {}: {}", self.timestamp, self.level, self.message)
    }
}

/// Represents a log level used in the logger.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogLevel {
    Info,
    Warn,
    Error,
    Debug,
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
