# IC News Square Error Handling Examples

This document provides practical examples of how to use the error handling system in the IC News Square project.

## Basic Error Handling

### Example 1: Handling Not Found Errors

```rust
pub fn get_user_profile(user_id: String) -> SquareResult<UserProfileResponse> {
    const MODULE: &str = "services::user";
    const FUNCTION: &str = "get_user_profile";
    
    STORAGE.with(|storage| {
        let storage = storage.borrow();
        
        // Use the not_found_error utility function
        let user = storage.users.get(&user_id)
            .ok_or_else(|| not_found_error("User", &user_id, MODULE, FUNCTION))?;
            
        // Rest of the function...
        Ok(UserProfileResponse { /* ... */ })
    })
}
```

### Example 2: Validation Errors

```rust
pub fn create_comment(request: CreateCommentRequest, caller: Principal) -> SquareResult<CommentResponse> {
    const MODULE: &str = "services::content";
    const FUNCTION: &str = "create_comment";
    
    // Validate content length
    if request.content.len() > MAX_COMMENT_LENGTH {
        return log_and_return(content_too_long_error(
            "Comment", 
            MAX_COMMENT_LENGTH, 
            request.content.len(), 
            MODULE, 
            FUNCTION
        ));
    }
    
    // Rest of the function...
}
```

### Example 3: Authorization Errors

```rust
pub fn update_user_role(user_id: String, role: UserRole, caller: Principal) -> SquareResult<()> {
    const MODULE: &str = "services::user";
    const FUNCTION: &str = "update_user_role";
    
    // Check if caller is admin
    if !is_admin(caller) {
        return log_and_return(unauthorized_error(
            "Only administrators can update user roles",
            MODULE,
            FUNCTION
        ));
    }
    
    // Rest of the function...
    Ok(())
}
```

## Advanced Error Handling

### Example 4: Adding Context to Errors

```rust
pub fn process_transaction(transaction_id: String) -> SquareResult<TransactionResponse> {
    const MODULE: &str = "services::payment";
    const FUNCTION: &str = "process_transaction";
    
    // Create a base error
    let base_error = not_found_error("Transaction", &transaction_id, MODULE, FUNCTION);
    
    // Add additional context
    let enhanced_error = base_error
        .with_details("Transaction may have expired or been cancelled")
        .with_timestamp(time())
        .with_operation("payment_processing");
        
    // Log and return the enhanced error
    log_and_return(enhanced_error)
}
```

### Example 5: Recoverable Errors

```rust
pub fn send_notification(user_id: String, message: String) -> SquareResult<()> {
    const MODULE: &str = "services::notification";
    const FUNCTION: &str = "send_notification";
    
    // Check if notification service is available
    if !is_notification_service_available() {
        return log_and_return(service_unavailable_error(
            "NotificationService",
            "Service is currently under maintenance",
            MODULE,
            FUNCTION
        ).recoverable("Notifications will be delivered when the service is back online"));
    }
    
    // Rest of the function...
    Ok(())
}
```

### Example 6: Using Try With Logging

```rust
pub fn update_user_stats(user_id: String) -> SquareResult<UserStatsResponse> {
    const MODULE: &str = "services::user";
    const FUNCTION: &str = "update_user_stats";
    
    // Use try_with_logging to handle potential errors
    try_with_logging(|| {
        // Operation that might fail
        let result = calculate_user_stats(user_id)?;
        
        // Update storage with new stats
        STORAGE.with(|storage| {
            let mut storage = storage.borrow_mut();
            storage.user_stats.insert(user_id.clone(), result.clone());
            Ok(result)
        })
    }, MODULE, FUNCTION)
}
```

## Error Handling in API Endpoints

### Example 7: Using Middleware for API Endpoints

```rust
// In lib.rs
#[update]
fn create_post(request: CreatePostRequest) -> ApiResponse<PostResponse> {
    with_error_handling(|| services::content::create_post(request, caller()))()
}
```

### Example 8: Handling Inter-Canister Calls

```rust
pub async fn fetch_external_data(canister_id: Principal) -> SquareResult<ExternalData> {
    const MODULE: &str = "services::external";
    const FUNCTION: &str = "fetch_external_data";
    
    // Make inter-canister call
    let result: Result<ExternalData, (RejectionCode, String)> = 
        ic_cdk::call(canister_id, "get_data", ()).await;
        
    // Handle potential errors
    handle_canister_error(result, MODULE, FUNCTION)
}
```

### Example 9: Using Retry Logic

```rust
pub async fn process_payment(payment_id: String) -> SquareResult<PaymentResult> {
    const MODULE: &str = "services::payment";
    const FUNCTION: &str = "process_payment";
    
    // Try the operation with retries
    try_with_retry(
        || async {
            // Payment processing logic that might fail
            let result = call_payment_service(payment_id.clone()).await?;
            Ok(result)
        },
        3, // Number of retries
        MODULE,
        FUNCTION
    ).await
}
```

## Error Monitoring and Testing

### Example 10: Testing Error Handling

```rust
// In a test module
#[test]
fn test_error_handling() {
    // Generate test errors
    utils::error_test::generate_test_errors().unwrap();
    
    // Check error history
    let history = utils::error_monitor::get_error_history();
    assert!(!history.is_empty());
    
    // Check most common errors
    let common_errors = utils::error_monitor::get_most_common_errors(5);
    assert!(!common_errors.is_empty());
}
```

## Best Practices Summary

1. **Always use utility functions** for creating errors
2. **Include module and function constants** at the top of each function
3. **Log errors** using `log_and_return`
4. **Add context** to errors when possible
5. **Provide recovery hints** for recoverable errors
6. **Use middleware** for API endpoints
7. **Handle inter-canister call errors** properly
8. **Use retry logic** for operations that might fail temporarily
9. **Test error handling** thoroughly

By following these examples and best practices, you can ensure consistent, informative, and actionable error handling throughout the IC News Square project.
