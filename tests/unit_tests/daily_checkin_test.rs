use candid::{CandidType, Deserialize, Principal};
use ic_cdk::api::{time, caller};
use std::cell::RefCell;
use std::collections::HashMap;

// Import the daily_checkin_task module
// Note: In a real test, you would import the actual module
// For this example, we'll mock the necessary components

// Mock the daily check-in storage
thread_local! {
    static MOCK_STORAGE: RefCell<MockDailyCheckInStorage> = RefCell::new(MockDailyCheckInStorage::default());
}

#[derive(Default, Clone)]
struct MockDailyCheckInStorage {
    user_checkins: HashMap<Principal, u64>,
    consecutive_days: HashMap<Principal, u64>,
    user_points: HashMap<Principal, u64>,
}

// Mock the daily check-in response
#[derive(CandidType, Deserialize, Clone, Debug)]
struct MockDailyCheckInResponse {
    pub success: bool,
    pub points_earned: u64,
    pub consecutive_days: u64,
    pub bonus_points: u64,
    pub total_points: u64,
    pub next_claim_available_at: u64,
}

// Constants for testing
const SECONDS_IN_DAY: u64 = 86400;
const DAILY_CHECK_IN_POINTS: u64 = 10;

// Mock the claim_daily_check_in function
fn mock_claim_daily_check_in(user: Principal, current_time: u64) -> Result<MockDailyCheckInResponse, String> {
    // Calculate the start of the current day in UTC time
    let today_start = (current_time / SECONDS_IN_DAY) * SECONDS_IN_DAY;
    
    // Check if already claimed today
    let already_checked_in = MOCK_STORAGE.with(|storage| {
        let storage = storage.borrow();
        
        if let Some(last_checkin) = storage.user_checkins.get(&user) {
            // Check if last check-in time is within today's range
            let last_checkin_day_start = *last_checkin - (*last_checkin % SECONDS_IN_DAY);
            let checked_in_today = last_checkin_day_start == today_start;
            
            checked_in_today
        } else {
            false
        }
    });
    
    if already_checked_in {
        return Err("Already claimed daily check-in today".to_string());
    }
    
    // Process the check-in
    let (consecutive_days, bonus_points) = mock_process_daily_checkin(user, current_time, today_start);
    
    // Return the response
    Ok(MockDailyCheckInResponse {
        success: true,
        points_earned: DAILY_CHECK_IN_POINTS,
        consecutive_days,
        bonus_points,
        total_points: DAILY_CHECK_IN_POINTS + bonus_points,
        next_claim_available_at: today_start + SECONDS_IN_DAY,
    })
}

// Mock the process_daily_checkin function
fn mock_process_daily_checkin(user: Principal, now: u64, today_start: u64) -> (u64, u64) {
    MOCK_STORAGE.with(|storage| {
        let mut storage = storage.borrow_mut();
        
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
                new_consecutive_days = current_consecutive_days + 1;
                
                // Calculate bonus points
                bonus_points = DAILY_CHECK_IN_POINTS * new_consecutive_days / 2;
            }
        }
        
        // Update storage
        storage.user_checkins.insert(user, today_start);
        storage.consecutive_days.insert(user, new_consecutive_days);
        
        // Update points
        let total_points = DAILY_CHECK_IN_POINTS + bonus_points;
        let current_points = *storage.user_points.get(&user).unwrap_or(&0);
        storage.user_points.insert(user, current_points + total_points);
        
        (new_consecutive_days, bonus_points)
    })
}

// Unit tests
#[cfg(test)]
mod tests {
    use super::*;
    
    // Helper function to create a test principal
    fn test_principal() -> Principal {
        Principal::from_text("2vxsx-fae").unwrap()
    }
    
    // Helper function to reset the mock storage
    fn reset_mock_storage() {
        MOCK_STORAGE.with(|storage| {
            *storage.borrow_mut() = MockDailyCheckInStorage::default();
        });
    }
    
    #[test]
    fn test_first_check_in() {
        reset_mock_storage();
        
        let user = test_principal();
        let current_time = 1_000_000 * SECONDS_IN_DAY; // Some arbitrary time
        
        let result = mock_claim_daily_check_in(user, current_time);
        
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.success, true);
        assert_eq!(response.consecutive_days, 1);
        assert_eq!(response.bonus_points, 0);
        assert_eq!(response.points_earned, DAILY_CHECK_IN_POINTS);
        assert_eq!(response.total_points, DAILY_CHECK_IN_POINTS);
    }
    
    #[test]
    fn test_duplicate_check_in() {
        reset_mock_storage();
        
        let user = test_principal();
        let current_time = 1_000_000 * SECONDS_IN_DAY; // Some arbitrary time
        
        // First check-in should succeed
        let first_result = mock_claim_daily_check_in(user, current_time);
        assert!(first_result.is_ok());
        
        // Second check-in on the same day should fail
        let second_result = mock_claim_daily_check_in(user, current_time + 1000); // Same day, just a bit later
        assert!(second_result.is_err());
        assert_eq!(second_result.unwrap_err(), "Already claimed daily check-in today");
    }
    
    #[test]
    fn test_consecutive_check_in() {
        reset_mock_storage();
        
        let user = test_principal();
        let day1_time = 1_000_000 * SECONDS_IN_DAY; // Day 1
        let day2_time = day1_time + SECONDS_IN_DAY; // Day 2
        
        // Check-in on day 1
        let day1_result = mock_claim_daily_check_in(user, day1_time);
        assert!(day1_result.is_ok());
        
        // Check-in on day 2
        let day2_result = mock_claim_daily_check_in(user, day2_time);
        assert!(day2_result.is_ok());
        
        let day2_response = day2_result.unwrap();
        assert_eq!(day2_response.consecutive_days, 2);
        assert_eq!(day2_response.bonus_points, DAILY_CHECK_IN_POINTS); // Bonus should be 10 * 2 / 2 = 10
        assert_eq!(day2_response.total_points, DAILY_CHECK_IN_POINTS * 2);
    }
    
    #[test]
    fn test_non_consecutive_check_in() {
        reset_mock_storage();
        
        let user = test_principal();
        let day1_time = 1_000_000 * SECONDS_IN_DAY; // Day 1
        let day3_time = day1_time + (2 * SECONDS_IN_DAY); // Day 3 (skipping day 2)
        
        // Check-in on day 1
        let day1_result = mock_claim_daily_check_in(user, day1_time);
        assert!(day1_result.is_ok());
        
        // Check-in on day 3 (skipping day 2)
        let day3_result = mock_claim_daily_check_in(user, day3_time);
        assert!(day3_result.is_ok());
        
        let day3_response = day3_result.unwrap();
        assert_eq!(day3_response.consecutive_days, 1); // Should reset to 1
        assert_eq!(day3_response.bonus_points, 0); // No bonus for non-consecutive days
    }
}
