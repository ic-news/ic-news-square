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

// Daily check-in response for direct API calls
#[derive(CandidType, Deserialize, Clone)]
pub struct DailyCheckInResponse {
    pub success: bool,
    pub points_earned: u64,
    pub consecutive_days: u64,
    pub bonus_points: u64,
    pub total_points: u64,
    pub next_claim_available_at: u64,
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
    pub level: u64,
    pub level_up: bool,
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
    pub requirements: TaskRequirements,
}

#[derive(CandidType, Deserialize, Clone)]
pub struct TaskRequirements {
    pub min_level: u64,
    pub required_tokens: Vec<String>,
    pub required_nfts: Vec<String>,
    pub custom_requirements: Vec<String>,
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
    pub requirements: TaskRequirements,
}

#[derive(CandidType, Deserialize, Clone)]
pub struct UserRewardsResponse {
    pub points: u64,
    pub level: u64,
    pub completed_tasks: Vec<String>,
    pub points_history: Vec<PointsTransaction>,
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
