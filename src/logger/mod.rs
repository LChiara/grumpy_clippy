pub mod buffer; // Buffering and background flushing module
pub mod config; // Configuration management module
pub mod core; // Logger initialization and logging methods
pub mod macros;
pub mod model; // Data models like LogEntry and LogLevel // List of macros

pub use core::init_logger; // Re-exporting to provide easy access
