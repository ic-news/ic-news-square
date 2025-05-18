use candid::Principal;
use std::collections::HashMap;
use std::borrow::Borrow;

use crate::models::user::*;
use crate::models::error::{SquareError, SquareResult};
use crate::models::content::PaginationParams;
use crate::storage::{STORAGE, UserStatus as StorageUserStatus, UserRole as StorageUserRole};
use crate::utils::error_handler::*;

// Convert storage user status to model user status
pub fn map_storage_status_to_model(status: StorageUserStatus) -> UserStatus {
    match status {
        StorageUserStatus::Active => UserStatus::Active,
        StorageUserStatus::Suspended => UserStatus::Suspended,
        StorageUserStatus::Banned => UserStatus::Banned,
        StorageUserStatus::Restricted => UserStatus::Restricted,
    }
}

// Convert storage user role to model user role
pub fn map_storage_role_to_model(role: StorageUserRole) -> UserRole {
    match role {
        StorageUserRole::User => UserRole::User,
        StorageUserRole::Admin => UserRole::Admin,
        StorageUserRole::Moderator => UserRole::Moderator,
        StorageUserRole::Creator => UserRole::Creator,
    }
}

// Find user principal by handle
pub fn find_user_by_handle(handle: &str) -> SquareResult<Principal> {
    const MODULE: &str = "services::user::utils";
    const FUNCTION: &str = "find_user_by_handle";
    
    // Search in main storage
    let principal_result = STORAGE.with(|storage| {
        let store = storage.borrow();
        if let Some(profiles) = &store.user_profiles {
            for (principal, profile) in profiles {
                if profile.handle == handle {
                    return Some(*principal);
                }
            }
        }
        None
    });
    
    match principal_result {
        Some(principal) => Ok(principal),
        None => log_and_return(not_found_error(
            "User", 
            handle, 
            MODULE, 
            FUNCTION
        ).with_details(format!("No user found with handle: {}", handle))),
    }
}

// Get user leaderboard
pub fn get_user_leaderboard(pagination: PaginationParams) -> SquareResult<UserLeaderboardResponse> {
    const MODULE: &str = "services::user::utils";
    const FUNCTION: &str = "get_user_leaderboard";
    
    let mut users: Vec<UserLeaderboardItem> = Vec::new();
    
    // Get user stats from main storage
    STORAGE.with(|storage| {
        let store = storage.borrow();
        
        if let Some(stats) = &store.user_stats {
            // Convert stats to leaderboard entries
            for (principal, stat) in stats {
                // Get user profile
                let profile = if let Some(profiles) = &store.user_profiles {
                    profiles.get(principal).cloned()
                } else {
                    None
                };
                
                if let Some(profile) = profile {
                    let post_count = store.posts.values().filter(|post| post.author == *principal).count() as u64;
                    users.push(UserLeaderboardItem {
                        principal: *principal,
                        username: profile.username,
                        handle: profile.handle,
                        avatar: profile.avatar,
                        rank: 0, // Will be set after sorting
                        post_count: post_count,
                        last_claim_date: 0, // TODO: Get from daily check-in
                        consecutive_daily_logins: 0, // TODO: Get from daily check-in
                        followers_count: profile.followers_count,
                        comment_count: stat.comment_count,
                        like_count: stat.like_count,
                        reputation: stat.reputation,
                    });
                }
            }
        }
    });
    
    // Sort users by reputation (descending)
    users.sort_by(|a, b| b.reputation.cmp(&a.reputation));
    
    // Set ranks based on reputation
    for (i, user) in users.iter_mut().enumerate() {
        user.rank = (i + 1) as u64;
    }
    
    // Apply pagination
    let total = users.len() as u64;
    let start = pagination.offset.unwrap_or(0);
    let limit = pagination.limit.unwrap_or(10);
    let end = std::cmp::min(start + limit, users.len());
    
    // Get paginated users
    let paginated_users = if start < users.len() {
        users[start..end].to_vec()
    } else {
        Vec::new()
    };
    
    Ok(UserLeaderboardResponse {
        users: paginated_users,
        total_users: total,
        has_more: (start + limit) < users.len(),
        next_offset: if (start + limit) < users.len() {
            (start + limit) as u64
        } else {
            start as u64
        },
    })
}