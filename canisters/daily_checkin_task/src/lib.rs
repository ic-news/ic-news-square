use candid::{CandidType, Deserialize, Principal};
use ic_cdk::api::{time, caller};
use ic_cdk_macros::*;

use std::cell::RefCell;
use std::collections::HashMap;

// Import Value enum for flexible data representation
#[derive(CandidType, Deserialize, Clone, Debug)]
pub enum Value {
    Int(i64),
    Nat(u64),
    Float(f64),
    Text(String),
    Bool(bool),
    Blob(Vec<u8>),
    Array(Vec<Value>),
    Map(Vec<(String, Value)>),
    Principal(Principal),
    Null,
}

// Storage for the daily check-in task
thread_local! {
    static STORAGE: RefCell<DailyCheckInStorage> = RefCell::new(DailyCheckInStorage::default());
}

#[derive(Default, Clone, CandidType, Deserialize)]
struct DailyCheckInStorage {
    // Map of user principal to their last check-in timestamp
    user_checkins: HashMap<Principal, u64>,
    // Map of user principal to their consecutive check-in days
    consecutive_days: HashMap<Principal, u64>,
    // User points (simplified from the main system)
    user_points: HashMap<Principal, u64>,
    // Points history
    points_history: HashMap<Principal, Vec<PointsTransaction>>,
    // Task configuration
    task_config: TaskConfig,
    // Admin principals
    admins: Vec<Principal>,
}

// Points transaction record
#[derive(CandidType, Deserialize, Clone)]
struct PointsTransaction {
    pub amount: i64,
    pub reason: String,
    pub timestamp: u64,
    pub reference_id: Option<String>,
}

// Task configuration
#[derive(CandidType, Deserialize, Clone, Default)]
struct TaskConfig {
    pub title: String,
    pub description: String,
    pub base_points: u64,
    pub max_consecutive_bonus_days: u64,
    pub consecutive_days_bonus_multiplier: u64,
    pub enabled: bool,
}

// Task verification request from the main canister
#[derive(CandidType, Deserialize, Clone)]
struct TaskVerificationRequest {
    pub user: Principal,
    pub task_id: String,
    pub timestamp: u64,
    pub proof: Option<String>,
}

// Verification data to return to the main canister
#[derive(CandidType, Deserialize, Clone)]
struct VerificationData {
    pub task_id: String,
    pub points_earned: u64,
    pub completion_timestamp: u64,
    pub metadata: HashMap<String, String>,
}

// Task verification response to the main canister
#[derive(CandidType, Deserialize, Clone)]
struct TaskVerificationResponse {
    pub success: bool,
    pub message: String,
    pub verification_data: Option<VerificationData>,
}

// Daily check-in response for direct API calls
#[derive(CandidType, Deserialize, Clone)]
struct DailyCheckInResponse {
    pub success: bool,
    pub points_earned: u64,
    pub consecutive_days: u64,
    pub bonus_points: u64,
    pub total_points: u64,
    pub next_claim_available_at: u64,
}

// Constants
const SECONDS_IN_DAY: u64 = 86400000;
const DAILY_CHECK_IN_POINTS: u64 = 10;
const MAX_CONSECUTIVE_BONUS_DAYS: u64 = 7;
const CONSECUTIVE_DAYS_BONUS_MULTIPLIER: u64 = 2;

// Direct daily check-in API for users
#[update]
fn claim_daily_check_in() -> DailyCheckInResponse {
    let caller = caller();
    let now = time() / 1_000_000;
    
    // Calculate the start of the current day in UTC time
    // Note: IC timestamp is in nanoseconds, convert to seconds for calculation
    let today_start = (now / SECONDS_IN_DAY) * SECONDS_IN_DAY;
    
    // Check if already claimed today
    let already_checked_in = STORAGE.with(|storage| {
        let storage = storage.borrow();
        
        if let Some(last_checkin) = storage.user_checkins.get(&caller) {
            // Check if last check-in time is within today's range
            // Calculate the start of the last check-in day
            let last_checkin_day_start = *last_checkin - (*last_checkin % SECONDS_IN_DAY);
            let checked_in_today = last_checkin_day_start == today_start;
            
            checked_in_today
        } else {
            false
        }
    });
    
    if already_checked_in {
        return DailyCheckInResponse {
            success: false,
            points_earned: 0,
            consecutive_days: 0,
            bonus_points: 0,
            total_points: 0,
            next_claim_available_at: today_start + SECONDS_IN_DAY,
        };
    }
    
    // Process the check-in
    let result = process_daily_checkin(caller, now, today_start);
    
    // Return the response
    DailyCheckInResponse {
        success: true,
        points_earned: DAILY_CHECK_IN_POINTS,
        consecutive_days: result.0,
        bonus_points: result.1,
        total_points: DAILY_CHECK_IN_POINTS + result.1,
        next_claim_available_at: today_start + SECONDS_IN_DAY,
    }
}

// Verify the daily check-in task (called by the main system)
#[update]
fn verify_task(request: TaskVerificationRequest) -> TaskVerificationResponse {
    let user = request.user;
    let now = time() / 1_000_000;
    
    // Calculate the start of the current day in UTC time
    let today_start = (now / SECONDS_IN_DAY) * SECONDS_IN_DAY;
    
    // Check if the user has already checked in today
    let already_checked_in = STORAGE.with(|storage| {
        let storage = storage.borrow();
        
        if let Some(last_checkin) = storage.user_checkins.get(&user) {
            // Check if the last check-in time is within today's range
            // Calculate the start of the last check-in day
            let last_checkin_day_start = *last_checkin - (*last_checkin % SECONDS_IN_DAY);
            let checked_in_today = last_checkin_day_start == today_start;
            
            checked_in_today
        } else {
            false
        }
    });
    
    if already_checked_in {
        return TaskVerificationResponse {
            success: false,
            message: "Already claimed daily check-in today".to_string(),
            verification_data: None,
        };
    }
    
    // Process the check-in
    let (consecutive_days, bonus_points) = process_daily_checkin(user, now, today_start);
    
    // Calculate total points
    let total_points = DAILY_CHECK_IN_POINTS + bonus_points;
    
    // Create metadata
    let mut metadata = HashMap::new();
    metadata.insert("consecutive_days".to_string(), consecutive_days.to_string());
    metadata.insert("bonus_points".to_string(), bonus_points.to_string());
    metadata.insert("next_claim_available_at".to_string(), (today_start + SECONDS_IN_DAY).to_string());
    
    // Create verification data
    let verification_data = VerificationData {
        task_id: request.task_id,
        points_earned: total_points,
        completion_timestamp: now,
        metadata,
    };
    
    // Return successful response
    TaskVerificationResponse {
        success: true,
        message: format!("Daily check-in successful (Day {})", consecutive_days),
        verification_data: Some(verification_data),
    }
}

// Common function to process a daily check-in
fn process_daily_checkin(user: Principal, now: u64, today_start: u64) -> (u64, u64) {
    STORAGE.with(|storage| {
        let mut storage = storage.borrow_mut();
        
        // Get task configuration
        let config = &storage.task_config;
        let base_points = if config.base_points > 0 { config.base_points } else { DAILY_CHECK_IN_POINTS };
        let max_consecutive_days = if config.max_consecutive_bonus_days > 0 { 
            config.max_consecutive_bonus_days 
        } else { 
            MAX_CONSECUTIVE_BONUS_DAYS 
        };
        let bonus_multiplier_divisor = if config.consecutive_days_bonus_multiplier > 0 {
            config.consecutive_days_bonus_multiplier
        } else {
            CONSECUTIVE_DAYS_BONUS_MULTIPLIER
        };
        
        // Get current consecutive days (default to 0)
        let current_consecutive_days = *storage.consecutive_days.get(&user).unwrap_or(&0);
        let mut new_consecutive_days = 1; // Default to 1 (first check-in)
        let mut bonus_points = 0;
        
        // Check if this is a consecutive day
        if let Some(last_checkin) = storage.user_checkins.get(&user) {
            let last_checkin_day = *last_checkin - (*last_checkin % SECONDS_IN_DAY);
            let yesterday_start = today_start - SECONDS_IN_DAY;
            
            if last_checkin_day == yesterday_start {
                // This is a consecutive day
                // If current consecutive days reach the maximum, reset to 1 and no bonus points
                if current_consecutive_days >= max_consecutive_days {
                    new_consecutive_days = 1;
                    bonus_points = 0;
                } else {
                    new_consecutive_days = current_consecutive_days + 1;
                    
                    // Only calculate bonus points if not resetting
                    // Calculate bonus multiplier
                    let bonus_multiplier = new_consecutive_days;
                    
                    // Calculate bonus points
                    bonus_points = (base_points * bonus_multiplier) / bonus_multiplier_divisor;
                }
                
            }
        }
        
        // Update storage - store today's start time instead of current time
        // This ensures consistent check-in status verification
        storage.user_checkins.insert(user, today_start);
        storage.consecutive_days.insert(user, new_consecutive_days);
        
        // Update points (for direct API calls)
        let total_points = DAILY_CHECK_IN_POINTS + bonus_points;
        let current_points = *storage.user_points.get(&user).unwrap_or(&0);
        storage.user_points.insert(user, current_points + total_points);
        
        // Add points transaction
        let transaction = PointsTransaction {
            amount: total_points as i64,
            reason: format!("Daily check-in (Day {})", new_consecutive_days),
            timestamp: now,
            reference_id: Some("daily_checkin".to_string()),
        };
        
        let user_history = storage.points_history.entry(user).or_insert_with(Vec::new);
        user_history.push(transaction);
        
        (new_consecutive_days, bonus_points)
    })
}

// Get the current check-in status for the caller
#[query]
fn get_my_checkin_status() -> HashMap<String, String> {
    let user = caller();
    get_checkin_status(user)
}

// Get the current check-in status for a specific user (admin or for verification)
#[query]
fn get_checkin_status(user: Principal) -> HashMap<String, String> {
    let now = time() / 1_000_000;
    
    // Calculate the start of the current day in UTC time
    let today_start = (now / SECONDS_IN_DAY) * SECONDS_IN_DAY;
    
    STORAGE.with(|storage| {
        let storage = storage.borrow();
        let mut status = HashMap::new();
        
        // Check if user exists in check-in records
        if let Some(last_checkin) = storage.user_checkins.get(&user) {
            let last_checkin_day = *last_checkin - (*last_checkin % SECONDS_IN_DAY);
            let has_checked_in_today = last_checkin_day == today_start;
            
            // Get consecutive days and total points
            let consecutive_days = storage.consecutive_days.get(&user).unwrap_or(&0);
            let total_points = storage.user_points.get(&user).unwrap_or(&0);
            
            // Fill status information
            status.insert("has_checked_in_today".to_string(), has_checked_in_today.to_string());
            status.insert("consecutive_days".to_string(), consecutive_days.to_string());
            status.insert("total_points".to_string(), total_points.to_string());
            status.insert("last_checkin_time".to_string(), last_checkin.to_string()); // Add last check-in time for debugging
            status.insert("last_checkin_day".to_string(), last_checkin_day.to_string()); // Add last check-in date for debugging
            status.insert("today_start".to_string(), today_start.to_string()); // Add today start time for debugging
            status.insert("next_claim_available_at".to_string(), 
                if has_checked_in_today {
                    (today_start + SECONDS_IN_DAY).to_string()
                } else {
                    "0".to_string() // Can claim now
                }
            );
        } else {
            // Get consecutive days and total points
            let consecutive_days = storage.consecutive_days.get(&user).unwrap_or(&0);
            let total_points = storage.user_points.get(&user).unwrap_or(&0);
            
            // Fill status information
            status.insert("has_checked_in_today".to_string(), "false".to_string());
            status.insert("consecutive_days".to_string(), consecutive_days.to_string());
            status.insert("total_points".to_string(), total_points.to_string());
            status.insert("today_start".to_string(), today_start.to_string()); // Add today start time for debugging
            status.insert("next_claim_available_at".to_string(), "0".to_string()); // Can claim now
        }
        
        status
    })
}

// Get user points history
#[query]
fn get_user_rewards(user: Principal) -> HashMap<String, Value> {
    let mut result = HashMap::new();
    
    STORAGE.with(|storage| {
        let storage = storage.borrow();
        
        // Get user points
        let points = storage.user_points.get(&user).cloned().unwrap_or(0);
        result.insert("points".to_string(), Value::Nat(points));
        
        // Get consecutive days
        let consecutive_days = storage.consecutive_days.get(&user).cloned().unwrap_or(0);
        result.insert("consecutive_days".to_string(), Value::Nat(consecutive_days));
        
        // Get last check-in timestamp
        if let Some(last_checkin) = storage.user_checkins.get(&user) {
            result.insert("last_checkin".to_string(), Value::Nat(*last_checkin));
            
            // Calculate if user can check in today
            let now = time() / 1_000_000;
            let today_start = now - (now % SECONDS_IN_DAY);
            let last_checkin_day = last_checkin - (last_checkin % SECONDS_IN_DAY);
            let has_checked_in_today = last_checkin_day == today_start;
            
            result.insert("can_checkin_today".to_string(), Value::Bool(!has_checked_in_today));
            
            if has_checked_in_today {
                let next_claim_time = today_start + SECONDS_IN_DAY;
                result.insert("next_claim_available_at".to_string(), Value::Nat(next_claim_time));
            }
        } else {
            // User has never checked in
            result.insert("can_checkin_today".to_string(), Value::Bool(true));
        }
        
        // Get points history
        if let Some(history) = storage.points_history.get(&user) {
            result.insert("points_history_count".to_string(), Value::Nat(history.len() as u64));
            
            // Add latest transaction if available
            if !history.is_empty() {
                let latest = &history[history.len() - 1];
                result.insert("latest_transaction_amount".to_string(), Value::Int(latest.amount));
                result.insert("latest_transaction_reason".to_string(), Value::Text(latest.reason.clone()));
                result.insert("latest_transaction_timestamp".to_string(), Value::Nat(latest.timestamp));
            }
            
            // Add full points history
            let history_entries: Vec<Value> = history.iter().map(|transaction| {
                let mut entry = Vec::new();
                entry.push(("amount".to_string(), Value::Int(transaction.amount)));
                entry.push(("reason".to_string(), Value::Text(transaction.reason.clone())));
                entry.push(("timestamp".to_string(), Value::Nat(transaction.timestamp)));
                if let Some(ref_id) = &transaction.reference_id {
                    entry.push(("reference_id".to_string(), Value::Text(ref_id.clone())));
                }
                Value::Map(entry)
            }).collect();
            
            result.insert("points_history".to_string(), Value::Array(history_entries));
        } else {
            result.insert("points_history_count".to_string(), Value::Nat(0));
            result.insert("points_history".to_string(), Value::Array(vec![]));
        }
    });
    
    result
}

// Reset a user's check-in streak (admin function)
#[update]
fn reset_user_streak(user: Principal) {
    // Verify caller is an admin
    let caller_principal = caller();
    let is_admin = STORAGE.with(|storage| {
        let storage = storage.borrow();
        storage.admins.contains(&caller_principal)
    });
    
    if !is_admin {
        ic_cdk::trap("Caller is not authorized to perform this action");
    }
    STORAGE.with(|storage| {
        let mut storage = storage.borrow_mut();
        storage.consecutive_days.insert(user, 0);
    });
}

// Manually award points to a user (admin function)
#[update]
fn award_points(user: Principal, points: u64, reason: String) {
    // Verify caller is an admin
    let caller_principal = caller();
    let is_admin = STORAGE.with(|storage| {
        let storage = storage.borrow();
        storage.admins.contains(&caller_principal)
    });
    
    if !is_admin {
        ic_cdk::trap("Caller is not authorized to perform this action");
    }
    STORAGE.with(|storage| {
        let mut storage = storage.borrow_mut();
        
        // Update points
        let current_points = *storage.user_points.get(&user).unwrap_or(&0);
        storage.user_points.insert(user, current_points + points);
        
        // Add transaction
        let transaction = PointsTransaction {
            amount: points as i64,
            reason,
            timestamp: time() / 1_000_000,
            reference_id: None,
        };
        
        let user_history = storage.points_history.entry(user).or_insert_with(Vec::new);
        user_history.push(transaction);
    });
}

// Initialize the canister with default settings
#[init]
fn init() {
    let caller_principal = caller();
    
    STORAGE.with(|storage| {
        let mut storage = storage.borrow_mut();
        
        // Add the deployer as an admin
        storage.admins.push(caller_principal);
        
        // Set default task configuration
        storage.task_config = TaskConfig {
            title: "Daily Check-in".to_string(),
            description: "Check in daily to earn points and build a streak".to_string(),
            base_points: DAILY_CHECK_IN_POINTS,
            max_consecutive_bonus_days: MAX_CONSECUTIVE_BONUS_DAYS,
            consecutive_days_bonus_multiplier: CONSECUTIVE_DAYS_BONUS_MULTIPLIER,
            enabled: true,
        };
    });
}

// Update task configuration (admin function)
#[update]
fn update_task_config(config: TaskConfig) {
    // Verify caller is an admin
    let caller_principal = caller();
    let is_admin = STORAGE.with(|storage| {
        let storage = storage.borrow();
        storage.admins.contains(&caller_principal)
    });
    
    if !is_admin {
        ic_cdk::trap("Caller is not authorized to perform this action");
    }
    
    STORAGE.with(|storage| {
        let mut storage = storage.borrow_mut();
        storage.task_config = config;
    });
}

// Get task configuration
#[query]
fn get_task_config() -> TaskConfig {
    STORAGE.with(|storage| {
        let storage = storage.borrow();
        storage.task_config.clone()
    })
}

// Add an admin (admin function)
#[update]
fn add_admin(principal: Principal) {
    // Verify caller is an admin
    let caller_principal = caller();
    let is_admin = STORAGE.with(|storage| {
        let storage = storage.borrow();
        storage.admins.contains(&caller_principal)
    });
    
    if !is_admin {
        ic_cdk::trap("Caller is not authorized to perform this action");
    }
    
    STORAGE.with(|storage| {
        let mut storage = storage.borrow_mut();
        if !storage.admins.contains(&principal) {
            storage.admins.push(principal);
        }
    });
}

// Remove an admin (admin function)
#[update]
fn remove_admin(principal: Principal) {
    // Verify caller is an admin
    let caller_principal = caller();
    let is_admin = STORAGE.with(|storage| {
        let storage = storage.borrow();
        storage.admins.contains(&caller_principal)
    });
    
    if !is_admin {
        ic_cdk::trap("Caller is not authorized to perform this action");
    }
    
    STORAGE.with(|storage| {
        let mut storage = storage.borrow_mut();
        storage.admins.retain(|p| p != &principal);
    });
}

// Get all admins
#[query]
fn get_admins() -> Vec<Principal> {
    STORAGE.with(|storage| {
        let storage = storage.borrow();
        storage.admins.clone()
    })
}

// State management for canister upgrades
#[pre_upgrade]
fn pre_upgrade() {
    ic_cdk::println!("Starting pre-upgrade hook for daily_checkin_task");
    save_state_for_upgrade();
    ic_cdk::println!("Pre-upgrade hook completed");
}

#[post_upgrade]
fn post_upgrade() {
    ic_cdk::println!("Starting post-upgrade hook for daily_checkin_task");
    
    // Restore state
    restore_state_after_upgrade();
    
    ic_cdk::println!("Post-upgrade hook completed");
}

// Save state to stable storage before canister upgrade
fn save_state_for_upgrade() {
    use ic_cdk::storage::stable_save;
    
    STORAGE.with(|storage| {
        let storage_data = storage.borrow();
        
        // Save storage data
        match stable_save((storage_data.clone(),)) {
            Ok(_) => ic_cdk::println!("Successfully saved daily check-in state"),
            Err(e) => {
                let error_msg = format!("Failed to save daily check-in state: {:?}", e);
                ic_cdk::println!("{}", error_msg);
                ic_cdk::trap(&error_msg);
            }
        }
    });
}

// Restore state from stable storage after canister upgrade
fn restore_state_after_upgrade() {
    use ic_cdk::storage::stable_restore;
    
    // Try to restore the state
    let restore_result: Result<(DailyCheckInStorage,), String> = stable_restore();
    
    match restore_result {
        Ok((storage_data,)) => {
            STORAGE.with(|storage| {
                *storage.borrow_mut() = storage_data;
            });
            
            // Log some stats about the restored data
            STORAGE.with(|storage| {
                let storage = storage.borrow();
                ic_cdk::println!("Restored daily check-in state: {} users, {} admins", 
                               storage.user_checkins.len(),
                               storage.admins.len());
            });
        },
        Err(e) => {
            let error_msg = format!("Failed to restore daily check-in state: {:?}", e);
            ic_cdk::println!("{}", error_msg);
            ic_cdk::println!("Starting with empty state");
            
            // Initialize with default state
            let caller_principal = caller();
            STORAGE.with(|storage| {
                let mut storage = storage.borrow_mut();
                
                // Add the deployer as an admin
                storage.admins.push(caller_principal);
                
                // Set default task configuration
                storage.task_config = TaskConfig {
                    title: "Daily Check-in".to_string(),
                    description: "Check in daily to earn points and build a streak".to_string(),
                    base_points: DAILY_CHECK_IN_POINTS,
                    max_consecutive_bonus_days: MAX_CONSECUTIVE_BONUS_DAYS,
                    consecutive_days_bonus_multiplier: CONSECUTIVE_DAYS_BONUS_MULTIPLIER,
                    enabled: true,
                };
            });
        }
    }
}

ic_cdk::export_candid!();
