use candid::CandidType;
use ic_cdk::api::call::RejectionCode;
use ic_cdk::trap;

use crate::models::error::{SquareError, SquareResult, ErrorCode, ErrorSeverity};
use crate::utils::error_monitor;

/// Response wrapper for API endpoints
#[derive(CandidType, Clone)]
pub struct ApiResponse<T: CandidType> {
    /// Whether the operation was successful
    pub success: bool,
    /// The response data (if successful)
    pub data: Option<T>,
    /// Error information (if unsuccessful)
    pub error: Option<ApiError>,
}

/// API error information
#[derive(CandidType, Clone)]
pub struct ApiError {
    /// Error code
    pub code: u32,
    /// Error message
    pub message: String,
    /// Error details (optional)
    pub details: Option<String>,
    /// Whether the error is recoverable
    pub recoverable: bool,
    /// Recovery hint (if recoverable)
    pub recovery_hint: Option<String>,
}

/// Convert a SquareError to an ApiError
fn to_api_error(error: &SquareError) -> ApiError {
    match error {
        SquareError::Enhanced(enhanced) => ApiError {
            code: enhanced.code as u32,
            message: enhanced.message.clone(),
            details: enhanced.context.details.clone(),
            recoverable: enhanced.recoverable,
            recovery_hint: enhanced.recovery_hint.clone(),
        },
        _ => ApiError {
            code: error.code() as u32,
            message: error.to_string(),
            details: None,
            recoverable: false,
            recovery_hint: None,
        },
    }
}

/// Wrap an API handler with error handling middleware
pub fn with_error_handling<T: CandidType, F>(handler: F) -> impl FnOnce() -> ApiResponse<T>
where
    F: FnOnce() -> SquareResult<T>,
{
    move || {
        match handler() {
            Ok(data) => ApiResponse {
                success: true,
                data: Some(data),
                error: None,
            },
            Err(error) => {
                // Log the error
                error.log();
                
                // Record the error in the monitoring system
                error_monitor::record_error(&error);
                
                // Return a formatted error response
                ApiResponse {
                    success: false,
                    data: None,
                    error: Some(to_api_error(&error)),
                }
            }
        }
    }
}

/// Handle inter-canister call errors
pub fn handle_canister_error<T>(
    result: Result<T, (RejectionCode, String)>,
    module: &str,
    function: &str,
) -> SquareResult<T> {
    match result {
        Ok(value) => Ok(value),
        Err((code, message)) => {
            let error_code = match code {
                RejectionCode::SysFatal => ErrorCode::SystemError,
                RejectionCode::SysTransient => ErrorCode::ServiceUnavailable,
                RejectionCode::DestinationInvalid => ErrorCode::InvalidOperation,
                RejectionCode::CanisterReject => ErrorCode::OperationFailed,
                RejectionCode::CanisterError => ErrorCode::DependencyFailed,
                RejectionCode::Unknown => ErrorCode::UnexpectedError,
                RejectionCode::NoError => ErrorCode::UnexpectedError,
            };
            
            let error = SquareError::new(
                error_code,
                format!("Inter-canister call failed: {}", message),
                module,
                function,
                ErrorSeverity::Error,
            );
            
            error.log();
            Err(error)
        }
    }
}

/// Handle critical errors by trapping with a formatted message
pub fn handle_critical_error(error: SquareError, context: &str) -> ! {
    error.log();
    trap(&format!("Critical error in {}: {}", context, error));
}

/// Try to execute an operation with retry logic
pub async fn try_with_retry<T, F, Fut>(
    operation: F,
    retries: usize,
    module: &str,
    function: &str,
) -> SquareResult<T>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = SquareResult<T>>,
{
    let mut attempts = 0;
    let mut last_error = None;
    
    while attempts <= retries {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(error) => {
                attempts += 1;
                
                last_error = Some(error);
            }
        }
    }
    
    // All retries failed
    Err(last_error.unwrap_or_else(|| {
        SquareError::new(
            ErrorCode::UnexpectedError,
            "Operation failed after retries with no specific error",
            module,
            function,
            ErrorSeverity::Error,
        )
    }))
}
