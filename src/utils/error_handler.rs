use crate::models::error::{SquareError, SquareResult, ErrorCode, ErrorSeverity};

/// Error handler utility functions
/// This module provides utility functions for standardized error handling across the application

/// Create a not found error with enhanced context
pub fn not_found_error(
    entity_type: &str,
    entity_id: &str,
    module: &str,
    function: &str
) -> SquareError {
    SquareError::new(
        ErrorCode::NotFound,
        format!("{} not found: {}", entity_type, entity_id),
        module,
        function,
        ErrorSeverity::Error
    )
    .with_entity_id(entity_id)
}

/// Create an already exists error with enhanced context
pub fn already_exists_error(
    entity_type: &str,
    entity_id: &str,
    module: &str,
    function: &str
) -> SquareError {
    SquareError::new(
        ErrorCode::AlreadyExists,
        format!("{} already exists: {}", entity_type, entity_id),
        module,
        function,
        ErrorSeverity::Error
    )
    .with_entity_id(entity_id)
}

/// Create an unauthorized error with enhanced context
pub fn unauthorized_error(
    reason: &str,
    module: &str,
    function: &str
) -> SquareError {
    SquareError::new(
        ErrorCode::Unauthorized,
        format!("Unauthorized: {}", reason),
        module,
        function,
        ErrorSeverity::Warning
    )
}

/// Create a validation error with enhanced context
pub fn validation_error(
    message: &str,
    module: &str,
    function: &str
) -> SquareError {
    SquareError::new(
        ErrorCode::ValidationFailed,
        format!("Validation failed: {}", message),
        module,
        function,
        ErrorSeverity::Warning
    )
}

/// Create a content too long error with enhanced context
pub fn content_too_long_error(
    content_type: &str,
    max_length: usize,
    actual_length: usize,
    module: &str,
    function: &str
) -> SquareError {
    SquareError::new(
        ErrorCode::ContentTooLong,
        format!("{} content too long: {} characters (max: {})", 
            content_type, actual_length, max_length),
        module,
        function,
        ErrorSeverity::Warning
    )
}

/// Create an invalid operation error with enhanced context
pub fn invalid_operation_error(
    operation: &str,
    reason: &str,
    module: &str,
    function: &str
) -> SquareError {
    SquareError::new(
        ErrorCode::InvalidOperation,
        format!("Invalid operation '{}': {}", operation, reason),
        module,
        function,
        ErrorSeverity::Error
    )
}

/// Create a system error with enhanced context
pub fn system_error(
    message: &str,
    module: &str,
    function: &str
) -> SquareError {
    SquareError::new(
        ErrorCode::SystemError,
        format!("System error: {}", message),
        module,
        function,
        ErrorSeverity::Critical
    )
}

/// Create a data inconsistency error with enhanced context
pub fn data_inconsistency_error(
    entity_type: &str,
    entity_id: &str,
    details: &str,
    module: &str,
    function: &str
) -> SquareError {
    SquareError::new(
        ErrorCode::DataInconsistency,
        format!("Data inconsistency in {}: {}", entity_type, entity_id),
        module,
        function,
        ErrorSeverity::Critical
    )
    .with_details(details)
    .with_entity_id(entity_id)
}

/// Create a resource unavailable error with enhanced context
pub fn resource_unavailable_error(
    resource_type: &str,
    resource_id: &str,
    module: &str,
    function: &str
) -> SquareError {
    SquareError::new(
        ErrorCode::ResourceUnavailable,
        format!("{} is currently unavailable: {}", resource_type, resource_id),
        module,
        function,
        ErrorSeverity::Error
    )
    .with_entity_id(resource_id)
    .recoverable("Try again later")
}

/// Return an error without logging
/// This function was previously logging errors, but logging has been disabled to save cycles
pub fn log_and_return<T>(error: SquareError) -> SquareResult<T> {
    // error.log(); // Disabled to save cycles
    Err(error)
}

/// Try to execute an operation, log any error, and return the result
pub fn try_with_logging<T, F>(operation: F, module: &str, function: &str) -> SquareResult<T>
where
    F: FnOnce() -> SquareResult<T>,
{
    match operation() {
        Ok(result) => Ok(result),
        Err(error) => {
            if let SquareError::Enhanced(_) = error {
                Err(error)
            } else {
                // Otherwise, enhance it with context but don't log
                let enhanced = SquareError::new(
                    ErrorCode::UnexpectedError,
                    format!("Operation failed: {}", error),
                    module,
                    function,
                    ErrorSeverity::Error
                );
                Err(enhanced)
            }
        }
    }
}

/// Create a rate limit exceeded error with enhanced context
pub fn rate_limit_error(
    operation: &str,
    limit: u64,
    module: &str,
    function: &str
) -> SquareError {
    SquareError::new(
        ErrorCode::RateLimitExceeded,
        format!("Rate limit exceeded for operation '{}': limit {}", operation, limit),
        module,
        function,
        ErrorSeverity::Warning
    )
    .recoverable("Try again later")
}

/// Create a dependency failed error with enhanced context
pub fn dependency_error(
    dependency: &str,
    details: &str,
    module: &str,
    function: &str
) -> SquareError {
    SquareError::new(
        ErrorCode::DependencyFailed,
        format!("Dependency '{}' failed", dependency),
        module,
        function,
        ErrorSeverity::Error
    )
    .with_details(details)
}

/// Create a permission denied error with enhanced context
pub fn permission_denied_error(
    operation: &str,
    reason: &str,
    module: &str,
    function: &str
) -> SquareError {
    SquareError::new(
        ErrorCode::PermissionDenied,
        format!("Permission denied for operation '{}': {}", operation, reason),
        module,
        function,
        ErrorSeverity::Warning
    )
}

/// Create a service unavailable error with enhanced context
pub fn service_unavailable_error(
    service: &str,
    details: &str,
    module: &str,
    function: &str
) -> SquareError {
    SquareError::new(
        ErrorCode::ServiceUnavailable,
        format!("Service '{}' is currently unavailable", service),
        module,
        function,
        ErrorSeverity::Error
    )
    .with_details(details)
    .recoverable("Try again later")
}

/// Create a quota exceeded error with enhanced context
pub fn quota_exceeded_error(
    resource_type: &str,
    limit: u64,
    module: &str,
    function: &str
) -> SquareError {
    SquareError::new(
        ErrorCode::QuotaExceeded,
        format!("Quota exceeded for resource '{}': limit {}", resource_type, limit),
        module,
        function,
        ErrorSeverity::Warning
    )
}
