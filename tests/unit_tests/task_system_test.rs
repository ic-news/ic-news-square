use candid::{CandidType, Deserialize, Principal};
use std::cell::RefCell;
use std::collections::HashMap;

// Mock structures for task system
#[derive(CandidType, Deserialize, Clone, Debug)]
struct TaskVerificationRequest {
    pub user: Principal,
    pub task_id: String,
    pub metadata: HashMap<String, String>,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
struct TaskVerificationData {
    pub task_id: String,
    pub points_earned: u64,
    pub completion_timestamp: u64,
    pub metadata: HashMap<String, String>,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
struct TaskVerificationResponse {
    pub success: bool,
    pub message: String,
    pub verification_data: Option<TaskVerificationData>,
}

#[derive(Default, Clone)]
struct TaskConfig {
    pub base_points: u64,
    pub max_consecutive_bonus_days: u64,
    pub consecutive_days_bonus_multiplier: u64,
}

// Mock storage for tasks
thread_local! {
    static MOCK_TASK_STORAGE: RefCell<MockTaskStorage> = RefCell::new(MockTaskStorage::default());
}

#[derive(Default, Clone)]
struct MockTaskStorage {
    user_task_completions: HashMap<(Principal, String), u64>, // (user, task_id) -> timestamp
    task_config: HashMap<String, TaskConfig>,
    user_points: HashMap<Principal, u64>,
}

// Constants
const SECONDS_IN_DAY: u64 = 86400;

// Mock verify_task function
fn mock_verify_task(request: TaskVerificationRequest, current_time: u64) -> Result<TaskVerificationResponse, String> {
    let user = request.user;
    let task_id = &request.task_id;
    
    // Calculate the start of the current day in UTC time
    let today_start = (current_time / SECONDS_IN_DAY) * SECONDS_IN_DAY;
    
    // Check if the user has already completed this task today
    let already_completed = MOCK_TASK_STORAGE.with(|storage| {
        let storage = storage.borrow();
        
        if let Some(last_completion) = storage.user_task_completions.get(&(user, task_id.clone())) {
            // Check if the last completion time is within today's range
            let last_completion_day_start = *last_completion - (*last_completion % SECONDS_IN_DAY);
            let completed_today = last_completion_day_start == today_start;
            
            completed_today
        } else {
            false
        }
    });
    
    if already_completed {
        return Err(format!("Already completed task '{}' today", task_id));
    }
    
    // Process the task completion
    let points_earned = process_task_completion(user, task_id.clone(), current_time, today_start);
    
    // Create metadata
    let mut metadata = HashMap::new();
    metadata.insert("completion_time".to_string(), current_time.to_string());
    metadata.insert("next_available_at".to_string(), (today_start + SECONDS_IN_DAY).to_string());
    
    // Create verification data
    let verification_data = TaskVerificationData {
        task_id: task_id.clone(),
        points_earned,
        completion_timestamp: current_time,
        metadata,
    };
    
    // Return successful response
    Ok(TaskVerificationResponse {
        success: true,
        message: format!("Task '{}' completed successfully", task_id),
        verification_data: Some(verification_data),
    })
}

// Process task completion
fn process_task_completion(user: Principal, task_id: String, now: u64, today_start: u64) -> u64 {
    MOCK_TASK_STORAGE.with(|storage| {
        let mut storage = storage.borrow_mut();
        
        // Get task configuration or use default
        let config = storage.task_config.get(&task_id).cloned().unwrap_or_default();
        let points = if config.base_points > 0 { config.base_points } else { 10 }; // Default 10 points
        
        // Update storage
        storage.user_task_completions.insert((user, task_id), today_start);
        
        // Update user points
        let current_points = *storage.user_points.get(&user).unwrap_or(&0);
        storage.user_points.insert(user, current_points + points);
        
        points
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
        MOCK_TASK_STORAGE.with(|storage| {
            *storage.borrow_mut() = MockTaskStorage::default();
        });
    }
    
    // Helper function to set up a task configuration
    fn setup_task_config(task_id: &str, base_points: u64) {
        MOCK_TASK_STORAGE.with(|storage| {
            let mut storage = storage.borrow_mut();
            let mut config = TaskConfig::default();
            config.base_points = base_points;
            storage.task_config.insert(task_id.to_string(), config);
        });
    }
    
    #[test]
    fn test_first_task_completion() {
        reset_mock_storage();
        setup_task_config("test_task", 15);
        
        let user = test_principal();
        let current_time = 1_000_000 * SECONDS_IN_DAY;
        
        let request = TaskVerificationRequest {
            user,
            task_id: "test_task".to_string(),
            metadata: HashMap::new(),
        };
        
        let result = mock_verify_task(request, current_time);
        
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.success, true);
        assert_eq!(response.verification_data.unwrap().points_earned, 15);
    }
    
    #[test]
    fn test_duplicate_task_completion() {
        reset_mock_storage();
        setup_task_config("test_task", 15);
        
        let user = test_principal();
        let current_time = 1_000_000 * SECONDS_IN_DAY;
        
        let request = TaskVerificationRequest {
            user,
            task_id: "test_task".to_string(),
            metadata: HashMap::new(),
        };
        
        // First completion should succeed
        let first_result = mock_verify_task(request.clone(), current_time);
        assert!(first_result.is_ok());
        
        // Second completion on the same day should fail
        let second_result = mock_verify_task(request, current_time + 1000);
        assert!(second_result.is_err());
        assert!(second_result.unwrap_err().contains("Already completed task"));
    }
    
    #[test]
    fn test_different_task_completion() {
        reset_mock_storage();
        setup_task_config("task_1", 10);
        setup_task_config("task_2", 20);
        
        let user = test_principal();
        let current_time = 1_000_000 * SECONDS_IN_DAY;
        
        // Complete task 1
        let request1 = TaskVerificationRequest {
            user,
            task_id: "task_1".to_string(),
            metadata: HashMap::new(),
        };
        let result1 = mock_verify_task(request1, current_time);
        assert!(result1.is_ok());
        
        // Complete task 2 (should succeed as it's a different task)
        let request2 = TaskVerificationRequest {
            user,
            task_id: "task_2".to_string(),
            metadata: HashMap::new(),
        };
        let result2 = mock_verify_task(request2, current_time);
        assert!(result2.is_ok());
        
        // Check points
        let total_points = MOCK_TASK_STORAGE.with(|storage| {
            let storage = storage.borrow();
            *storage.user_points.get(&user).unwrap_or(&0)
        });
        
        assert_eq!(total_points, 30); // 10 + 20
    }
    
    #[test]
    fn test_next_day_task_completion() {
        reset_mock_storage();
        setup_task_config("test_task", 15);
        
        let user = test_principal();
        let day1_time = 1_000_000 * SECONDS_IN_DAY;
        let day2_time = day1_time + SECONDS_IN_DAY;
        
        let request = TaskVerificationRequest {
            user,
            task_id: "test_task".to_string(),
            metadata: HashMap::new(),
        };
        
        // Complete task on day 1
        let day1_result = mock_verify_task(request.clone(), day1_time);
        assert!(day1_result.is_ok());
        
        // Complete task on day 2 (should succeed as it's a new day)
        let day2_result = mock_verify_task(request, day2_time);
        assert!(day2_result.is_ok());
        
        // Check points
        let total_points = MOCK_TASK_STORAGE.with(|storage| {
            let storage = storage.borrow();
            *storage.user_points.get(&user).unwrap_or(&0)
        });
        
        assert_eq!(total_points, 30); // 15 + 15
    }
}
