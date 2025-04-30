use candid::{CandidType, Deserialize, Principal};
use std::collections::HashMap;

// Points transaction record
#[derive(CandidType, Deserialize, Clone)]
pub struct PointsTransaction {
    pub amount: i64,
    pub reason: String,
    pub timestamp: u64,
    pub reference_id: Option<String>,
}

// Task configuration
#[derive(CandidType, Deserialize, Clone, Default)]
pub struct TaskConfig {
    pub title: String,
    pub description: String,
    pub base_points: u64,
    pub max_consecutive_bonus_days: u64,
    pub consecutive_days_bonus_multiplier: u64,
    pub enabled: bool,
}

// Task verification request from the main canister
#[derive(CandidType, Deserialize, Clone)]
pub struct TaskVerificationRequest {
    pub user: Principal,
    pub task_id: String,
    pub timestamp: u64,
    pub proof: Option<String>,
}

// Verification data to return to the main canister
#[derive(CandidType, Deserialize, Clone)]
pub struct VerificationData {
    pub task_id: String,
    pub points_earned: u64,
    pub completion_timestamp: u64,
    pub metadata: HashMap<String, String>,
}

// Task verification response to the main canister
#[derive(CandidType, Deserialize, Clone)]
pub struct TaskVerificationResponse {
    pub success: bool,
    pub message: String,
    pub verification_data: Option<VerificationData>,
}

// Task related DTOs
#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct TaskDefinition {
    pub id: String,
    pub title: String,
    pub description: String,
    pub points: u64,
    pub task_type: TaskType,
    pub completion_criteria: String,
    pub expiration_time: Option<u64>,
    pub created_at: u64,
    pub updated_at: u64,
    pub is_active: bool,
    pub requirements: Option<TaskRequirements>,
    pub canister_id: Principal,
}

#[derive(CandidType, Deserialize, Clone)]
pub struct CompleteTaskRequest {
    pub task_id: String,
    pub proof: Option<String>,
}

#[derive(CandidType, Deserialize, Clone)]
pub struct TaskCompletionResponse {
    pub success: bool,
    pub points_earned: u64,
    pub total_points: u64,
    pub message: String,
}

#[derive(CandidType, Deserialize, Clone)]
pub struct TaskResponse {
    pub id: String,
    pub title: String,
    pub description: String,
    pub points: u64,
    pub task_type: TaskType,
    pub completion_criteria: String,
    pub is_completed: bool,
    pub expiration_time: Option<u64>,
    pub created_at: u64,
}

#[derive(CandidType, Deserialize, Clone)]
pub struct AwardPointsRequest {
    pub principal: Principal,
    pub points: u64,
    pub reason: String,
    pub reference_id: Option<String>,
}

#[derive(CandidType, Deserialize, Clone)]
pub struct CreateTaskRequest {
    pub id: String,
    pub title: String,
    pub description: String,
    pub points_reward: u64,
    pub task_type: TaskType,
    pub canister_id: Principal,
    pub start_time: Option<u64>,
    pub end_time: Option<u64>,
    pub completion_criteria: String,
    pub requirements: Option<TaskRequirements>,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct TaskRequirements {
    // Social interaction requirements
    pub social_interaction: Option<SocialInteractionRequirement>,
    
    // User qualification requirements
    pub required_tokens: Option<Vec<String>>,
    pub required_nfts: Option<Vec<String>>,
    
    // Login streak requirements
    pub login_streak: Option<LoginStreakRequirement>,
    
    // Custom requirements
    pub custom_requirements: Option<Vec<String>>,
    
    // Content creation requirements
    pub content_creation: Option<ContentCreationRequirement>
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct SocialInteractionRequirement {
    pub like_count: Option<u64>,
    pub follow_count: Option<u64>
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct LoginStreakRequirement {
    pub days_required: u64
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct ContentCreationRequirement {
    pub comment_count: Option<u64>,
    pub post_count: Option<u64>,
    pub required_hashtags: Option<Vec<String>>
}

#[derive(CandidType, Deserialize, Clone)]
pub struct UpdateTaskRequest {
    pub id: String,
    pub title: String,
    pub description: String,
    pub points_reward: u64,
    pub task_type: TaskType,
    pub completion_criteria: String,
    pub canister_id: Principal,
    pub start_time: Option<u64>,
    pub end_time: Option<u64>,
    pub requirements: Option<TaskRequirements>,
}


// Generic value type for flexible data representation
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

// Type alias for a more flexible rewards response
#[derive(CandidType, Deserialize, Clone)]
pub struct UserRewardsResponse(pub HashMap<String, Value>);

// Implementation to provide convenient methods for the HashMap wrapper
impl UserRewardsResponse {
    // Create a new empty response
    pub fn new() -> Self {
        UserRewardsResponse(HashMap::new())
    }
    
    // Add a key-value pair
    pub fn insert(&mut self, key: String, value: Value) -> Option<Value> {
        self.0.insert(key, value)
    }
    
    // Get a value by key
    pub fn get(&self, key: &str) -> Option<&Value> {
        self.0.get(key)
    }
    
    // Check if the response contains a key
    pub fn contains_key(&self, key: &str) -> bool {
        self.0.contains_key(key)
    }
    
    // Get the underlying HashMap
    pub fn inner(&self) -> &HashMap<String, Value> {
        &self.0
    }
    
    // Get a mutable reference to the underlying HashMap
    pub fn inner_mut(&mut self) -> &mut HashMap<String, Value> {
        &mut self.0
    }
}

// Enums
#[derive(CandidType, Deserialize, Clone, PartialEq, Debug)]
pub enum TaskType {
    Daily,
    Weekly,
    Monthly,
    OneTime,
    Special
}

// Constants
pub const SECONDS_IN_DAY: u64 = 86400;
pub const DAILY_CHECK_IN_POINTS: u64 = 10;
pub const MAX_CONSECUTIVE_BONUS_DAYS: u64 = 7;
pub const CONSECUTIVE_DAYS_BONUS_MULTIPLIER: u64 = 2;
