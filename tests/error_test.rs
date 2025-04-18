use ic_news_square::models::error::{SquareError, SquareResult};
use ic_news_square::utils::error_handler::*;
use ic_news_square::utils::error_monitor;

/// Test utility to verify error handling functionality
/// This module provides functions to test and validate the error handling system

/// Test function to generate various types of errors
/// This is useful for testing the error monitoring system
pub fn generate_test_errors() -> SquareResult<()> {
    const MODULE: &str = "utils::error_test";
    const FUNCTION: &str = "generate_test_errors";
    
    // Generate a not found error
    let not_found = not_found_error("User", "user123", MODULE, FUNCTION);
    error_monitor::record_error(&not_found);
    
    // Generate an already exists error
    let already_exists = already_exists_error("Post", "post456", MODULE, FUNCTION);
    error_monitor::record_error(&already_exists);
    
    // Generate an unauthorized error
    let unauthorized = unauthorized_error("Missing permissions", MODULE, FUNCTION);
    error_monitor::record_error(&unauthorized);
    
    // Generate a validation error
    let validation = validation_error("Title cannot be empty", MODULE, FUNCTION);
    error_monitor::record_error(&validation);
    
    // Generate a content too long error
    let content_too_long = content_too_long_error("Comment", 500, 750, MODULE, FUNCTION);
    error_monitor::record_error(&content_too_long);
    
    // Generate an invalid operation error
    let invalid_operation = invalid_operation_error(
        "delete_post", 
        "Cannot delete a post with active comments", 
        MODULE, 
        FUNCTION
    );
    error_monitor::record_error(&invalid_operation);
    
    // Generate a system error
    let system = system_error("Database connection failed", MODULE, FUNCTION);
    error_monitor::record_error(&system);
    
    // Generate a data inconsistency error
    let data_inconsistency = data_inconsistency_error(
        "User", 
        "user789", 
        "User exists but profile is missing", 
        MODULE, 
        FUNCTION
    );
    error_monitor::record_error(&data_inconsistency);
    
    // Generate a resource unavailable error
    let resource_unavailable = resource_unavailable_error(
        "Database", 
        "primary", 
        MODULE, 
        FUNCTION
    );
    error_monitor::record_error(&resource_unavailable);
    
    // Generate a rate limit error
    let rate_limit = rate_limit_error(
        "create_post", 
        10, 
        MODULE, 
        FUNCTION
    );
    error_monitor::record_error(&rate_limit);
    
    // Generate a dependency error
    let dependency = dependency_error(
        "token_service", 
        "Service returned error code 500", 
        MODULE, 
        FUNCTION
    );
    error_monitor::record_error(&dependency);
    
    // Generate a permission denied error
    let permission_denied = permission_denied_error(
        "update_user", 
        "Only admins can update other users", 
        MODULE, 
        FUNCTION
    );
    error_monitor::record_error(&permission_denied);
    
    // Generate a service unavailable error
    let service_unavailable = service_unavailable_error(
        "notification_service", 
        "Service is under maintenance", 
        MODULE, 
        FUNCTION
    );
    error_monitor::record_error(&service_unavailable);
    
    // Generate a quota exceeded error
    let quota_exceeded = quota_exceeded_error(
        "storage", 
        1024 * 1024 * 100, // 100MB
        MODULE, 
        FUNCTION
    );
    error_monitor::record_error(&quota_exceeded);
    
    // Return success to indicate the test completed
    Ok(())
}

/// Test function to verify error recovery hints
pub fn test_error_recovery_hints() -> Vec<String> {
    const MODULE: &str = "utils::error_test";
    const FUNCTION: &str = "test_error_recovery_hints";
    
    let mut recovery_hints = Vec::new();
    
    // Test recoverable errors
    let resource_error = resource_unavailable_error(
        "Database", 
        "primary", 
        MODULE, 
        FUNCTION
    );
    
    // recovery_hint method has been removed, adding default message
    recovery_hints.push(format!("Resource error: {}", resource_error));
    
    let rate_error = rate_limit_error(
        "create_post", 
        10, 
        MODULE, 
        FUNCTION
    );
    
    // recovery_hint method has been removed, adding default message
    recovery_hints.push(format!("Rate limit error: {}", rate_error));
    
    let service_error = service_unavailable_error(
        "notification_service", 
        "Service is under maintenance", 
        MODULE, 
        FUNCTION
    );
    
    // recovery_hint method has been removed, adding default message
    recovery_hints.push(format!("Service error: {}", service_error));
    
    // Test non-recoverable errors
    let not_found = not_found_error("User", "user123", MODULE, FUNCTION);
    
    // recovery_hint method has been removed, adding default message
    recovery_hints.push(format!("Not found error: {}", not_found));
    
    recovery_hints
}

/// Test function to verify error context information
pub fn test_error_context() -> Vec<String> {
    const MODULE: &str = "utils::error_test";
    const FUNCTION: &str = "test_error_context";
    
    let mut context_info = Vec::new();
    
    // Create an error with detailed context
    // with_timestamp and with_operation methods have been removed
    let error = data_inconsistency_error(
        "User", 
        "user789", 
        "User exists but profile is missing", 
        MODULE, 
        FUNCTION
    );
    
    // Extract context information
    if let SquareError::Enhanced(enhanced) = &error {
        context_info.push(format!("Error code: {}", enhanced.code as u32));
        context_info.push(format!("Error message: {}", enhanced.message));
        context_info.push(format!("Module: {}", enhanced.context.module));
        context_info.push(format!("Function: {}", enhanced.context.function));
        context_info.push(format!("Severity: {:?}", enhanced.context.severity));
        
        if let Some(details) = &enhanced.context.details {
            context_info.push(format!("Details: {}", details));
        }
        
        if let Some(entity_id) = &enhanced.context.entity_id {
            context_info.push(format!("Entity ID: {}", entity_id));
        }
        
        // timestamp field type has changed from Option<u64> to u64
        let timestamp = enhanced.context.timestamp;
        context_info.push(format!("Timestamp: {}", timestamp));
    }
    
    context_info
}

/// Test function to verify error logging and monitoring
pub fn test_error_monitoring() -> SquareResult<()> {
    const MODULE: &str = "utils::error_test";
    const FUNCTION: &str = "test_error_monitoring";
    
    // Generate a test error
    let error = validation_error("Test monitoring error", MODULE, FUNCTION);
    
    // Record the error
    error_monitor::record_error(&error);
    
    // Get error stats
    let stats = error_monitor::get_error_stats();
    
    // Verify that the error was recorded
    assert!(stats.total_errors > 0, "Error was not recorded");
    
    Ok(())
}
