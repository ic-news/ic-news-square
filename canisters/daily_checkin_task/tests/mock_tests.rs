// Mock tests for daily check-in task canister
#[cfg(test)]
mod tests {
    use crate::*;
    use crate::test_utils::*;
    use candid::Principal;

    // Setup function to initialize test environment
    fn setup() {
        // Reset mocks
        reset_mocks();
        
        // Initialize storage with default values
        STORAGE.with(|storage| {
            *storage.borrow_mut() = DailyCheckInStorage::default();
        });
        
        // Set initial time
        set_mock_time(1000000000000000000); // Some arbitrary starting time
        
        // Set initial admin
        let admin = test_principal(0);
        STORAGE.with(|storage| {
            storage.borrow_mut().admins.push(admin);
        });
    }
    
    // Teardown function to clean up after tests
    fn teardown() {
        reset_mocks();
    }
    
    // Test initialization
    #[test]
    fn test_init() {
        setup();
        
        // Call init function
        init();
        
        // Verify task configuration is set to default values
        STORAGE.with(|storage| {
            let storage = storage.borrow();
            assert_eq!(storage.task_config.title, "");
            assert_eq!(storage.task_config.description, "");
            assert_eq!(storage.task_config.base_points, 0);
            assert_eq!(storage.task_config.max_consecutive_bonus_days, 0);
            assert_eq!(storage.task_config.consecutive_days_bonus_multiplier, 0);
            assert_eq!(storage.task_config.enabled, false);
            
            // Verify admin list contains the caller principal
            let caller = get_caller();
            assert!(storage.admins.contains(&caller));
        });
        
        teardown();
    }
    
    // Test daily check-in
    #[test]
    fn test_daily_checkin() {
        setup();
        
        // Set up a test user
        let user = test_principal(1);
        set_mock_caller(user);
        
        // Call claim_daily_check_in
        let response = claim_daily_check_in();
        
        // Verify the response is successful
        assert!(response.success);
        assert_eq!(response.points_earned, DAILY_CHECK_IN_POINTS);
        assert_eq!(response.consecutive_days, 1);
        assert_eq!(response.bonus_points, 0);
        
        // Verify user data was updated correctly in storage
        STORAGE.with(|storage| {
            let storage = storage.borrow();
            assert!(storage.user_checkins.contains_key(&user));
            assert_eq!(storage.consecutive_days.get(&user).unwrap(), &1);
        });
        
        teardown();
    }
    
    // Test consecutive check-ins
    #[test]
    fn test_consecutive_checkins() {
        setup();
        
        // Set up a test user
        let user = test_principal(1);
        set_mock_caller(user);
        
        // First check-in
        let response1 = claim_daily_check_in();
        assert!(response1.success);
        assert_eq!(response1.consecutive_days, 1);
        assert_eq!(response1.bonus_points, 0);
        
        // Advance time by 1 day
        advance_time_by_days(1);
        
        // Second check-in
        let response2 = claim_daily_check_in();
        assert!(response2.success);
        assert_eq!(response2.consecutive_days, 2);
        assert_eq!(response2.bonus_points, 2); // 1 * 2 = 2
        
        // Advance time by 1 day
        advance_time_by_days(1);
        
        // Third check-in
        let response3 = claim_daily_check_in();
        assert!(response3.success);
        assert_eq!(response3.consecutive_days, 3);
        assert_eq!(response3.bonus_points, 4); // 2 * 2 = 4
        
        // Verify user data was updated correctly in storage
        STORAGE.with(|storage| {
            let storage = storage.borrow();
            assert_eq!(storage.consecutive_days.get(&user).unwrap(), &3);
        });
        
        teardown();
    }
    
    // Test missed check-in (streak reset)
    #[test]
    fn test_missed_checkin() {
        setup();
        
        // Set up a test user
        let user = test_principal(1);
        set_mock_caller(user);
        
        // First check-in
        let response1 = claim_daily_check_in();
        assert!(response1.success);
        assert_eq!(response1.consecutive_days, 1);
        
        // Advance time by 2 days (skipping a day)
        advance_time_by_days(2);
        
        // Check-in after skipping a day
        let response2 = claim_daily_check_in();
        assert!(response2.success);
        assert_eq!(response2.consecutive_days, 1); // Streak reset to 1
        assert_eq!(response2.bonus_points, 0);
        
        // Verify user data was updated correctly in storage
        STORAGE.with(|storage| {
            let storage = storage.borrow();
            assert_eq!(storage.consecutive_days.get(&user).unwrap(), &1);
        });
        
        teardown();
    }
    
    // Test double check-in (same day)
    #[test]
    fn test_double_checkin() {
        setup();
        
        // Set up a test user
        let user = test_principal(1);
        set_mock_caller(user);
        
        // First check-in
        let response1 = claim_daily_check_in();
        assert!(response1.success);
        
        // Try to check-in again on the same day
        let response2 = claim_daily_check_in();
        assert!(!response2.success);
        
        teardown();
    }
    
    // Test admin functions
    #[test]
    fn test_admin_functions() {
        setup();
        
        // Set up admin and regular user principals
        let admin = test_principal(0);
        let user = test_principal(1);
        let new_admin = test_principal(2);
        
        // Test adding a new admin
        set_mock_caller(admin);
        add_admin(new_admin);
        
        // Verify new admin was added
        STORAGE.with(|storage| {
            let storage = storage.borrow();
            assert!(storage.admins.contains(&new_admin));
        });
        
        // Test updating task configuration
        set_mock_caller(new_admin);
        update_task_config(TaskConfig {
            title: "Test Task".to_string(),
            description: "Test Description".to_string(),
            base_points: 20,
            max_consecutive_bonus_days: 10,
            consecutive_days_bonus_multiplier: 3,
            enabled: true,
        });
        
        // Verify task configuration was updated
        STORAGE.with(|storage| {
            let storage = storage.borrow();
            assert_eq!(storage.task_config.title, "Test Task");
            assert_eq!(storage.task_config.base_points, 20);
        });
        
        // Test awarding points to a user
        award_points(user, 50, "Test award".to_string());
        
        // Verify points were awarded
        STORAGE.with(|storage| {
            let storage = storage.borrow();
            assert_eq!(storage.user_points.get(&user).unwrap(), &50);
            assert_eq!(storage.points_history.get(&user).unwrap().len(), 1);
            assert_eq!(storage.points_history.get(&user).unwrap()[0].amount, 50);
            assert_eq!(storage.points_history.get(&user).unwrap()[0].reason, "Test award");
        });
        
        // Test resetting a user's streak
        // First, let's give the user a streak
        set_mock_caller(user);
        claim_daily_check_in();
        
        // Verify user has a streak
        STORAGE.with(|storage| {
            let storage = storage.borrow();
            assert_eq!(storage.consecutive_days.get(&user).unwrap(), &1);
        });
        
        // Now reset the streak
        set_mock_caller(new_admin);
        reset_user_streak(user);
        
        // Verify streak was reset
        STORAGE.with(|storage| {
            let storage = storage.borrow();
            assert_eq!(storage.consecutive_days.get(&user).unwrap(), &0);
        });
        
        teardown();
    }
    
    // Test task verification
    #[test]
    fn test_task_verification() {
        setup();
        
        // Set up a test user and admin
        let admin = test_principal(0);
        let user = test_principal(1);
        
        // Set caller as admin
        set_mock_caller(admin);
        
        // Create a task verification request
        let request = TaskVerificationRequest {
            user,
            task_id: "daily_checkin".to_string(),
            timestamp: current_time_millis(),
            proof: None,
        };
        
        // Call verify_task
        let response = verify_task(request);
        
        // Verify the response
        assert!(response.success);
        assert!(response.verification_data.is_some());
        
        // Verify user data was updated
        STORAGE.with(|storage| {
            let storage = storage.borrow();
            assert!(storage.user_checkins.contains_key(&user));
            assert_eq!(storage.consecutive_days.get(&user).unwrap(), &1);
        });
        
        teardown();
    }
    
    // Test maximum consecutive bonus
    #[test]
    fn test_max_consecutive_bonus() {
        setup();
        
        // Set up a test user
        let user = test_principal(1);
        set_mock_caller(user);
        
        // Set max consecutive bonus days to 3
        STORAGE.with(|storage| {
            storage.borrow_mut().task_config.max_consecutive_bonus_days = 3;
            storage.borrow_mut().task_config.consecutive_days_bonus_multiplier = 2;
        });
        
        // Check in for 5 consecutive days
        for day in 1..=5 {
            if day > 1 {
                advance_time_by_days(1);
            }
            
            let response = claim_daily_check_in();
            assert!(response.success);
            assert_eq!(response.consecutive_days, day);
            
            // Calculate expected bonus
            let expected_bonus = if day <= 1 {
                0
            } else {
                let bonus_days = std::cmp::min(day - 1, 3);
                bonus_days * 2
            };
            
            assert_eq!(response.bonus_points, expected_bonus);
        }
        
        teardown();
    }
    
    // Test points history
    #[test]
    fn test_points_history() {
        setup();
        
        // Set up a test user and admin
        let admin = test_principal(0);
        let user = test_principal(1);
        
        // Award points as admin
        set_mock_caller(admin);
        award_points(user, 10, "First award".to_string());
        award_points(user, 20, "Second award".to_string());
        
        // Check-in as user
        set_mock_caller(user);
        claim_daily_check_in();
        
        // Get points history
        set_mock_caller(admin);
        let history = get_user_rewards(user);
        
        // Verify history
        assert_eq!(history.len(), 3); // 2 awards + 1 check-in
        
        // Check if history contains the awards
        let has_first_award = history.iter().any(|tx| tx.amount == 10 && tx.reason == "First award");
        let has_second_award = history.iter().any(|tx| tx.amount == 20 && tx.reason == "Second award");
        let has_checkin = history.iter().any(|tx| tx.amount == 10 && tx.reason.contains("Daily check-in"));
        
        assert!(has_first_award);
        assert!(has_second_award);
        assert!(has_checkin);
        
        teardown();
    }
    
    // Test removing admin
    #[test]
    fn test_remove_admin() {
        setup();
        
        // Set up admin principals
        let admin1 = test_principal(0);
        let admin2 = test_principal(2);
        
        // Add admin2
        set_mock_caller(admin1);
        add_admin(admin2);
        
        // Verify admin2 was added
        STORAGE.with(|storage| {
            let storage = storage.borrow();
            assert!(storage.admins.contains(&admin2));
        });
        
        // Remove admin2
        remove_admin(admin2);
        
        // Verify admin2 was removed
        STORAGE.with(|storage| {
            let storage = storage.borrow();
            assert!(!storage.admins.contains(&admin2));
        });
        
        teardown();
    }
    
    // Test get_checkin_status and get_my_checkin_status
    #[test]
    fn test_get_checkin_status() {
        setup();
        
        // Set up a test user and admin
        let admin = test_principal(0);
        let user = test_principal(1);
        
        // Check-in as user
        set_mock_caller(user);
        claim_daily_check_in();
        
        // Get user's own status
        let my_status = get_my_checkin_status();
        
        // Verify my_status
        let my_status_map: HashMap<String, String> = my_status.into_iter().collect();
        assert!(my_status_map.contains_key("consecutive_days"));
        assert!(my_status_map.contains_key("last_checkin"));
        assert_eq!(my_status_map.get("consecutive_days").unwrap(), "1");
        
        // Get user's status as admin
        set_mock_caller(admin);
        let admin_view = get_checkin_status(user);
        
        // Verify admin_view
        let admin_view_map: HashMap<String, String> = admin_view.into_iter().collect();
        assert!(admin_view_map.contains_key("consecutive_days"));
        assert!(admin_view_map.contains_key("last_checkin"));
        assert_eq!(admin_view_map.get("consecutive_days").unwrap(), "1");
        
        teardown();
    }
}
