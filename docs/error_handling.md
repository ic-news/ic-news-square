# IC News Square Error Handling Guide

This document provides guidelines and best practices for error handling in the IC News Square project.

## Table of Contents

1. [Introduction](#introduction)
2. [Error Model](#error-model)
3. [Error Handling Utilities](#error-handling-utilities)
4. [Middleware](#middleware)
5. [Error Monitoring](#error-monitoring)
6. [Best Practices](#best-practices)
7. [Testing](#testing)

## Introduction

Effective error handling is critical for building reliable, maintainable, and user-friendly applications. The IC News Square project implements a comprehensive error handling system that provides:

- Standardized error types and codes
- Contextual error information
- Error logging and monitoring
- Recovery hints for recoverable errors
- Middleware for consistent API responses

## Error Model

The error model is defined in `src/models/error.rs` and consists of the following components:

### Error Codes

Error codes are defined in the `ErrorCode` enum and categorize errors by their type:

```rust
pub enum ErrorCode {
    NotFound,
    AlreadyExists,
    Unauthorized,
    ValidationFailed,
    ContentTooLong,
    InvalidOperation,
    SystemError,
    DataInconsistency,
    ResourceUnavailable,
    RateLimitExceeded,
    DependencyFailed,
    PermissionDenied,
    ServiceUnavailable,
    QuotaExceeded,
    UnexpectedError,
}
```

### Error Severity

Error severity levels help categorize errors by their impact:

```rust
pub enum ErrorSeverity {
    Info,
    Warning,
    Error,
    Critical,
}
```

### Error Context

The `ErrorContext` struct provides detailed information about where and when an error occurred:

```rust
pub struct ErrorContext {
    pub module: String,
    pub function: String,
    pub severity: ErrorSeverity,
    pub timestamp: Option<u64>,
    pub details: Option<String>,
    pub entity_id: Option<String>,
    pub operation: Option<String>,
}
```

### Enhanced Error

The `EnhancedError` struct combines error code, message, context, and recovery information:

```rust
pub struct EnhancedError {
    pub code: ErrorCode,
    pub message: String,
    pub context: ErrorContext,
    pub recoverable: bool,
    pub recovery_hint: Option<String>,
}
```

## Error Handling Utilities

The `src/utils/error_handler.rs` module provides utility functions for creating standardized errors:

### Common Error Creation Functions

- `not_found_error(entity_type, entity_id, module, function)`
- `already_exists_error(entity_type, entity_id, module, function)`
- `unauthorized_error(reason, module, function)`
- `validation_error(message, module, function)`
- `content_too_long_error(content_type, max_length, actual_length, module, function)`
- `invalid_operation_error(operation, reason, module, function)`
- `system_error(message, module, function)`
- `data_inconsistency_error(entity_type, entity_id, details, module, function)`
- `resource_unavailable_error(resource_type, resource_id, module, function)`
- `rate_limit_error(operation, limit, module, function)`
- `dependency_error(dependency, details, module, function)`
- `permission_denied_error(operation, reason, module, function)`
- `service_unavailable_error(service, details, module, function)`
- `quota_exceeded_error(resource_type, limit, module, function)`

### Helper Functions

- `log_and_return(error)`: Logs an error and returns it
- `try_with_logging(operation, module, function)`: Executes an operation and logs any error

## Middleware

The `src/utils/middleware.rs` module provides middleware for handling errors in API endpoints:

### API Response Wrapper

```rust
pub struct ApiResponse<T: CandidType> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<ApiError>,
}
```

### Error Handling Middleware

The `with_error_handling` function wraps API handlers with error handling:

```rust
pub fn with_error_handling<T: CandidType, F>(handler: F) -> impl FnOnce() -> ApiResponse<T>
where
    F: FnOnce() -> SquareResult<T>
```

### Usage Example

```rust
#[update]
fn create_post(request: CreatePostRequest) -> ApiResponse<PostResponse> {
    with_error_handling(|| services::content::create_post(request, caller()))()
}
```

## Error Monitoring

The `src/utils/error_monitor.rs` module provides error monitoring capabilities:

### Monitoring Functions

- `record_error(error)`: Records an error in the monitoring system
- `get_error_history()`: Gets the error history
- `get_error_stats()`: Gets error statistics
- `get_most_common_errors(limit)`: Gets the most common errors

### Admin API Endpoints

```rust
#[query(guard = "auth::is_admin")]
fn get_error_history() -> ApiResponse<Vec<String>>

#[query(guard = "auth::is_admin")]
fn get_error_stats() -> ApiResponse<Vec<(ErrorCode, u64, u64, u64)>>

#[query(guard = "auth::is_admin")]
fn get_most_common_errors(limit: usize) -> ApiResponse<Vec<(ErrorCode, u64)>>
```

## Best Practices

### 1. Use Specific Error Types

Always use the most specific error type for the situation:

```rust
// Good
return Err(not_found_error("User", &id, MODULE, FUNCTION));

// Bad
return Err(SquareError::NotFound(format!("User not found: {}", id)));
```

### 2. Include Context Information

Always include module and function information:

```rust
const MODULE: &str = "services::user";
const FUNCTION: &str = "get_user_profile";

// Later in the code
return Err(not_found_error("User", &id, MODULE, FUNCTION));
```

### 3. Add Detailed Error Messages

Provide clear, actionable error messages:

```rust
// Good
return Err(validation_error("Username must be between 3 and 20 characters", MODULE, FUNCTION));

// Bad
return Err(validation_error("Invalid username", MODULE, FUNCTION));
```

### 4. Use Recovery Hints for Recoverable Errors

For errors that users can recover from, provide hints:

```rust
return Err(rate_limit_error("create_post", 10, MODULE, FUNCTION)
    .recoverable("Try again in 5 minutes"));
```

### 5. Log Errors Appropriately

Use the `log_and_return` helper for consistent logging:

```rust
if !is_valid_input(&request) {
    return log_and_return(validation_error("Invalid input", MODULE, FUNCTION));
}
```

### 6. Use Try With Logging

Wrap operations that might fail with `try_with_logging`:

```rust
try_with_logging(|| {
    // Operation that might fail
    storage.update_user(user)
}, MODULE, FUNCTION)
```

## Testing

The `src/utils/error_test.rs` module provides functions for testing the error handling system:

### Test Functions

- `generate_test_errors()`: Generates various types of errors
- `test_error_recovery_hints()`: Tests error recovery hints
- `test_error_context()`: Tests error context information
- `test_error_monitoring()`: Tests error monitoring

### Admin API Endpoints for Testing

```rust
#[update(guard = "auth::is_admin")]
fn test_error_handling() -> ApiResponse<()>

#[query(guard = "auth::is_admin")]
fn test_error_recovery_hints() -> ApiResponse<Vec<String>>

#[query(guard = "auth::is_admin")]
fn test_error_context() -> ApiResponse<Vec<String>>
```

## Conclusion

By following these guidelines and using the provided error handling utilities, you can ensure consistent, informative, and actionable error handling throughout the IC News Square project.
