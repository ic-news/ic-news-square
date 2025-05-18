use candid::Principal;
use std::collections::{HashSet, HashMap};
use std::borrow::{Borrow, BorrowMut};

use crate::models::user::*;
use crate::models::error::{SquareError, SquareResult};
use crate::models::notification::NotificationType;
use crate::storage::{STORAGE};
use crate::utils::error_handler::*;

// Social interactions
pub fn follow_user(request: FollowUserRequest, caller: Principal) -> SquareResult<()> {
    const MODULE: &str = "services::user::social";
    const FUNCTION: &str = "follow_user";
    
    // Get target user principal
    let target_principal = request.user_to_follow;
    
    // Prevent self-follow
    if target_principal == caller {
        return log_and_return(validation_error(
            "Cannot follow yourself", 
            MODULE, 
            FUNCTION
        ));
    }
    
    // Check if target user exists
    let target_exists = STORAGE.with(|storage| {
        let store = storage.borrow();
        store.users.contains_key(&target_principal)
    });
    
    if !target_exists {
        return log_and_return(not_found_error(
            "User", 
            &target_principal.to_string(), 
            MODULE, 
            FUNCTION
        ).with_details("Target user not found"));
    }
    
    // Update follower's profile
    STORAGE.with(|storage| {
        let mut store = storage.borrow_mut();
        
        if let Some(profiles) = &mut store.user_profiles {
            // Update follower's profile
            if let Some(follower_profile) = profiles.get_mut(&caller) {
                follower_profile.followed_users.insert(target_principal);
                follower_profile.following_count = follower_profile.followed_users.len() as u64;
            }
            
            // Update target's profile
            if let Some(target_profile) = profiles.get_mut(&target_principal) {
                target_profile.followers.insert(caller);
                target_profile.followers_count = target_profile.followers.len() as u64;
            }
        }
    });
    
    // Create notification for target user
    let follower_username = STORAGE.with(|storage| {
        let store = storage.borrow();
        if let Some(profiles) = &store.user_profiles {
            if let Some(profile) = profiles.get(&caller) {
                profile.username.clone()
            } else {
                caller.to_string()
            }
        } else {
            caller.to_string()
        }
    });
    
    let notification_content = format!("{} started following you", follower_username);
    super::notification::create_notification(
        target_principal, 
        NotificationType::Follow, 
        notification_content, 
        None, 
        Some(caller)
    )?;
    
    Ok(())
}

pub fn unfollow_user(request: FollowUserRequest, caller: Principal) -> SquareResult<()> {
    const MODULE: &str = "services::user::social";
    const FUNCTION: &str = "unfollow_user";
    
    // Get target user principal
    let target_principal = request.user_to_follow;
    
    // Update follower's profile
    STORAGE.with(|storage| {
        let mut store = storage.borrow_mut();
        
        if let Some(profiles) = &mut store.user_profiles {
            // Update follower's profile
            if let Some(follower_profile) = profiles.get_mut(&caller) {
                follower_profile.followed_users.remove(&target_principal);
                follower_profile.following_count = follower_profile.followed_users.len() as u64;
            }
            
            // Update target's profile
            if let Some(target_profile) = profiles.get_mut(&target_principal) {
                target_profile.followers.remove(&caller);
                target_profile.followers_count = target_profile.followers.len() as u64;
            }
        }
    });
    
    Ok(())
}

pub fn follow_topic(request: FollowTopicRequest, caller: Principal) -> SquareResult<()> {
    const MODULE: &str = "services::user::social";
    const FUNCTION: &str = "follow_topic";
    
    // Update user profile
    STORAGE.with(|storage| {
        let mut store = storage.borrow_mut();
        
        if let Some(profiles) = &mut store.user_profiles {
            if let Some(profile) = profiles.get_mut(&caller) {
                profile.followed_topics.insert(request.topic.clone());
            }
        }
    });
    
    Ok(())
}

pub fn unfollow_topic(request: FollowTopicRequest, caller: Principal) -> SquareResult<()> {
    const MODULE: &str = "services::user::social";
    const FUNCTION: &str = "unfollow_topic";
    
    // Update user profile
    STORAGE.with(|storage| {
        let mut store = storage.borrow_mut();
        
        if let Some(profiles) = &mut store.user_profiles {
            if let Some(profile) = profiles.get_mut(&caller) {
                profile.followed_topics.remove(&request.topic);
            }
        }
    });
    
    Ok(())
}

// Follower management functions
pub fn get_followers(user_identifier: String, caller: Option<Principal>) -> SquareResult<Vec<UserSocialResponse>> {
    const MODULE: &str = "services::user::social";
    const FUNCTION: &str = "get_followers";
    
    // Parse user identifier
    let principal = if let Ok(principal) = Principal::from_text(&user_identifier) {
        principal
    } else {
        // Try to find user by handle
        super::utils::find_user_by_handle(&user_identifier)?
    };
    
    // Get user profile
    let profile_result = STORAGE.with(|storage| {
        let store = storage.borrow();
        if let Some(profiles) = &store.user_profiles {
            profiles.get(&principal).cloned()
        } else {
            None
        }
    });
    
    let profile = match profile_result {
        Some(profile) => profile,
        None => return log_and_return(not_found_error(
            "UserProfile", 
            &principal.to_string(), 
            MODULE, 
            FUNCTION
        ).with_details("User profile not found")),
    };
    
    // Get followers
    let mut followers = Vec::new();
    
    for follower_principal in &profile.followers {
        let social_info = get_user_social_info(follower_principal.to_string(), caller)?;
        followers.push(social_info);
    }
    
    Ok(followers)
}

pub fn get_following(user_identifier: String, caller: Option<Principal>) -> SquareResult<Vec<UserSocialResponse>> {
    const MODULE: &str = "services::user::social";
    const FUNCTION: &str = "get_following";
    
    // Parse user identifier
    let principal = if let Ok(principal) = Principal::from_text(&user_identifier) {
        principal
    } else {
        // Try to find user by handle
        super::utils::find_user_by_handle(&user_identifier)?
    };
    
    // Get user profile
    let profile_result = STORAGE.with(|storage| {
        let store = storage.borrow();
        if let Some(profiles) = &store.user_profiles {
            profiles.get(&principal).cloned()
        } else {
            None
        }
    });
    
    let profile = match profile_result {
        Some(profile) => profile,
        None => return log_and_return(not_found_error(
            "UserProfile", 
            &principal.to_string(), 
            MODULE, 
            FUNCTION
        ).with_details("User profile not found")),
    };
    
    // Get followed users
    let mut following = Vec::new();
    
    for followed_principal in &profile.followed_users {
        let social_info = get_user_social_info(followed_principal.to_string(), caller)?;
        following.push(social_info);
    }
    
    Ok(following)
}

pub fn get_user_social_info(user_identifier: String, caller: Option<Principal>) -> SquareResult<UserSocialResponse> {
    const MODULE: &str = "services::user::social";
    const FUNCTION: &str = "get_user_social_info";
    
    // Parse user identifier
    let principal = if let Ok(principal) = Principal::from_text(&user_identifier) {
        principal
    } else {
        // Try to find user by handle
        super::utils::find_user_by_handle(&user_identifier)?
    };
    
    // Get user profile
    let profile_result = STORAGE.with(|storage| {
        let store = storage.borrow();
        if let Some(profiles) = &store.user_profiles {
            profiles.get(&principal).cloned()
        } else {
            None
        }
    });
    
    let profile = match profile_result {
        Some(profile) => profile,
        None => return log_and_return(not_found_error(
            "UserProfile", 
            &principal.to_string(), 
            MODULE, 
            FUNCTION
        ).with_details("User profile not found")),
    };
    
    // Check if caller is following this user
    let is_following = if let Some(caller_principal) = caller {
        STORAGE.with(|storage| {
            let store = storage.borrow();
            if let Some(profiles) = &store.user_profiles {
                if let Some(caller_profile) = profiles.get(&caller_principal) {
                    caller_profile.followed_users.contains(&principal)
                } else {
                    false
                }
            } else {
                false
            }
        })
    } else {
        false
    };
    
    Ok(UserSocialResponse {
        interests: profile.interests,
        is_followed_by_caller: is_following,  // They are the same in this context
        principal,
        username: profile.username,
        handle: profile.handle,
        avatar: profile.avatar,
        bio: profile.bio,
        followers_count: profile.followers_count,
        following_count: profile.following_count,
        is_following,
    })
}