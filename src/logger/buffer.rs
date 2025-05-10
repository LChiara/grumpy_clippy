use crate::logger::model::LogEntry;
use crossbeam::channel::{Receiver, select};
use std::{
    fs::OpenOptions,
    io::{BufWriter, Write},
    thread,
    time::{Duration, Instant},
};

/// Public struct holding the `Sender` used to push log entries to the background worker.
pub struct LogBuffer {
    pub sender: crossbeam::channel::Sender<LogEntry>,
}

/// Initializes the log buffer and spawns a background thread to flush entries.
pub fn init_buffer(file_path: String, is_json: bool) -> LogBuffer {
    let (sender, receiver) = crossbeam::channel::unbounded();

    // Spawning the background thread
    thread::spawn(move || {
        buffer_worker(receiver, file_path, is_json);
    });

    LogBuffer { sender }
}

/// Background worker that buffers log entries and periodically writes them to disk.
fn buffer_worker(receiver: Receiver<LogEntry>, path: String, is_json: bool) {
    // Open the file for append; create if not exists.
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .unwrap();
    let mut writer = BufWriter::new(file);

    // Buffer and timer
    let mut buffer: Vec<LogEntry> = Vec::new();
    let mut last_flush = Instant::now();

    loop {
        // Attempt to receive a new log entry, or wait until timeout
        select! {
            recv(receiver) -> msg => {
                match msg {
                    Ok(entry) => {
                        buffer.push(entry);
                    },
                    Err(_) => break, // Sender dropped, exit thread
                }
            },
            default(Duration::from_secs(1)) => {} // Check periodically
        }

        // If time elapsed and buffer is not empty, flush to disk
        if last_flush.elapsed() >= Duration::from_secs(5) && !buffer.is_empty() {
            for entry in buffer.drain(..) {
                let line = if is_json {
                    serde_json::to_string(&entry).unwrap() + "\n"
                } else {
                    entry.format() + "\n"
                };
                writer.write_all(line.as_bytes()).unwrap();
            }
            writer.flush().unwrap();
            last_flush = Instant::now();
        }
    }
}
