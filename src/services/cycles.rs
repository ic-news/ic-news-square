use candid::Principal;
use ic_cdk::api::{canister_balance, time};
use std::cell::RefCell;
use std::collections::VecDeque;

use crate::models::cycles::*;
use crate::models::error::SquareResult;

use crate::auth;
use crate::utils::error_handler::*;
use crate::storage::STORAGE;

// Constants
const WARNING_THRESHOLD: u64 = 100_000_000_000;  // 100 billion cycles (0.1 ICP)
const CRITICAL_THRESHOLD: u64 = 50_000_000_000;  // 50 billion cycles (0.05 ICP)
const TRILLION: f64 = 1_000_000_000_000.0;
const MAX_HISTORY_DAYS: usize = 30;  // Keep 30 days of history
const ESTIMATED_DAILY_CONSUMPTION: u64 = 5_000_000_000;  // 5 billion cycles per day by default
const SECONDS_IN_DAY: u64 = 24 * 60 * 60; // 24 hours in seconds
const CYCLES_HISTORY_MAX_DAYS: usize = 30; // Keep 30 days of history

// Thread-local storage for cycles consumption history
thread_local! {
    static CYCLES_HISTORY: RefCell<VecDeque<DailyConsumption>> = RefCell::new(VecDeque::with_capacity(MAX_HISTORY_DAYS));
    static LAST_RECORDED_BALANCE: RefCell<u64> = RefCell::new(0);
    static CYCLES_THRESHOLD_CONFIG: RefCell<CyclesThresholdConfig> = RefCell::new(CyclesThresholdConfig {
        warning_threshold: WARNING_THRESHOLD,
        critical_threshold: CRITICAL_THRESHOLD,
        notification_enabled: true,
    });
    static CYCLES_NOTIFICATIONS: RefCell<Vec<CyclesWarningNotification>> = RefCell::new(Vec::new());
    static NOTIFICATION_SETTINGS: RefCell<bool> = RefCell::new(true);
    static LAST_NOTIFICATION_TIME: RefCell<u64> = RefCell::new(0);
    static EMERGENCY_MODE: RefCell<bool> = RefCell::new(false);
}

// Initialize cycles monitoring
pub fn init_cycles_monitoring() {
    let current_balance = canister_balance();
    LAST_RECORDED_BALANCE.with(|balance| {
        *balance.borrow_mut() = current_balance;
    });
    
    // Initialize with empty history
    CYCLES_HISTORY.with(|history| {
        history.borrow_mut().clear();
    });
}

// Record daily cycles consumption
pub fn record_cycles_consumption() {
    let current_balance = canister_balance();
    let current_time = time() / 1_000_000;
    
    LAST_RECORDED_BALANCE.with(|last_balance| {
        let last = *last_balance.borrow();
        
        // Only record if balance decreased (consumption occurred)
        if last > current_balance {
            let consumption = last - current_balance;
            
            CYCLES_HISTORY.with(|history| {
                let mut history_mut = history.borrow_mut();
                
                // Add new consumption record
                let daily_consumption = DailyConsumption {
                    date: current_time,
                    consumption,
                    operations: 1, // Increment for each recording period
                };
                
                // Add to history and maintain max size
                history_mut.push_back(daily_consumption);
                if history_mut.len() > MAX_HISTORY_DAYS {
                    history_mut.pop_front();
                }
            });
        }
        
        // Update last recorded balance
        *last_balance.borrow_mut() = current_balance;
    });
    
    // Check if balance is below threshold and notify if needed
    check_balance_threshold();
}

// Get current cycles balance
pub fn get_cycles_balance() -> SquareResult<CyclesBalanceResponse> {
    const MODULE: &str = "services::cycles";
    const FUNCTION: &str = "get_cycles_balance";
    
    
    let current_balance = canister_balance();
    let balance_in_trillion = current_balance as f64 / TRILLION;
    
    // Calculate estimated days remaining based on average consumption
    let average_daily_consumption = get_average_daily_consumption();
    let estimated_days = if average_daily_consumption > 0 {
        current_balance / average_daily_consumption
    } else {
        current_balance / ESTIMATED_DAILY_CONSUMPTION
    };
    
    // Check if balance is below warning threshold
    let threshold_warning = CYCLES_THRESHOLD_CONFIG.with(|config| {
        let config_ref = config.borrow();
        current_balance < config_ref.warning_threshold
    });
    
    // Log low balance warning if necessary
    
    Ok(CyclesBalanceResponse {
        balance: current_balance,
        balance_in_trillion,
        estimated_days_remaining: estimated_days,
        threshold_warning,
    })
}

// Get cycles consumption history
pub fn get_cycles_consumption_history() -> SquareResult<CyclesConsumptionResponse> {
    const MODULE: &str = "services::cycles";
    const FUNCTION: &str = "get_cycles_consumption_history";
    
    
    let mut daily_consumption = Vec::new();
    let mut total_consumed = 0u64;
    let mut total_days = 0usize;
    
    CYCLES_HISTORY.with(|history| {
        let history_ref = history.borrow();
        
        // Get last 7 days for weekly consumption
        let week_history: Vec<&DailyConsumption> = history_ref.iter()
            .rev()
            .take(7)
            .collect();
            
        // Calculate total consumed in the last week
        for day in &week_history {
            total_consumed += day.consumption;
        }
        
        // Convert all history to vector
        daily_consumption = history_ref.iter().cloned().collect();
        total_days = if history_ref.len() > 0 { history_ref.len() } else { 1 };
    });
    
    // Calculate average daily consumption
    let average_daily_consumption = if total_days > 0 {
        total_consumed / total_days as u64
    } else {
        ESTIMATED_DAILY_CONSUMPTION
    };
    
    Ok(CyclesConsumptionResponse {
        daily_consumption,
        average_daily_consumption,
        total_consumed_last_week: total_consumed,
    })
}

// Update cycles threshold configuration
pub fn update_cycles_threshold(request: UpdateCyclesThresholdRequest, _caller: Principal) -> SquareResult<()> {
    const MODULE: &str = "services::cycles";
    const FUNCTION: &str = "update_cycles_threshold";
    
    
    // Only admin can update threshold configuration
    match auth::is_admin() {
        Err(msg) => return log_and_return(unauthorized_error(
            &msg,
            MODULE,
            FUNCTION
        )),
        Ok(_) => {
            
        }
    }
    
    CYCLES_THRESHOLD_CONFIG.with(|config| {
        let mut config_mut = config.borrow_mut();
        
        if let Some(warning) = request.warning_threshold {
            config_mut.warning_threshold = warning;
        }
        
        if let Some(critical) = request.critical_threshold {
            config_mut.critical_threshold = critical;
        }
        
        if let Some(enabled) = request.notification_enabled {
            config_mut.notification_enabled = enabled;
        }
    });
    
    Ok(())
}

// Get current cycles threshold configuration
pub fn get_cycles_threshold() -> SquareResult<CyclesThresholdConfig> {
    const MODULE: &str = "services::cycles";
    const FUNCTION: &str = "get_cycles_threshold";
    
    CYCLES_THRESHOLD_CONFIG.with(|config| {
        Ok(config.borrow().clone())
    })
}

// Get all cycles notifications
pub fn get_cycles_notifications() -> SquareResult<CyclesNotificationsResponse> {
    const MODULE: &str = "services::cycles";
    const FUNCTION: &str = "get_cycles_notifications";
    
    let mut notifications = Vec::new();
    let mut unacknowledged_count = 0;
    
    CYCLES_NOTIFICATIONS.with(|notifs| {
        let notifs_ref = notifs.borrow();
        notifications = notifs_ref.clone();
        
        // Count unacknowledged notifications
        unacknowledged_count = notifs_ref.iter()
            .filter(|n| !n.is_acknowledged)
            .count();
    });
    
    Ok(CyclesNotificationsResponse {
        notifications,
        unacknowledged_count,
    })
}

// Acknowledge a notification
pub fn acknowledge_notification(timestamp: u64, _caller: Principal) -> SquareResult<()> {
    const MODULE: &str = "services::cycles";
    const FUNCTION: &str = "acknowledge_notification";
    
    // Only admin can acknowledge notifications
    match auth::is_admin() {
        Err(msg) => return log_and_return(unauthorized_error(
            &msg,
            MODULE,
            FUNCTION
        )),
        Ok(_) => {}
    }
    
    CYCLES_NOTIFICATIONS.with(|notifs| {
        let mut notifs_mut = notifs.borrow_mut();
        
        // Find the notification with the given timestamp
        for notification in notifs_mut.iter_mut() {
            if notification.timestamp == timestamp {
                notification.is_acknowledged = true;
                break;
            }
        }
    });
    
    Ok(())
}

// Update notification settings
pub fn update_notification_settings(enabled: Option<bool>, _caller: Principal) -> SquareResult<()> {
    const MODULE: &str = "services::cycles";
    const FUNCTION: &str = "update_notification_settings";
    
    // Only admin can update notification settings
    match auth::is_admin() {
        Err(msg) => return log_and_return(unauthorized_error(
            &msg,
            MODULE,
            FUNCTION
        )),
        Ok(_) => {}
    }
    
    if let Some(notification_enabled) = enabled {
        NOTIFICATION_SETTINGS.with(|settings| {
            *settings.borrow_mut() = notification_enabled;
        });
    }
    
    Ok(())
}

// Get current notification settings
pub fn get_notification_settings(_caller: Principal) -> SquareResult<bool> {
    const MODULE: &str = "services::cycles";
    const FUNCTION: &str = "get_notification_settings";
    
    // Only admin can view notification settings
    match auth::is_admin() {
        Err(msg) => return log_and_return(unauthorized_error(
            &msg,
            MODULE,
            FUNCTION
        )),
        Ok(_) => {}
    }
    
    NOTIFICATION_SETTINGS.with(|settings| {
        Ok(*settings.borrow())
    })
}

// Private helper functions

// Check if balance is below threshold and handle accordingly
fn check_balance_threshold() {
    let current_balance = canister_balance();
    
    let notifications_enabled = NOTIFICATION_SETTINGS.with(|settings| {
        *settings.borrow()
    });
    
    if !notifications_enabled {
        return;
    }
    
    CYCLES_THRESHOLD_CONFIG.with(|config| {
        let config_ref = config.borrow();
        
        // Check if we should create a notification based on thresholds
        if current_balance < config_ref.critical_threshold {
            // Create a critical notification
            create_warning_notification(
                current_balance,
                config_ref.critical_threshold,
                CyclesWarningSeverity::Critical,
                format!("CRITICAL: Cycles balance is critically low ({}). Immediate action required!", current_balance)
            );
            
            // Implement additional emergency actions
            // For example, disable non-essential operations to save cycles
            emergency_cycles_conservation();
            
        } else if current_balance < config_ref.warning_threshold {
            
            // Create a warning notification
            create_warning_notification(
                current_balance,
                config_ref.warning_threshold,
                CyclesWarningSeverity::Warning,
                format!("WARNING: Cycles balance is running low ({}). Consider topping up soon.", current_balance)
            );
        }
        
        // Check if we should send a Bark notification for critical balance
        if current_balance <= config_ref.critical_threshold {
            // Use the existing Bark notification function
            ic_cdk::spawn(async {
                match send_bark_notification().await {
                    Ok(_) => ic_cdk::println!("Successfully sent Bark notification for critical cycles balance"),
                    Err(e) => ic_cdk::println!("Failed to send Bark notification: {}", e)
                }
            });
        }
    });
}

// Create a warning notification
fn create_warning_notification(balance: u64, threshold: u64, severity: CyclesWarningSeverity, message: String) {
    let notification = CyclesWarningNotification {
        timestamp: time() / 1_000_000,
        balance,
        threshold,
        severity: severity.clone(),
        message,
        is_acknowledged: false,
    };
    
    CYCLES_NOTIFICATIONS.with(|notifications| {
        let mut notifications_mut = notifications.borrow_mut();
        
        // Check if we already have a similar unacknowledged notification
        let has_similar = notifications_mut.iter().any(|n| {
            !n.is_acknowledged && n.severity == severity
        });
        
        // Only add if we don't have a similar unacknowledged notification
        if !has_similar {
            notifications_mut.push(notification);
            
            // Keep only the last 50 notifications
            if notifications_mut.len() > 50 {
                notifications_mut.remove(0);
            }
        }
    });
}



use ic_cdk::api::management_canister::http_request::{HttpMethod, TransformArgs, TransformContext, http_request, CanisterHttpRequestArgument, HttpResponse};

// Send notification using Bark service
fn send_bark_message(message: &str) {
    // Log the notification message
    ic_cdk::println!("Sending notification: {}", message);
    
    // Create a custom notification with the provided message
    let custom_message = message.to_string();
    
    // Use spawn to run the async function
    ic_cdk::spawn(async move {
        // Call the Bark notification service with the custom message
        match send_custom_bark_notification(&custom_message).await {
            Ok(_) => {
                ic_cdk::println!("Successfully sent notification via Bark");
            }
            Err(e) => {
                ic_cdk::println!("Failed to send notification via Bark: {}", e);
            }
        }
    });
}

async fn send_bark_notification() -> Result<(), String> {
    // Get Bark API key from storage
    let api_key = crate::storage::STORAGE.with(|storage| {
        let storage = storage.borrow();
        storage.bark_api_key.clone()
    });
    
    // Check if API key is set
    if api_key.is_empty() {
        return Err("Bark API key is not configured".to_string());
    }
    
    let url = format!("https://api.day.app/{}", api_key);
    
    // Get current balance for notification content
    let current_balance = canister_balance();
    let balance_in_trillion = current_balance as f64 / TRILLION;
    
    // Build the request parameters for Bark API
    // Documentation: https://github.com/Finb/Bark/blob/master/README.md
    let title = "Critical: Cycles Balance Warning";
    let body = format!("Cycles balance is critically low: {} ({:.6} T). Please top up soon to avoid service interruption.", 
                      current_balance, balance_in_trillion);
    
    // URL encode the parameters
    let encoded_title = url_encode(title);
    let encoded_body = url_encode(&body);
    
    // Construct the full URL with parameters
    let full_url = format!("{}/{}?body={}&group=IC-News-Square&isArchive=1&sound=alarm", 
                          url, encoded_title, encoded_body);
    
    // Build HTTP request
    let request = CanisterHttpRequestArgument {
        url: full_url,
        method: HttpMethod::GET,
        body: None,
        max_response_bytes: Some(1024),
        transform: Some(TransformContext::from_name(
            "transform_bark_response".to_string(),
            vec![]
        )),
        headers: vec![],
    };
    
    // Send HTTP request
    match http_request(request, 0).await {
        Ok((response,)) => {
            // Check if status code is in 200-299 range
            // Since candid::Nat cannot be directly compared with integers, we convert it to string
            let status_str = response.status.to_string();
            let status_code = status_str.parse::<u16>().unwrap_or(0);
            if status_code >= 200 && status_code < 300 {
                Ok(())
            } else {
                Err(format!(
                    "Bark API returned error status: {}. Body: {}",
                    response.status,
                    String::from_utf8_lossy(&response.body)
                ))
            }
        }
        Err((code, message)) => {
            Err(format!("HTTP request failed with code {:?} and message: {}", code, message))
        }
    }
}

// Transform function for the Bark HTTP response
fn transform_bark_response(args: TransformArgs) -> HttpResponse {
    HttpResponse {
        status: args.response.status,
        headers: vec![],
        body: args.response.body,
    }
}

// Send a custom notification message using Bark service
async fn send_custom_bark_notification(message: &str) -> Result<(), String> {
    // Bark API configuration
    let api_key = crate::storage::STORAGE.with(|storage| {
        let storage = storage.borrow();
        storage.bark_api_key.clone()
    });
    let url = format!("https://api.day.app/{}", api_key);
    
    // Build the request parameters for Bark API
    let title = "IC News Square Notification";
    let body = message;
    
    // URL encode the parameters
    let encoded_title = url_encode(title);
    let encoded_body = url_encode(body);
    
    // Construct the full URL with parameters
    let full_url = format!("{}/{}?body={}&group=IC-News-Square&isArchive=1", 
                          url, encoded_title, encoded_body);
    
    // Build HTTP request
    let request = CanisterHttpRequestArgument {
        url: full_url,
        method: HttpMethod::GET,
        body: None,
        max_response_bytes: Some(1024),
        transform: Some(TransformContext::from_name(
            "transform_bark_response".to_string(),
            vec![]
        )),
        headers: vec![],
    };
    
    // Send HTTP request
    match http_request(request, 0).await {
        Ok((response,)) => {
            // Check if status code is in 200-299 range
            let status_str = response.status.to_string();
            let status_code = status_str.parse::<u16>().unwrap_or(0);
            if status_code >= 200 && status_code < 300 {
                Ok(())
            } else {
                Err(format!(
                    "Bark API returned error status: {}. Body: {}",
                    response.status,
                    String::from_utf8_lossy(&response.body)
                ))
            }
        }
        Err((code, message)) => {
            Err(format!("HTTP request failed with code {:?} and message: {}", code, message))
        }
    }
}

// Simple URL encoding function
fn url_encode(s: &str) -> String {
    let mut result = String::new();
    for c in s.chars() {
        match c {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' | '.' | '~' => result.push(c),
            ' ' => result.push('+'),
            _ => {
                let bytes = c.to_string().into_bytes();
                for b in bytes {
                    result.push_str(&format!("%{:02X}", b));
                }
            }
        }
    }
    result
}

// Emergency cycles conservation measures
fn emergency_cycles_conservation() {
    // This function implements emergency measures to conserve cycles
    
    // Log the emergency measures
    let current_balance = canister_balance();
    let timestamp = time() / 1_000_000; // Convert to seconds
    
    ic_cdk::println!(
        "[EMERGENCY] Activating cycles conservation measures at timestamp: {}, current balance: {}", 
        timestamp, 
        current_balance
    );
    
    // 1. Reduce heartbeat interval to save cycles
    STORAGE.with(|storage| {
        let mut store = storage.borrow_mut();
        // Double the interval to reduce frequency
        store.heartbeat_interval_hours = store.heartbeat_interval_hours.saturating_mul(2);
        ic_cdk::println!("[EMERGENCY] Heartbeat interval increased to {} hours", store.heartbeat_interval_hours);
    });
    
    // 2. Disable trending content updates which consume cycles
    crate::storage::STORAGE.with(|storage| {
        let mut storage = storage.borrow_mut();
        // Clear trending content to avoid processing
        storage.trending_content.clear();
        ic_cdk::println!("[EMERGENCY] Cleared trending content cache to save processing cycles");
    });
    
    // 3. Set a flag to limit certain operations in other parts of the code
    // We can use a global static to indicate emergency mode
    EMERGENCY_MODE.with(|mode| {
        *mode.borrow_mut() = true;
        ic_cdk::println!("[EMERGENCY] Emergency mode activated - non-essential features disabled");
    });
    
    // 4. Update cycles threshold to be more conservative
    STORAGE.with(|storage| {
        let mut store = storage.borrow_mut();
        // Increase heartbeat interval to reduce canister activity
        if store.heartbeat_interval_hours < 24 {
            store.heartbeat_interval_hours = store.heartbeat_interval_hours.saturating_mul(2);
            ic_cdk::println!("[EMERGENCY] Heartbeat interval increased to {} hours", store.heartbeat_interval_hours);
        }
    });
    
    // Notify administrators
    let message = format!("Emergency cycles conservation measures activated: increased heartbeat interval, disabled trending updates, and limited non-essential features. Current balance: {}", current_balance);
    send_bark_message(&message);
}

// Get average daily consumption
fn get_average_daily_consumption() -> u64 {
    let mut total_consumption = 0u64;
    let mut days = 0usize;
    
    CYCLES_HISTORY.with(|history| {
        let history_ref = history.borrow();
        for day in history_ref.iter() {
            total_consumption += day.consumption;
            days += 1;
        }
    });
    
    if days > 0 {
        total_consumption / days as u64
    } else {
        ESTIMATED_DAILY_CONSUMPTION
    }
}
