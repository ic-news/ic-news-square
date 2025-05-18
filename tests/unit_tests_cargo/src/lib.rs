// Main test file for IC News Square unit tests

// Re-export modules for testing
pub mod tests;

// Export common types and utilities
pub use candid::{CandidType, Deserialize, Principal};
pub use ic_cdk::api;
pub use std::cell::RefCell;
pub use std::collections::HashMap;

// Constants used across tests
pub const SECONDS_IN_DAY: u64 = 86400;
pub const DAILY_CHECK_IN_POINTS: u64 = 10;
pub const MAX_CONSECUTIVE_BONUS_DAYS: u64 = 7;
pub const CONSECUTIVE_DAYS_BONUS_MULTIPLIER: u64 = 2;
