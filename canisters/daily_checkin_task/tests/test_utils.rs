// Test utilities for daily check-in task canister
use candid::Principal;
use ic_cdk::api::time;
use std::cell::RefCell;
use std::collections::HashMap;

// Constants for testing
pub const TEST_SECONDS_IN_DAY: u64 = 86400;

// Mock time for testing
thread_local! {
    static MOCK_TIME: RefCell<Option<u64>> = RefCell::new(None);
}

// Mock caller for testing
thread_local! {
    static MOCK_CALLER: RefCell<Option<Principal>> = RefCell::new(None);
}

// Set mock time
pub fn set_mock_time(timestamp_nanos: u64) {
    MOCK_TIME.with(|t| {
        *t.borrow_mut() = Some(timestamp_nanos);
    });
}

// Clear mock time
pub fn clear_mock_time() {
    MOCK_TIME.with(|t| {
        *t.borrow_mut() = None;
    });
}

// Get time (real or mocked)
pub fn get_time() -> u64 {
    MOCK_TIME.with(|t| {
        t.borrow().unwrap_or_else(|| time())
    })
}

// Set mock caller
pub fn set_mock_caller(principal: Principal) {
    MOCK_CALLER.with(|c| {
        *c.borrow_mut() = Some(principal);
    });
}

// Clear mock caller
pub fn clear_mock_caller() {
    MOCK_CALLER.with(|c| {
        *c.borrow_mut() = None;
    });
}

// Get caller (real or mocked)
pub fn get_caller() -> Principal {
    MOCK_CALLER.with(|c| {
        c.borrow().unwrap_or_else(|| ic_cdk::api::caller())
    })
}

// Helper function to create a test principal
pub fn test_principal(id: u8) -> Principal {
    Principal::from_slice(&[id; 29])
}

// Helper function to get current timestamp in milliseconds
pub fn current_time_millis() -> u64 {
    get_time() / 1_000_000
}

// Helper function to get today's start timestamp in seconds
pub fn today_start_seconds() -> u64 {
    let now_seconds = current_time_millis() / 1000;
    now_seconds - (now_seconds % TEST_SECONDS_IN_DAY)
}

// Helper function to advance time by days
pub fn advance_time_by_days(days: u64) {
    let current = get_time();
    set_mock_time(current + days * TEST_SECONDS_IN_DAY * 1_000_000_000);
}

// Reset all mocks
pub fn reset_mocks() {
    clear_mock_time();
    clear_mock_caller();
}
