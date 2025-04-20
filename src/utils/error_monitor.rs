use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};
use ic_cdk::api::time;

use crate::models::error::{ErrorCode, ErrorSeverity, SquareError};

// Maximum error history size
const MAX_ERROR_HISTORY: usize = 100;

// Error statistics period (24 hours, in nanoseconds)
const ERROR_STATS_PERIOD: u64 = 24 * 60 * 60 * 1_000_000_000;

/// Error record
#[derive(Clone, Debug)]
struct ErrorRecord {
    /// Error code
    code: ErrorCode,
    /// Error message
    message: String,
    /// Error module
    module: String,
    /// Error function
    function: String,
    /// Error severity
    severity: ErrorSeverity,
    /// Error timestamp
    timestamp: u64,
}

/// Error statistics
#[derive(Clone, Debug)]
struct ErrorStats {
    /// Error code
    code: ErrorCode,
    /// Error count
    count: u64,
    /// First seen time
    first_seen: u64,
    /// Last seen time
    last_seen: u64,
    /// Module counts
    module_counts: HashMap<String, u64>,
}

/// Error monitoring system
thread_local! {
    static ERROR_MONITOR: RefCell<ErrorMonitor> = RefCell::new(ErrorMonitor::new());
}

/// Error monitoring system implementation
struct ErrorMonitor {
    /// Error history
    error_history: VecDeque<ErrorRecord>,
    /// Error statistics
    error_stats: HashMap<ErrorCode, ErrorStats>,
    /// Last cleanup time
    last_cleanup: u64,
}

impl ErrorMonitor {
    /// Create a new error monitoring system
    fn new() -> Self {
        Self {
            error_history: VecDeque::with_capacity(MAX_ERROR_HISTORY),
            error_stats: HashMap::new(),
            last_cleanup: time(),
        }
    }

    /// Record an error
    fn record_error(&mut self, error: &SquareError) {
        // Create error record
        let record = match error {
            SquareError::Enhanced(enhanced) => ErrorRecord {
                code: enhanced.code,
                message: enhanced.message.clone(),
                module: enhanced.context.module.clone(),
                function: enhanced.context.function.clone(),
                severity: enhanced.context.severity,
                timestamp: time(),
            },
            _ => ErrorRecord {
                code: error.code(),
                message: error.to_string(),
                module: "unknown".to_string(),
                function: "unknown".to_string(),
                severity: ErrorSeverity::Error,
                timestamp: time(),
            },
        };

        // Add to history
        self.error_history.push_back(record.clone());
        if self.error_history.len() > MAX_ERROR_HISTORY {
            self.error_history.pop_front();
        }

        // Update statistics
        let stats = self.error_stats.entry(record.code).or_insert_with(|| ErrorStats {
            code: record.code,
            count: 0,
            first_seen: record.timestamp,
            last_seen: record.timestamp,
            module_counts: HashMap::new(),
        });

        stats.count += 1;
        stats.last_seen = record.timestamp;
        *stats.module_counts.entry(record.module.clone()).or_insert(0) += 1;

        // Check if old statistics need cleanup
        self.cleanup_old_stats();
    }

    /// Clean up old statistics
    fn cleanup_old_stats(&mut self) {
        let now = time();
        if now - self.last_cleanup > ERROR_STATS_PERIOD {
            // Clean up statistics older than the statistics period
            let cutoff = now - ERROR_STATS_PERIOD;
            self.error_stats.retain(|_, stats| stats.last_seen > cutoff);
            self.last_cleanup = now;
        }
    }

    /// Get error history
    fn get_error_history(&self) -> Vec<String> {
        self.error_history
            .iter()
            .map(|record| {
                format!(
                    "[{}] [{}] [{}:{}] Error {}: {}",
                    match record.severity {
                        ErrorSeverity::Info => "INFO",
                        ErrorSeverity::Warning => "WARNING",
                        ErrorSeverity::Error => "ERROR",
                        ErrorSeverity::Critical => "CRITICAL",
                    },
                    record.timestamp,
                    record.module,
                    record.function,
                    record.code as u32,
                    record.message
                )
            })
            .collect()
    }

    /// Get error statistics
    fn get_error_stats(&self) -> Vec<(ErrorCode, u64, u64, u64)> {
        self.error_stats
            .values()
            .map(|stats| (stats.code, stats.count, stats.first_seen, stats.last_seen))
            .collect()
    }

    /// Get the most common errors
    fn get_most_common_errors(&self, limit: usize) -> Vec<(ErrorCode, u64)> {
        let mut errors: Vec<(ErrorCode, u64)> = self
            .error_stats
            .values()
            .map(|stats| (stats.code, stats.count))
            .collect();

        errors.sort_by(|a, b| b.1.cmp(&a.1));
        errors.truncate(limit);
        errors
    }

    /// Get the most critical errors
    fn get_critical_errors(&self) -> Vec<ErrorRecord> {
        self.error_history
            .iter()
            .filter(|record| record.severity == ErrorSeverity::Critical)
            .cloned()
            .collect()
    }
}

/// Record an error
pub fn record_error(error: &SquareError) {
    ERROR_MONITOR.with(|monitor| {
        monitor.borrow_mut().record_error(error);
    });
}

/// Get error history
pub fn get_error_history() -> Vec<String> {
    ERROR_MONITOR.with(|monitor| {
        monitor.borrow().get_error_history()
    })
}

/// Get error statistics
pub fn get_error_stats() -> Vec<(ErrorCode, u64, u64, u64)> {
    ERROR_MONITOR.with(|monitor| {
        monitor.borrow().get_error_stats()
    })
}

/// Get the most common errors
pub fn get_most_common_errors(limit: usize) -> Vec<(ErrorCode, u64)> {
    ERROR_MONITOR.with(|monitor| {
        monitor.borrow().get_most_common_errors(limit)
    })
}

/// Get the most critical errors
pub fn get_critical_errors() -> Vec<ErrorRecord> {
    ERROR_MONITOR.with(|monitor| {
        monitor.borrow().get_critical_errors()
    })
}
