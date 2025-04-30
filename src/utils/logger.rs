use std::cell::RefCell;
use candid::{CandidType, Deserialize};
use serde::Serialize;

// Maximum number of log entries to keep
const MAX_LOG_ENTRIES: usize = 1000;

#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct LogEntry {
    timestamp: u64,
    message: String,
}

thread_local! {
    static LOG_BUFFER: RefCell<Vec<LogEntry>> = RefCell::new(Vec::new());
}

/// Log a message to both the standard IC log and our custom log buffer
pub fn log(message: &str) {
    ic_cdk::println!("{}", message);
    
    LOG_BUFFER.with(|buffer| {
        let mut buffer = buffer.borrow_mut();
        if buffer.len() >= MAX_LOG_ENTRIES {
            buffer.remove(0);
        }
        buffer.push(LogEntry {
            timestamp: ic_cdk::api::time() / 1_000_000, // Convert nanoseconds to milliseconds
            message: message.to_string(),
        });
    });
}

/// Format and log a message with arguments
pub fn log_fmt(fmt: impl AsRef<str>, args: impl std::fmt::Display) {
    let message = format!("{}", fmt.as_ref().replace("{:?}", &format!("{}", args)));
    log(&message);
}

/// Get all logs from the buffer
pub fn get_all_logs() -> Vec<LogEntry> {
    LOG_BUFFER.with(|buffer| buffer.borrow().clone())
}

/// Get the most recent logs, limited by count
pub fn get_recent_logs(count: usize) -> Vec<LogEntry> {
    LOG_BUFFER.with(|buffer| {
        let buffer = buffer.borrow();
        let start = if buffer.len() > count {
            buffer.len() - count
        } else {
            0
        };
        buffer[start..].to_vec()
    })
}

/// Clear all logs from the buffer
pub fn clear_logs() {
    LOG_BUFFER.with(|buffer| {
        buffer.borrow_mut().clear();
    });
}

/// Save logs to stable storage during upgrades
pub fn save_logs() -> Vec<LogEntry> {
    get_all_logs()
}

/// Restore logs from stable storage after upgrades
pub fn restore_logs(logs: Vec<LogEntry>) {
    LOG_BUFFER.with(|buffer| {
        let mut buffer = buffer.borrow_mut();
        buffer.clear();
        buffer.extend(logs);
        
        // Ensure we don't exceed the maximum
        if buffer.len() > MAX_LOG_ENTRIES {
            *buffer = buffer.iter().skip(buffer.len() - MAX_LOG_ENTRIES).cloned().collect();
        }
    });
}
