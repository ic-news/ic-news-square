use candid::Principal;
use ic_cdk::api::time;

use crate::models::error::{SquareError, SquareResult, ErrorCode, ErrorSeverity};
use crate::utils::error_handler::*;
use crate::utils::error_monitor;
use crate::utils::middleware::{ApiResponse, with_error_handling};

// Error handling unit tests
#[test]
fn test_error_creation() {
    // Test not found error
    let not_found = not_found_error("User", "user123", "test_module", "test_function");
    assert_eq!(not_found.code(), ErrorCode::NotFound);
    
    // Test already exists error
    let already_exists = already_exists_error("Post", "post456", "test_module", "test_function");
    assert_eq!(already_exists.code(), ErrorCode::AlreadyExists);
    
    // Test unauthorized error
    let unauthorized = unauthorized_error("Missing permissions", "test_module", "test_function");
    assert_eq!(unauthorized.code(), ErrorCode::Unauthorized);
    
    // Test validation error
    let validation = validation_error("Title cannot be empty", "test_module", "test_function");
    assert_eq!(validation.code(), ErrorCode::ValidationFailed);
    
    // Test content too long error
    let content_too_long = content_too_long_error("Comment", 500, 750, "test_module", "test_function");
    assert_eq!(content_too_long.code(), ErrorCode::ContentTooLong);
    
    // Test invalid operation error
    let invalid_operation = invalid_operation_error(
        "delete_post", 
        "Cannot delete a post with active comments", 
        "test_module", 
        "test_function"
    );
    assert_eq!(invalid_operation.code(), ErrorCode::InvalidOperation);
    
    // Test system error
    let system = system_error("Database connection failed", "test_module", "test_function");
    assert_eq!(system.code(), ErrorCode::SystemError);
    
    // Test data inconsistency error
    let data_inconsistency = data_inconsistency_error(
        "User", 
        "user789", 
        "User exists but profile is missing", 
        "test_module", 
        "test_function"
    );
    assert_eq!(data_inconsistency.code(), ErrorCode::DataInconsistency);
}

#[test]
fn test_error_context() {
    // Create an error with context
    let error = not_found_error("User", "user123", "test_module", "test_function")
        .with_timestamp(time())
        .with_operation("get_user")
        .with_details("User might have been deleted");
    
    if let SquareError::Enhanced(enhanced) = error {
        assert_eq!(enhanced.context.module, "test_module");
        assert_eq!(enhanced.context.function, "test_function");
        assert_eq!(enhanced.context.severity, ErrorSeverity::Error);
        assert!(enhanced.context.timestamp.is_some());
        assert_eq!(enhanced.context.operation, Some("get_user".to_string()));
        assert_eq!(enhanced.context.details, Some("User might have been deleted".to_string()));
        assert_eq!(enhanced.context.entity_id, Some("user123".to_string()));
    } else {
        panic!("Expected Enhanced error");
    }
}

#[test]
fn test_error_recovery() {
    // Create a recoverable error
    let error = resource_unavailable_error(
        "Database", 
        "primary", 
        "test_module", 
        "test_function"
    );
    
    assert!(error.recoverable());
    assert!(error.recovery_hint().is_some());
    assert_eq!(error.recovery_hint().unwrap(), "Try again later");
    
    // Create a non-recoverable error
    let error = not_found_error("User", "user123", "test_module", "test_function");
    assert!(!error.recoverable());
    assert!(error.recovery_hint().is_none());
    
    // Make a non-recoverable error recoverable
    let error = error.recoverable("Check if the user ID is correct");
    assert!(error.recoverable());
    assert_eq!(error.recovery_hint().unwrap(), "Check if the user ID is correct");
}

#[test]
fn test_try_with_logging() {
    // Test successful operation
    let result: SquareResult<i32> = try_with_logging(|| Ok(42), "test_module", "test_function");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 42);
    
    // Test failed operation
    let result: SquareResult<i32> = try_with_logging(
        || Err(SquareError::NotFound("Not found".to_string())), 
        "test_module", 
        "test_function"
    );
    assert!(result.is_err());
    
    // The error should be enhanced with context
    if let Err(SquareError::Enhanced(enhanced)) = result {
        assert_eq!(enhanced.context.module, "test_module");
        assert_eq!(enhanced.context.function, "test_function");
    } else {
        panic!("Expected Enhanced error");
    }
}

#[test]
fn test_middleware() {
    // Test successful operation
    let handler = || -> SquareResult<i32> { Ok(42) };
    let middleware = with_error_handling(handler);
    let response: ApiResponse<i32> = middleware();
    
    assert!(response.success);
    assert_eq!(response.data, Some(42));
    assert!(response.error.is_none());
    
    // Test failed operation
    let handler = || -> SquareResult<i32> { 
        Err(not_found_error("User", "user123", "test_module", "test_function")) 
    };
    let middleware = with_error_handling(handler);
    let response: ApiResponse<i32> = middleware();
    
    assert!(!response.success);
    assert!(response.data.is_none());
    assert!(response.error.is_some());
    
    let error = response.error.unwrap();
    assert_eq!(error.code, ErrorCode::NotFound as u32);
    assert!(error.message.contains("not found"));
}

#[test]
fn test_log_and_return() {
    // Create an error
    let error = not_found_error("User", "user123", "test_module", "test_function");
    
    // Log and return the error
    let result: SquareResult<i32> = log_and_return(error);
    
    // The result should be an error
    assert!(result.is_err());
}

// Test helper functions
fn create_test_error() -> SquareError {
    not_found_error("User", "user123", "test_module", "test_function")
}

fn operation_that_fails() -> SquareResult<i32> {
    Err(create_test_error())
}

fn operation_that_succeeds() -> SquareResult<i32> {
    Ok(42)
}
