use crate::*;
use std::collections::{HashMap, HashSet, BTreeMap};

// Mock the daily check-in storage
thread_local! {
    static MOCK_STORAGE: RefCell<MockDailyCheckInStorage> = RefCell::new(MockDailyCheckInStorage::default());
}

#[derive(Default, Clone)]
struct MockDailyCheckInStorage {
    user_checkins: HashMap<Principal, u64>,
    consecutive_days: HashMap<Principal, u64>,
    user_points: HashMap<Principal, u64>,
    admins: HashSet<Principal>,
    checkin_time_index: BTreeMap<u64, Vec<Principal>>,
    consecutive_days_index: BTreeMap<u64, Vec<Principal>>,
    total_points_index: BTreeMap<u64, Vec<Principal>>,
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
fn mock_process_daily_checkin(user: Principal, _now: u64, today_start: u64) -> (u64, u64) {
    MOCK_STORAGE.with(|storage| {
        let mut storage = storage.borrow_mut();
        
        // Get current consecutive days (default to 0)
        let current_consecutive_days = *storage.consecutive_days.get(&user).unwrap_or(&0);
        let mut new_consecutive_days = 1; // Default to 1 (first check-in)
        let mut bonus_points = 0;
        
        // Check if this is a consecutive day
        let is_consecutive = if let Some(last_checkin) = storage.user_checkins.get(&user).cloned() {
            let last_checkin_day = last_checkin - (last_checkin % SECONDS_IN_DAY);
            let yesterday_start = today_start - SECONDS_IN_DAY;
            
            // Check if consecutive and store the last_checkin value
            last_checkin_day == yesterday_start
        } else {
            false
        };
        
        if is_consecutive {
            // This is a consecutive day
            new_consecutive_days = current_consecutive_days + 1;
            
            // Calculate bonus points
            bonus_points = DAILY_CHECK_IN_POINTS * new_consecutive_days / CONSECUTIVE_DAYS_BONUS_MULTIPLIER;
        }
        
        // Get old values before updating
        let old_checkin = storage.user_checkins.get(&user).cloned();
        let old_days = storage.consecutive_days.get(&user).cloned();
        let old_points = storage.user_points.get(&user).cloned();
        
        // Now update the indices with the old values
        if let Some(last_checkin) = old_checkin {
            if let Some(users) = storage.checkin_time_index.get_mut(&last_checkin) {
                users.retain(|p| p != &user);
                if users.is_empty() {
                    storage.checkin_time_index.remove(&last_checkin);
                }
            }
        }
        
        if let Some(days) = old_days {
            if let Some(users) = storage.consecutive_days_index.get_mut(&days) {
                users.retain(|p| p != &user);
                if users.is_empty() {
                    storage.consecutive_days_index.remove(&days);
                }
            }
        }
        
        if let Some(points) = old_points {
            if let Some(users) = storage.total_points_index.get_mut(&points) {
                users.retain(|p| p != &user);
                if users.is_empty() {
                    storage.total_points_index.remove(&points);
                }
            }
        }
        
        // Update storage
        storage.user_checkins.insert(user, today_start);
        storage.consecutive_days.insert(user, new_consecutive_days);
        
        // Update points
        let total_points = DAILY_CHECK_IN_POINTS + bonus_points;
        let current_points = *storage.user_points.get(&user).unwrap_or(&0);
        let new_total_points = current_points + total_points;
        storage.user_points.insert(user, new_total_points);
        
        // Update indices
        storage.checkin_time_index.entry(today_start)
            .or_insert_with(Vec::new)
            .push(user);
            
        storage.consecutive_days_index.entry(new_consecutive_days)
            .or_insert_with(Vec::new)
            .push(user);
            
        storage.total_points_index.entry(new_total_points)
            .or_insert_with(Vec::new)
            .push(user);
        
        (new_consecutive_days, bonus_points)
    })
}

// Mock the CheckInDetail struct
#[derive(CandidType, Deserialize, Clone, Debug, PartialEq)]
struct MockCheckInDetail {
    pub user: Principal,
    pub last_checkin_time: u64,
    pub consecutive_days: u64,
    pub total_points: u64,
}

// Mock the PaginatedCheckInDetails struct
#[derive(CandidType, Deserialize, Clone, Debug)]
struct MockPaginatedCheckInDetails {
    pub details: Vec<MockCheckInDetail>,
    pub total_count: u64,
    pub page: u64,
    pub page_size: u64,
    pub total_pages: u64,
}

// Mock the get_all_checkin_details function
fn mock_get_all_checkin_details(caller: Principal, page: u64, page_size: u64, sort_by: Option<String>, sort_order: Option<String>) -> Result<MockPaginatedCheckInDetails, String> {
    // Check if caller is admin
    let is_admin = MOCK_STORAGE.with(|storage| {
        let storage = storage.borrow();
        storage.admins.contains(&caller)
    });
    
    if !is_admin {
        return Err("Unauthorized: Only admins can access all check-in details".to_string());
    }
    
    // Validate pagination parameters
    if page == 0 {
        return Err("Page number must be at least 1".to_string());
    }
    
    if page_size == 0 || page_size > 100 {
        return Err("Page size must be between 1 and 100".to_string());
    }
    
    // Get sort parameters with defaults
    let sort_by = sort_by.unwrap_or_else(|| "last_checkin_time".to_string());
    let sort_order = sort_order.unwrap_or_else(|| "desc".to_string());
    
    // Validate sort parameters
    if !vec!["last_checkin_time", "consecutive_days", "total_points"].contains(&sort_by.as_str()) {
        return Err("Invalid sort_by parameter. Must be one of: last_checkin_time, consecutive_days, total_points".to_string());
    }
    
    if !vec!["asc", "desc"].contains(&sort_order.as_str()) {
        return Err("Invalid sort_order parameter. Must be one of: asc, desc".to_string());
    }
    
    MOCK_STORAGE.with(|storage| {
        let storage = storage.borrow();
        
        // Get total count of users with check-ins
        let total_count = storage.user_checkins.len() as u64;
        let total_pages = (total_count + page_size - 1) / page_size; // Ceiling division
        
        // Adjust page if it exceeds total pages
        let effective_page = if page > total_pages && total_pages > 0 {
            total_pages
        } else {
            page
        };
        
        // Calculate pagination limits
        let start_idx = ((effective_page - 1) * page_size) as usize;
        let limit = page_size as usize;
        
        // Use the appropriate index based on sort field and order
        let mut page_details = Vec::with_capacity(limit);
        
        match sort_by.as_str() {
            "last_checkin_time" => {
                // Use the checkin_time_index for efficient retrieval
                let mut sorted_users = Vec::new();
                
                if sort_order == "asc" {
                    // Ascending order - iterate from oldest to newest
                    for (timestamp, users) in &storage.checkin_time_index {
                        for user in users {
                            sorted_users.push((*timestamp, *user));
                        }
                    }
                    sorted_users.sort_by(|a, b| a.0.cmp(&b.0)); // Sort by timestamp ascending
                } else {
                    // Descending order - iterate from newest to oldest
                    for (timestamp, users) in storage.checkin_time_index.iter().rev() {
                        for user in users {
                            sorted_users.push((*timestamp, *user));
                        }
                    }
                }
                
                // Apply pagination
                let paginated_users = if start_idx < sorted_users.len() {
                    let end_idx = std::cmp::min(start_idx + limit, sorted_users.len());
                    &sorted_users[start_idx..end_idx]
                } else {
                    &[]
                };
                
                // Create detail objects for the paginated users
                for (timestamp, user) in paginated_users {
                    let consecutive_days = storage.consecutive_days.get(user).cloned().unwrap_or(0);
                    let total_points = storage.user_points.get(user).cloned().unwrap_or(0);
                    
                    page_details.push(MockCheckInDetail {
                        user: *user,
                        last_checkin_time: *timestamp,
                        consecutive_days,
                        total_points,
                    });
                }
            },
            "consecutive_days" => {
                // Similar implementation for consecutive_days sorting
                let mut sorted_users = Vec::new();
                
                if sort_order == "asc" {
                    for (days, users) in &storage.consecutive_days_index {
                        for user in users {
                            sorted_users.push((*days, *user));
                        }
                    }
                } else {
                    for (days, users) in storage.consecutive_days_index.iter().rev() {
                        for user in users {
                            sorted_users.push((*days, *user));
                        }
                    }
                }
                
                let paginated_users = if start_idx < sorted_users.len() {
                    let end_idx = std::cmp::min(start_idx + limit, sorted_users.len());
                    &sorted_users[start_idx..end_idx]
                } else {
                    &[]
                };
                
                for (days, user) in paginated_users {
                    let last_checkin_time = storage.user_checkins.get(user).cloned().unwrap_or(0);
                    let total_points = storage.user_points.get(user).cloned().unwrap_or(0);
                    
                    page_details.push(MockCheckInDetail {
                        user: *user,
                        last_checkin_time,
                        consecutive_days: *days,
                        total_points,
                    });
                }
            },
            "total_points" => {
                // Similar implementation for total_points sorting
                let mut sorted_users = Vec::new();
                
                if sort_order == "asc" {
                    for (points, users) in &storage.total_points_index {
                        for user in users {
                            sorted_users.push((*points, *user));
                        }
                    }
                } else {
                    for (points, users) in storage.total_points_index.iter().rev() {
                        for user in users {
                            sorted_users.push((*points, *user));
                        }
                    }
                }
                
                let paginated_users = if start_idx < sorted_users.len() {
                    let end_idx = std::cmp::min(start_idx + limit, sorted_users.len());
                    &sorted_users[start_idx..end_idx]
                } else {
                    &[]
                };
                
                for (points, user) in paginated_users {
                    let last_checkin_time = storage.user_checkins.get(user).cloned().unwrap_or(0);
                    let consecutive_days = storage.consecutive_days.get(user).cloned().unwrap_or(0);
                    
                    page_details.push(MockCheckInDetail {
                        user: *user,
                        last_checkin_time,
                        consecutive_days,
                        total_points: *points,
                    });
                }
            },
            _ => {
                // Default to last_checkin_time desc
                let mut sorted_users = Vec::new();
                
                for (timestamp, users) in storage.checkin_time_index.iter().rev() {
                    for user in users {
                        sorted_users.push((*timestamp, *user));
                    }
                }
                
                let paginated_users = if start_idx < sorted_users.len() {
                    let end_idx = std::cmp::min(start_idx + limit, sorted_users.len());
                    &sorted_users[start_idx..end_idx]
                } else {
                    &[]
                };
                
                for (timestamp, user) in paginated_users {
                    let consecutive_days = storage.consecutive_days.get(user).cloned().unwrap_or(0);
                    let total_points = storage.user_points.get(user).cloned().unwrap_or(0);
                    
                    page_details.push(MockCheckInDetail {
                        user: *user,
                        last_checkin_time: *timestamp,
                        consecutive_days,
                        total_points,
                    });
                }
            }
        }
        
        Ok(MockPaginatedCheckInDetails {
            details: page_details,
            total_count,
            page: effective_page,
            page_size,
            total_pages,
        })
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
    
    // Helper function to create a principal from a number
    fn create_principal(n: u64) -> Principal {
        // Create a unique principal for each number
        let bytes = n.to_be_bytes();
        let mut principal_bytes = vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        principal_bytes[0..8].copy_from_slice(&bytes);
        Principal::from_slice(&principal_bytes)
    }
    
    // Helper function to reset the mock storage
    fn reset_mock_storage() {
        MOCK_STORAGE.with(|storage| {
            *storage.borrow_mut() = MockDailyCheckInStorage::default();
        });
    }
    
    // Helper function to set up an admin
    fn setup_admin(admin: Principal) {
        MOCK_STORAGE.with(|storage| {
            let mut storage = storage.borrow_mut();
            storage.admins.insert(admin);
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
    
    #[test]
    fn test_many_users_check_in() {
        // This test simulates 1000 users checking in and tests the pagination and sorting
        reset_mock_storage();
        
        let admin = test_principal();
        setup_admin(admin);
        
        // Create 1000 users and have them check in
        let base_time = 1000 * SECONDS_IN_DAY; // Day 1000
        let num_users = 1000;
        
        // Track some users for verification
        let mut user_timestamps = Vec::new();
        let mut user_consecutive_days = Vec::new();
        let mut user_total_points = Vec::new();
        
        println!("Setting up {} users for check-in test...", num_users);
        
        // Have users check in with different patterns
        for i in 0..num_users {
            let user = create_principal(i as u64);
            let days_offset = i % 10; // Create different check-in patterns
            let check_in_time = base_time + days_offset * SECONDS_IN_DAY;
            
            // First check-in
            let _ = mock_claim_daily_check_in(user, check_in_time);
            
            // Some users do consecutive check-ins
            if i % 3 == 0 {
                let consecutive_check_in_time = check_in_time + SECONDS_IN_DAY;
                let _ = mock_claim_daily_check_in(user, consecutive_check_in_time);
            }
            
            // Some users do even more consecutive check-ins
            if i % 9 == 0 {
                let consecutive_check_in_time_2 = check_in_time + 2 * SECONDS_IN_DAY;
                let _ = mock_claim_daily_check_in(user, consecutive_check_in_time_2);
            }
            
            // Store some user data for verification
            if i < 10 {
                MOCK_STORAGE.with(|storage| {
                    let storage = storage.borrow();
                    user_timestamps.push((user, *storage.user_checkins.get(&user).unwrap()));
                    user_consecutive_days.push((user, *storage.consecutive_days.get(&user).unwrap()));
                    user_total_points.push((user, *storage.user_points.get(&user).unwrap()));
                });
            }
        }
        
        println!("All users checked in. Testing pagination and sorting...");
        
        // Test pagination - first page
        let page_size = 50;
        let result = mock_get_all_checkin_details(admin, 1, page_size, None, None).unwrap();
        
        // Verify pagination metadata
        assert_eq!(result.total_count, num_users as u64);
        assert_eq!(result.page, 1);
        assert_eq!(result.page_size, page_size);
        assert_eq!(result.total_pages, (num_users as u64 + page_size - 1) / page_size);
        assert_eq!(result.details.len(), page_size as usize);
        
        // Test pagination - second page
        let result = mock_get_all_checkin_details(admin, 2, page_size, None, None).unwrap();
        assert_eq!(result.page, 2);
        assert_eq!(result.details.len(), page_size as usize);
        
        // Test pagination - last page
        let last_page = result.total_pages;
        let result = mock_get_all_checkin_details(admin, last_page, page_size, None, None).unwrap();
        assert_eq!(result.page, last_page);
        
        // Test sorting by last_checkin_time (default)
        let result = mock_get_all_checkin_details(admin, 1, 10, Some("last_checkin_time".to_string()), Some("desc".to_string())).unwrap();
        for i in 1..result.details.len() {
            assert!(result.details[i-1].last_checkin_time >= result.details[i].last_checkin_time);
        }
        
        // Test sorting by consecutive_days
        let result = mock_get_all_checkin_details(admin, 1, 10, Some("consecutive_days".to_string()), Some("desc".to_string())).unwrap();
        for i in 1..result.details.len() {
            assert!(result.details[i-1].consecutive_days >= result.details[i].consecutive_days);
        }
        
        // Test sorting by total_points
        let result = mock_get_all_checkin_details(admin, 1, 10, Some("total_points".to_string()), Some("desc".to_string())).unwrap();
        for i in 1..result.details.len() {
            assert!(result.details[i-1].total_points >= result.details[i].total_points);
        }
        
        // Test unauthorized access
        let non_admin = create_principal(9999);
        let error = mock_get_all_checkin_details(non_admin, 1, 10, None, None).unwrap_err();
        assert!(error.contains("Unauthorized"));
        
        // Test invalid pagination parameters
        let error = mock_get_all_checkin_details(admin, 0, 10, None, None).unwrap_err();
        assert!(error.contains("Page number must be at least 1"));
        
        let error = mock_get_all_checkin_details(admin, 1, 0, None, None).unwrap_err();
        assert!(error.contains("Page size must be between 1 and 100"));
        
        let error = mock_get_all_checkin_details(admin, 1, 101, None, None).unwrap_err();
        assert!(error.contains("Page size must be between 1 and 100"));
        
        // Test invalid sort parameters
        let error = mock_get_all_checkin_details(admin, 1, 10, Some("invalid_field".to_string()), None).unwrap_err();
        assert!(error.contains("Invalid sort_by parameter"));
        
        let error = mock_get_all_checkin_details(admin, 1, 10, None, Some("invalid_order".to_string())).unwrap_err();
        assert!(error.contains("Invalid sort_order parameter"));
        
        println!("All pagination and sorting tests passed!");
    }
}
