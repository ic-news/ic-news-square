
use ic_cdk::trap;

use crate::models::error::{SquareError, ErrorCode, ErrorSeverity};
use crate::utils::error_monitor;

/// Global error interceptor
/// This module provides functionality to intercept and handle uncaught errors

/// Handle an uncaught error
pub fn handle_uncaught_error(error: String) {
    // Create a system error
    let system_error = SquareError::new(
        ErrorCode::UnexpectedError,
        format!("Uncaught error: {}", error),
        "system",
        "global_interceptor",
        ErrorSeverity::Critical,
    );
    
    // Log the error
    system_error.log();
    
    // Record the error in the monitoring system
    error_monitor::record_error(&system_error);
}

/// Handle a critical system error
pub fn handle_critical_error(error: SquareError) -> ! {
    // Log the error
    error.log();
    
    // Record the error in the monitoring system
    error_monitor::record_error(&error);
    
    // Trap with error message
    trap(&format!("Critical system error: {}", error));
}

/// Create a panic hook to catch Rust panics
pub fn set_panic_hook() {
    std::panic::set_hook(Box::new(|panic_info| {
        let message = if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
            s.to_string()
        } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
            s.clone()
        } else {
            "Unknown panic".to_string()
        };
        
        let location = if let Some(location) = panic_info.location() {
            format!("{}:{}", location.file(), location.line())
        } else {
            "unknown location".to_string()
        };
        
        let panic_error = SquareError::new(
            ErrorCode::SystemError,
            format!("Panic: {} at {}", message, location),
            "system",
            "panic_hook",
            ErrorSeverity::Critical,
        );
        
        // Log the error
        panic_error.log();
        
        // Record the error in the monitoring system
        error_monitor::record_error(&panic_error);
    }));
}

/// Register a global error handler
pub fn register_global_error_handler() {
    // Set panic hook
    set_panic_hook();
}
