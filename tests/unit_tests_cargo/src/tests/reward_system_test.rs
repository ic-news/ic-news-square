use crate::*;

// Mock structures for reward system
#[derive(CandidType, Deserialize, Clone, Debug)]
struct PointsTransaction {
    pub amount: i64,
    pub reason: String,
    pub timestamp: u64,
    pub reference_id: Option<String>,
    pub points: u64,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
struct UserPointsResponse {
    pub total_points: u64,
    pub transactions: Vec<PointsTransaction>,
}

// Mock storage for rewards
thread_local! {
    static MOCK_REWARD_STORAGE: RefCell<MockRewardStorage> = RefCell::new(MockRewardStorage::default());
}

#[derive(Default, Clone)]
struct MockRewardStorage {
    user_points: HashMap<Principal, u64>,
    points_history: HashMap<Principal, Vec<PointsTransaction>>,
}

// Mock reward functions
fn mock_award_points(user: Principal, points: u64, reason: String, reference_id: Option<String>, current_time: u64) -> u64 {
    MOCK_REWARD_STORAGE.with(|storage| {
        let mut storage = storage.borrow_mut();
        
        // Update user points
        let current_points = *storage.user_points.get(&user).unwrap_or(&0);
        let new_total = current_points + points;
        storage.user_points.insert(user, new_total);
        
        // Create transaction record
        let transaction = PointsTransaction {
            amount: points as i64,
            reason,
            timestamp: current_time,
            reference_id,
            points,
        };
        
        // Add to history
        let user_history = storage.points_history.entry(user).or_insert_with(Vec::new);
        user_history.push(transaction);
        
        new_total
    })
}

fn mock_deduct_points(user: Principal, points: u64, reason: String, reference_id: Option<String>, current_time: u64) -> Result<u64, String> {
    MOCK_REWARD_STORAGE.with(|storage| {
        let mut storage = storage.borrow_mut();
        
        // Check if user has enough points
        let current_points = *storage.user_points.get(&user).unwrap_or(&0);
        
        if current_points < points {
            return Err(format!("Insufficient points. User has {} but needs {}", current_points, points));
        }
        
        // Update user points
        let new_total = current_points - points;
        storage.user_points.insert(user, new_total);
        
        // Create transaction record
        let transaction = PointsTransaction {
            amount: -(points as i64),
            reason,
            timestamp: current_time,
            reference_id,
            points,
        };
        
        // Add to history
        let user_history = storage.points_history.entry(user).or_insert_with(Vec::new);
        user_history.push(transaction);
        
        Ok(new_total)
    })
}

fn mock_get_user_points(user: Principal) -> UserPointsResponse {
    MOCK_REWARD_STORAGE.with(|storage| {
        let storage = storage.borrow();
        
        let total_points = *storage.user_points.get(&user).unwrap_or(&0);
        let transactions = storage.points_history.get(&user)
            .cloned()
            .unwrap_or_default();
        
        UserPointsResponse {
            total_points,
            transactions,
        }
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
        MOCK_REWARD_STORAGE.with(|storage| {
            *storage.borrow_mut() = MockRewardStorage::default();
        });
    }
    
    #[test]
    fn test_award_points() {
        reset_mock_storage();
        
        let user = test_principal();
        let current_time = 1_000_000;
        
        // Award points
        let new_total = mock_award_points(
            user,
            50,
            "Test award".to_string(),
            Some("test_ref_1".to_string()),
            current_time
        );
        
        assert_eq!(new_total, 50);
        
        // Check user points
        let user_points = mock_get_user_points(user);
        assert_eq!(user_points.total_points, 50);
        assert_eq!(user_points.transactions.len(), 1);
        assert_eq!(user_points.transactions[0].amount, 50);
        assert_eq!(user_points.transactions[0].reason, "Test award");
    }
    
    #[test]
    fn test_multiple_awards() {
        reset_mock_storage();
        
        let user = test_principal();
        let current_time = 1_000_000;
        
        // Award points multiple times
        mock_award_points(
            user,
            50,
            "First award".to_string(),
            Some("test_ref_1".to_string()),
            current_time
        );
        
        let new_total = mock_award_points(
            user,
            30,
            "Second award".to_string(),
            Some("test_ref_2".to_string()),
            current_time + 1000
        );
        
        assert_eq!(new_total, 80);
        
        // Check user points
        let user_points = mock_get_user_points(user);
        assert_eq!(user_points.total_points, 80);
        assert_eq!(user_points.transactions.len(), 2);
    }
    
    #[test]
    fn test_deduct_points_success() {
        reset_mock_storage();
        
        let user = test_principal();
        let current_time = 1_000_000;
        
        // Award points first
        mock_award_points(
            user,
            100,
            "Initial award".to_string(),
            None,
            current_time
        );
        
        // Deduct points
        let result = mock_deduct_points(
            user,
            30,
            "Test deduction".to_string(),
            Some("test_deduct_1".to_string()),
            current_time + 1000
        );
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 70);
        
        // Check user points
        let user_points = mock_get_user_points(user);
        assert_eq!(user_points.total_points, 70);
        assert_eq!(user_points.transactions.len(), 2);
        assert_eq!(user_points.transactions[1].amount, -30);
    }
    
    #[test]
    fn test_deduct_points_insufficient() {
        reset_mock_storage();
        
        let user = test_principal();
        let current_time = 1_000_000;
        
        // Award points first
        mock_award_points(
            user,
            20,
            "Initial award".to_string(),
            None,
            current_time
        );
        
        // Try to deduct more points than available
        let result = mock_deduct_points(
            user,
            30,
            "Test deduction".to_string(),
            Some("test_deduct_1".to_string()),
            current_time + 1000
        );
        
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Insufficient points"));
        
        // Check user points (should remain unchanged)
        let user_points = mock_get_user_points(user);
        assert_eq!(user_points.total_points, 20);
        assert_eq!(user_points.transactions.len(), 1); // Only the award transaction
    }
    
    #[test]
    fn test_transaction_history() {
        reset_mock_storage();
        
        let user = test_principal();
        let current_time = 1_000_000;
        
        // Create multiple transactions
        mock_award_points(
            user,
            50,
            "First award".to_string(),
            Some("ref_1".to_string()),
            current_time
        );
        
        mock_award_points(
            user,
            30,
            "Second award".to_string(),
            Some("ref_2".to_string()),
            current_time + 1000
        );
        
        mock_deduct_points(
            user,
            20,
            "First deduction".to_string(),
            Some("ref_3".to_string()),
            current_time + 2000
        ).unwrap();
        
        // Check transaction history
        let user_points = mock_get_user_points(user);
        assert_eq!(user_points.total_points, 60);
        assert_eq!(user_points.transactions.len(), 3);
        
        // Verify transaction details
        assert_eq!(user_points.transactions[0].amount, 50);
        assert_eq!(user_points.transactions[0].reason, "First award");
        
        assert_eq!(user_points.transactions[1].amount, 30);
        assert_eq!(user_points.transactions[1].reason, "Second award");
        
        assert_eq!(user_points.transactions[2].amount, -20);
        assert_eq!(user_points.transactions[2].reason, "First deduction");
    }
}
