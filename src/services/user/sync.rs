use candid::Principal;
use std::collections::{HashSet, HashMap};
use std::borrow::{Borrow, BorrowMut};
use ic_cdk::api::time;

use crate::models::user::*;
use crate::models::error::{SquareError, SquareResult};
use crate::storage::{STORAGE};
use crate::utils::error_handler::*;

// Data synchronization functions

// This function synchronizes data for a specific user
// It's a public interface to the synchronize_specific_user_data function
pub fn sync_user_data(principal: Principal) -> SquareResult<()> {
    synchronize_specific_user_data(principal)
}

// This function synchronizes all user data across all storage locations
// It ensures that all necessary data structures exist for each user
// Optimized for large-scale systems with batch processing
pub fn synchronize_all_user_data() -> SquareResult<()> {
    const MODULE: &str = "services::user::sync";
    const FUNCTION: &str = "synchronize_all_user_data";
    
    // Get all user principals from main storage
    let principals = STORAGE.with(|storage| {
        let store = storage.borrow();
        store.users.keys().cloned().collect::<Vec<Principal>>()
    });
    
    // Synchronize data for each user
    for principal in principals {
        match synchronize_specific_user_data(principal) {
            Ok(_) => (),
            Err(e) => {
                ic_cdk::println!("[{}::{}] Error synchronizing data for user {}: {}", 
                    MODULE, FUNCTION, principal, e);
            }
        }
    }
    
    Ok(())
}

// Helper function to synchronize data for a specific user
// Optimized for performance and error handling
pub fn synchronize_specific_user_data(principal: Principal) -> SquareResult<()> {
    const MODULE: &str = "services::user::sync";
    const FUNCTION: &str = "synchronize_specific_user_data";
    
    // Check if user exists in main storage
    let user_exists = STORAGE.with(|storage| {
        let store = storage.borrow();
        store.users.contains_key(&principal)
    });
    
    if !user_exists {
        return log_and_return(not_found_error(
            "User", 
            &principal.to_string(), 
            MODULE, 
            FUNCTION
        ).with_details("User not found in main storage"));
    }
    
    // Check if user profile exists in main storage
    let profile_exists = STORAGE.with(|storage| {
        let store = storage.borrow();
        if let Some(profiles) = &store.user_profiles {
            profiles.contains_key(&principal)
        } else {
            false
        }
    });
    
    if !profile_exists {
        // Create default profile
        let now = time() / 1_000_000;
        let profile = UserProfile {
            principal,
            username: format!("user_{}", principal.to_string().chars().take(8).collect::<String>()),
            handle: format!("user_{}", principal.to_string().chars().take(8).collect::<String>()),
            bio: String::new(),
            avatar: String::new(),
            social_links: Vec::new(),
            interests: Vec::new(),
            followed_users: HashSet::new(),
            followers: HashSet::new(),
            followed_topics: HashSet::new(),
            privacy_settings: None,
            updated_at: now,
            created_at: now,
            followers_count: 0,
            following_count: 0,
        };
        
        // Store profile in main storage
        STORAGE.with(|storage| {
            let mut store = storage.borrow_mut();
            
            // Create profiles map if it doesn't exist
            if store.user_profiles.is_none() {
                store.user_profiles = Some(HashMap::new());
            }
            
            // Store profile
            if let Some(profiles) = &mut store.user_profiles {
                profiles.insert(principal, profile);
            }
        });
        
        ic_cdk::println!("[{}::{}] Created default profile for user {}", 
            MODULE, FUNCTION, principal);
    }
    
    // Check if user stats exists in main storage
    let stats_exists = STORAGE.with(|storage| {
        let store = storage.borrow();
        if let Some(stats) = &store.user_stats {
            stats.contains_key(&principal)
        } else {
            false
        }
    });
    
    if !stats_exists {
        // Create default stats
        let stats = UserStats {
            principal,
            post_count: 0,
            comment_count: 0,
            like_count: 0,
            points: 0,
            reputation: 0,
        };
        
        // Store stats in main storage
        STORAGE.with(|storage| {
            let mut store = storage.borrow_mut();
            
            // Create stats map if it doesn't exist
            if store.user_stats.is_none() {
                store.user_stats = Some(HashMap::new());
            }
            
            // Store stats
            if let Some(user_stats) = &mut store.user_stats {
                user_stats.insert(principal, stats);
            }
        });
        
        ic_cdk::println!("[{}::{}] Created default stats for user {}", 
            MODULE, FUNCTION, principal);
    }
    
    // Fix followers and following counts
    STORAGE.with(|storage| {
        let mut store = storage.borrow_mut();
        
        if let Some(profiles) = &mut store.user_profiles {
            if let Some(profile) = profiles.get_mut(&principal) {
                // Update followers count
                profile.followers_count = profile.followers.len() as u64;
                
                // Update following count
                profile.following_count = profile.followed_users.len() as u64;
            }
        }
    });
    
    Ok(())
}

// 调试函数：修复用户数据不一致问题
pub fn debug_fix_user_data(principal_str: String) -> SquareResult<bool> {
    const MODULE: &str = "services::user::sync";
    const FUNCTION: &str = "debug_fix_user_data";
    
    // Parse principal
    let principal = match Principal::from_text(&principal_str) {
        Ok(p) => p,
        Err(_) => return log_and_return(validation_error(
            "Invalid principal format", 
            MODULE, 
            FUNCTION
        )),
    };
    
    // Check if user exists in main storage
    let user_exists = STORAGE.with(|storage| {
        let store = storage.borrow();
        store.users.contains_key(&principal)
    });
    
    if !user_exists {
        return log_and_return(not_found_error(
            "User", 
            &principal.to_string(), 
            MODULE, 
            FUNCTION
        ).with_details("User not found in main storage"));
    }
    
    // Synchronize user data
    synchronize_specific_user_data(principal)?;
    
    Ok(true)
}