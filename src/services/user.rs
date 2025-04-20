use candid::Principal;
use ic_cdk::api::time;
use std::collections::HashSet;

use crate::auth::{is_admin, is_manager_or_admin};
use crate::models::user;
use crate::models::user::*;
use crate::models::error::{SquareError, SquareResult};
use crate::models::content::PaginationParams;
use crate::storage::{STORAGE, User, UserProfile, UserStats};
use crate::storage_main::{UserStatus as StorageUserStatus, UserRole as StorageUserRole};
use crate::storage::sharded::{SHARDED_USERS, SHARDED_USER_PROFILES, SHARDED_USER_STATS};
use crate::utils::error_handler::*;

// Helper functions for type conversion

// Convert storage user status to model user status
fn map_storage_status_to_model(status: StorageUserStatus) -> user::UserStatus {
    match status {
        StorageUserStatus::Active => user::UserStatus::Active,
        StorageUserStatus::Suspended => user::UserStatus::Suspended,
        StorageUserStatus::Banned => user::UserStatus::Banned,
        StorageUserStatus::Restricted => user::UserStatus::Restricted,
    }
}

// Convert storage user role to model user role
fn map_storage_role_to_model(role: StorageUserRole) -> user::UserRole {
    match role {
        StorageUserRole::User => user::UserRole::User,
        StorageUserRole::Admin => user::UserRole::Admin,
        StorageUserRole::Moderator => user::UserRole::Moderator,
        StorageUserRole::Creator => user::UserRole::Creator,
    }
}

// User registration and profile management
pub fn register_user(request: RegisterUserRequest, caller: Principal) -> SquareResult<()> {
    const MODULE: &str = "services::user";
    const FUNCTION: &str = "register_user";
    
    // Validate username length
    if request.username.len() < MIN_USERNAME_LENGTH || request.username.len() > MAX_USERNAME_LENGTH {
        return log_and_return(validation_error(
            &format!("Username must be between {} and {} characters", MIN_USERNAME_LENGTH, MAX_USERNAME_LENGTH),
            MODULE,
            FUNCTION
        ));
    }
    
    // Validate handle format
    let handle_regex = regex::Regex::new(HANDLE_PATTERN).unwrap();
    if !handle_regex.is_match(&request.handle) {
        return log_and_return(validation_error(
            "Handle must start with a letter and can only contain letters, numbers, and underscores",
            MODULE,
            FUNCTION
        ));
    }
    
    // Validate bio length
    if request.bio.len() > MAX_BIO_LENGTH {
        return log_and_return(validation_error(
            &format!("Bio exceeds maximum length of {} characters", MAX_BIO_LENGTH),
            MODULE,
            FUNCTION
        ));
    }
    
    STORAGE.with(|storage| {
        let mut storage = storage.borrow_mut();
        
        // Check if user already exists
        if storage.users.contains_key(&caller) {
            return log_and_return(already_exists_error(
                "User", 
                &caller.to_string(), 
                MODULE, 
                FUNCTION
            ).with_details("User already registered"));
        }
        
        // Check if username or handle is already taken
        let username_taken = storage.user_profiles.values()
            .any(|profile| profile.username == request.username);
            
        if username_taken {
            return log_and_return(already_exists_error(
                "Username", 
                &request.username, 
                MODULE, 
                FUNCTION
            ).with_details(format!("Username '{}' is already taken", request.username)));
        }
        
        let handle_taken = storage.user_profiles.values()
            .any(|profile| profile.handle == request.handle);
            
        if handle_taken {
            return log_and_return(already_exists_error(
                "Handle", 
                &request.handle, 
                MODULE, 
                FUNCTION
            ).with_details(format!("Handle '{}' is already taken", request.handle)));
        }
        
        // Create user
        let now = time() / 1_000_000;
        let user = User {
            principal: caller,
            registered_at: now,
            last_login: now,
            status: StorageUserStatus::Active,
            role: StorageUserRole::User,
        };
        
        // Create user profile
        let profile = UserProfile {
            principal: caller,
            username: request.username,
            handle: request.handle,
            bio: request.bio,
            avatar: request.avatar,
            social_links: request.social_links,
            interests: request.interests.unwrap_or_default(),
            followed_users: HashSet::new(),
            followers: HashSet::new(),
            followed_topics: HashSet::new(),
            privacy_settings: None,
            updated_at: time(),
        };
        
        // Create user stats
        let stats = UserStats {
            principal: caller,
            post_count: 0,
            article_count: 0,
            share_count: 0,
            comment_count: 0,
            likes_received: 0,
            shares_received: 0,
            views_received: 0,
        };
        
        // Store user data
        storage.users.insert(caller, user);
        storage.user_profiles.insert(caller, profile);
        storage.user_stats.insert(caller, stats);
        
        Ok(())
    })
}

pub fn update_user_profile(request: UpdateProfileRequest, caller: Principal) -> SquareResult<()> {
    const MODULE: &str = "services::user";
    const FUNCTION: &str = "update_user_profile";
    
    
    STORAGE.with(|storage| {
        let mut storage = storage.borrow_mut();
        
        // Check if user exists
        let profile = storage.user_profiles.get_mut(&caller)
            .ok_or_else(|| not_found_error(
                "UserProfile", 
                &caller.to_string(), 
                MODULE, 
                FUNCTION
            ).with_details("User profile not found"))?;
            
        // Validate username length
        if let Some(username) = &request.username {
            if username.len() < MIN_USERNAME_LENGTH || username.len() > MAX_USERNAME_LENGTH {
                return log_and_return(validation_error(
                    &format!("Username must be between {} and {} characters", MIN_USERNAME_LENGTH, MAX_USERNAME_LENGTH),
                    MODULE,
                    FUNCTION
                ));
            }
        }
        
        // Check if new username is already taken (if different from current)
        if let Some(username) = &request.username {
            if username != &profile.username {
                // Clone current username for comparison
                let current_username = profile.username.clone();
                
                // Release the mutable borrow
                let _ = profile;
                
                // Now we can safely check if username is taken
                let username_taken = storage.user_profiles.values()
                    .any(|p| p.username != current_username && p.username == *username);
                    
                if username_taken {
                    return log_and_return(already_exists_error(
                        "Username", 
                        username, 
                        MODULE, 
                        FUNCTION
                    ).with_details(format!("Username '{}' is already taken", username)));
                }
                
                // Get the profile again
                let profile = storage.user_profiles.get_mut(&caller).unwrap();
                profile.username = username.clone();
            }
        }
        
        // Validate bio length
        if let Some(bio) = &request.bio {
            if bio.len() > MAX_BIO_LENGTH {
                return log_and_return(validation_error(
                    &format!("Bio exceeds maximum length of {} characters", MAX_BIO_LENGTH),
                    MODULE,
                    FUNCTION
                ));
            }
        }
        
        // Store all updates we want to make
        let social_links_update = request.social_links.clone();
        let handle_update = request.handle.clone();
        let bio_update = request.bio.clone();
        let avatar_update = request.avatar.clone();
        
        // Update handle if provided - need to check before getting mutable reference
        if let Some(ref handle) = handle_update {
            // Validate handle format
            let handle_regex = regex::Regex::new(HANDLE_PATTERN).unwrap();
            if !handle_regex.is_match(handle) {
                return log_and_return(validation_error(
                    "Handle must start with a letter and can only contain letters, numbers, and underscores",
                    MODULE,
                    FUNCTION
                ));
            }
            
            // Check if handle is already taken (using immutable borrow)
            let handle_taken = storage.user_profiles.values()
                .any(|p| p.principal != caller && p.handle == *handle);
                
            if handle_taken {
                return log_and_return(already_exists_error(
                    "Handle", 
                    handle, 
                    MODULE, 
                    FUNCTION
                ).with_details(format!("Handle '{}' is already taken", handle)));
            }
        }
        
        // Now get mutable reference and apply all updates
        let profile = storage.user_profiles.get_mut(&caller).unwrap();
        
        // Apply bio update
        if let Some(bio) = bio_update {
            profile.bio = bio;
        }
        
        // Update avatar URL
        if let Some(avatar) = avatar_update {
            profile.avatar = avatar;
        }
        
        // Update handle if provided
        if let Some(handle) = handle_update {
            profile.handle = handle;
        }
        
        // Update social links if provided
        if let Some(links) = social_links_update {
            profile.social_links = Some(links);
        }
        
        // Update last login time
        if let Some(user) = storage.users.get_mut(&caller) {
            user.last_login = time() / 1_000_000;
        }
        
        Ok(())
    })
}

pub fn get_user_profile(principal: Principal) -> SquareResult<UserProfileResponse> {
    const MODULE: &str = "services::user";
    const FUNCTION: &str = "get_user_profile";
    
    STORAGE.with(|storage| {
        let storage = storage.borrow();
        
        // Get user
        let user = storage.users.get(&principal)
            .ok_or_else(|| not_found_error("User", &principal.to_string(), MODULE, FUNCTION))?;
            
        // Get user profile
        let profile = storage.user_profiles.get(&principal)
            .ok_or_else(|| not_found_error("UserProfile", &principal.to_string(), MODULE, FUNCTION))?;
            
        // Get user stats
        let _stats = storage.user_stats.get(&principal)
            .ok_or_else(|| not_found_error("UserStats", &principal.to_string(), MODULE, FUNCTION))?;
            
        // Create response
        let response = UserProfileResponse {
            principal,
            username: profile.username.clone(),
            handle: profile.handle.clone(),
            bio: profile.bio.clone(),
            avatar: profile.avatar.clone(),
            social_links: profile.social_links.clone().unwrap_or_default(),
            followers_count: profile.followers.len() as u64,
            following_count: profile.followed_users.len() as u64,
            registered_at: user.registered_at,
            last_login: user.last_login,
            status: map_storage_status_to_model(user.status.clone()),
            role: map_storage_role_to_model(user.role.clone()),
            is_following: false, // Default to false
        };
        
        Ok(response)
    })
}

pub fn get_user_full_profile(principal: Principal) -> SquareResult<UserResponse> {
    const MODULE: &str = "services::user";
    const FUNCTION: &str = "get_user_full_profile";
    
    STORAGE.with(|storage| {
        let storage = storage.borrow();
        
        // Get user
        let user = storage.users.get(&principal)
            .ok_or_else(|| not_found_error("User", &principal.to_string(), MODULE, FUNCTION))?;
            
        // Get user profile
        let profile = storage.user_profiles.get(&principal)
            .ok_or_else(|| SquareError::NotFound("User profile not found".to_string()))?;
            
        // Get user stats
        let stats = storage.user_stats.get(&principal)
            .ok_or_else(|| SquareError::NotFound("User stats not found".to_string()))?;
            
        // Create response
        let response = UserResponse {
            principal,
            username: profile.username.clone(),
            handle: profile.handle.clone(),
            bio: profile.bio.clone(),
            avatar: profile.avatar.clone(),
            social_links: profile.social_links.clone().unwrap_or_default(),
            interests: Vec::new(), // Add a default empty interest list
            followers_count: profile.followers.len() as u64,
            following_count: profile.followed_users.len() as u64,
            created_at: user.registered_at, // Use registered_at as created_at
            updated_at: user.last_login, // Use last_login as updated_at
            stats: UserStatsResponse {
                post_count: stats.post_count,
                article_count: stats.article_count,
                comment_count: stats.comment_count,
                like_count: stats.likes_received, // Map likes_received to like_count
                share_count: stats.shares_received, // Map shares_received to share_count
                points: 0, // Default points
                reputation: 0, // Default reputation
            },
            status: user::UserStatus::Active, // Use model UserStatus instead of storage
            role: user::UserRole::User, // Use model UserRole instead of storage
            is_following: false, // Default to false
        };
        
        Ok(response)
    })
}

// Social interactions
pub fn follow_user(request: FollowUserRequest, caller: Principal) -> SquareResult<()> {
    const MODULE: &str = "services::user";
    const FUNCTION: &str = "follow_user";
    
    if request.user_to_follow == caller {
        return log_and_return(invalid_operation_error(
            "follow_user", 
            "Cannot follow yourself", 
            MODULE, 
            FUNCTION
        ));
    }
    
    STORAGE.with(|storage| {
        let mut storage = storage.borrow_mut();
        
        // Check if target user exists
        if !storage.users.contains_key(&request.user_to_follow) {
            return log_and_return(not_found_error(
                "User", 
                &request.user_to_follow.to_string(), 
                MODULE, 
                FUNCTION
            ).with_details("User to follow not found"));
        }
        
        // Check if caller exists
        let caller_profile = storage.user_profiles.get_mut(&caller)
            .ok_or_else(|| not_found_error(
                "UserProfile", 
                &caller.to_string(), 
                MODULE, 
                FUNCTION
            ).with_details("Your profile not found"))?;
            
        // Add to followed users
        caller_profile.followed_users.insert(request.user_to_follow);
        
        // Add caller to target user's followers
        if let Some(target_profile) = storage.user_profiles.get_mut(&request.user_to_follow) {
            target_profile.followers.insert(caller);
        }
        
        Ok(())
    })
}

pub fn unfollow_user(request: FollowUserRequest, caller: Principal) -> SquareResult<()> {
    const MODULE: &str = "services::user";
    const FUNCTION: &str = "unfollow_user";
    
    STORAGE.with(|storage| {
        let mut storage = storage.borrow_mut();
        
        // Check if caller exists
        let caller_profile = storage.user_profiles.get_mut(&caller)
            .ok_or_else(|| not_found_error(
                "UserProfile", 
                &caller.to_string(), 
                MODULE, 
                FUNCTION
            ).with_details("Your profile not found"))?;
            
        // Remove from followed users
        caller_profile.followed_users.remove(&request.user_to_follow);
        
        // Remove caller from target user's followers
        if let Some(target_profile) = storage.user_profiles.get_mut(&request.user_to_follow) {
            target_profile.followers.remove(&caller);
        }
        
        Ok(())
    })
}

pub fn follow_topic(request: FollowTopicRequest, caller: Principal) -> SquareResult<()> {
    STORAGE.with(|storage| {
        let mut storage = storage.borrow_mut();
        
        // Check if caller exists
        let caller_profile = storage.user_profiles.get_mut(&caller)
            .ok_or_else(|| SquareError::NotFound("Your profile not found".to_string()))?;
            
        // Add to followed topics
        caller_profile.followed_topics.insert(request.topic);
        
        Ok(())
    })
}

pub fn unfollow_topic(request: FollowTopicRequest, caller: Principal) -> SquareResult<()> {
    STORAGE.with(|storage| {
        let mut storage = storage.borrow_mut();
        
        // Check if caller exists
        let caller_profile = storage.user_profiles.get_mut(&caller)
            .ok_or_else(|| SquareError::NotFound("Your profile not found".to_string()))?;
            
        // Remove from followed topics
        caller_profile.followed_topics.remove(&request.topic);
        
        Ok(())
    })
}

// User management (admin functions)
pub fn update_user_status(request: UserStatusUpdateRequest) -> SquareResult<()> {
    // Check if caller is admin or manager
    is_manager_or_admin()?;
    
    STORAGE.with(|storage| {
        let mut storage = storage.borrow_mut();
        
        // Check if user exists
        let user = storage.users.get_mut(&request.principal)
            .ok_or_else(|| SquareError::NotFound("User not found".to_string()))?;
            
        // Update status - convert from models::user::UserStatus to storage_main::UserStatus
        user.status = match request.status {
            user::UserStatus::Active => StorageUserStatus::Active,
            user::UserStatus::Suspended => StorageUserStatus::Suspended,
            user::UserStatus::Banned => StorageUserStatus::Banned,
            user::UserStatus::Restricted => StorageUserStatus::Restricted,
        };
        
        Ok(())
    })
}

pub fn update_user_role(request: UserRoleUpdateRequest) -> SquareResult<()> {
    // Check if caller is admin
    is_admin()?;
    
    STORAGE.with(|storage| {
        let mut storage = storage.borrow_mut();
        
        // Check if user exists
        let user = storage.users.get_mut(&request.principal)
            .ok_or_else(|| SquareError::NotFound("User not found".to_string()))?;
            
        // Update role - convert from models::user::UserRole to storage_main::UserRole
        user.role = match request.role {
            user::UserRole::User => StorageUserRole::User,
            user::UserRole::Moderator => StorageUserRole::Moderator,
            user::UserRole::Admin => StorageUserRole::Admin,
            user::UserRole::Creator => StorageUserRole::Creator,
        };
        
        Ok(())
    })
}

// Follower management functions
pub fn get_followers(principal: Principal, caller: Option<Principal>) -> SquareResult<Vec<UserSocialResponse>> {
    const MODULE: &str = "services::user";
    const FUNCTION: &str = "get_followers";
    
    
    STORAGE.with(|storage| {
        let storage = storage.borrow();
        
        // Get user profile
        let profile = storage.user_profiles.get(&principal)
            .ok_or_else(|| not_found_error(
                "UserProfile", 
                &principal.to_string(), 
                MODULE, 
                FUNCTION
            ).with_details("User profile not found"))?;
            
        // Get followers
        let mut followers = Vec::new();
        for follower_principal in &profile.followers {
            if let Some(follower_profile) = storage.user_profiles.get(follower_principal) {
                // Check if caller follows this follower
                let is_followed_by_caller = match caller {
                    Some(caller_principal) => {
                        storage.user_profiles.get(&caller_principal)
                            .map(|caller_profile| caller_profile.followed_users.contains(follower_principal))
                            .unwrap_or(false)
                    },
                    None => false,
                };
                
                followers.push(UserSocialResponse {
                    principal: *follower_principal,
                    username: follower_profile.username.clone(),
                    handle: follower_profile.handle.clone(),
                    bio: follower_profile.bio.clone(),
                    avatar: follower_profile.avatar.clone(),
                    followers_count: follower_profile.followers.len() as u64,
                    following_count: follower_profile.followed_users.len() as u64,
                    is_following: false,
                    is_followed_by_caller,
                });
            }
        }
        
        Ok(followers)
    })
}

pub fn get_following(principal: Principal, caller: Option<Principal>) -> SquareResult<Vec<UserSocialResponse>> {
    const MODULE: &str = "services::user";
    const FUNCTION: &str = "get_following";
    
    
    STORAGE.with(|storage| {
        let storage = storage.borrow();
        
        // Get user profile
        let profile = storage.user_profiles.get(&principal)
            .ok_or_else(|| not_found_error(
                "UserProfile", 
                &principal.to_string(), 
                MODULE, 
                FUNCTION
            ).with_details("User profile not found"))?;
            
        // Get followed users
        let mut following = Vec::new();
        for followed_principal in &profile.followed_users {
            if let Some(followed_profile) = storage.user_profiles.get(followed_principal) {
                // Check if caller follows this user
                let is_followed_by_caller = match caller {
                    Some(caller_principal) => {
                        if caller_principal == principal {
                            true // The user follows themselves in this context
                        } else {
                            storage.user_profiles.get(&caller_principal)
                                .map(|caller_profile| caller_profile.followed_users.contains(followed_principal))
                                .unwrap_or(false)
                        }
                    },
                    None => false,
                };
                
                following.push(UserSocialResponse {
                    principal: *followed_principal,
                    username: followed_profile.username.clone(),
                    handle: followed_profile.handle.clone(),
                    bio: followed_profile.bio.clone(),
                    avatar: followed_profile.avatar.clone(),
                    followers_count: followed_profile.followers.len() as u64,
                    following_count: followed_profile.followed_users.len() as u64,
                    is_following: false,
                    is_followed_by_caller,
                });
            }
        }
        
        Ok(following)
    })
}

// Leaderboard functions
pub fn get_user_leaderboard(pagination: PaginationParams) -> SquareResult<UserLeaderboardResponse> {
    STORAGE.with(|storage| {
        let storage = storage.borrow();
        
        // Collect all users with their points
        let mut user_items: Vec<UserLeaderboardItem> = Vec::new();
        
        for principal in storage.user_profiles.keys() {
            if let (Some(profile), Some(user_rewards)) = (storage.user_profiles.get(principal), storage.user_rewards.get(principal)) {
                // Get user stats or create default values if not found
                let (post_count, article_count) = match storage.user_stats.get(principal) {
                    Some(stats) => (stats.post_count, stats.article_count),
                    None => (0, 0), // Default values if stats not found
                };
                
                // Create leaderboard item
                user_items.push(UserLeaderboardItem {
                    principal: *principal,
                    username: profile.username.clone(),
                    handle: profile.handle.clone(),
                    avatar: profile.avatar.clone(),
                    points: user_rewards.points,
                    rank: 0,
                    last_claim_date: user_rewards.last_claim_date,
                    consecutive_daily_logins: 0,
                    article_count,
                    post_count,
                    followers_count: profile.followers.len() as u64,
                });
            }
        }
        
        // Sort by points (highest first)
        user_items.sort_by(|a, b| b.points.cmp(&a.points));
        
        // Assign ranks
        for (index, item) in user_items.iter_mut().enumerate() {
            item.rank = (index + 1) as u64;
        }
        
        // Apply pagination
        let total = user_items.len();
        let start = std::cmp::min(pagination.offset, total);
        let end = std::cmp::min(start + pagination.limit, total);
        let users = user_items[start..end].to_vec();
        
        Ok(UserLeaderboardResponse {
            users,
            total_users: total as u64,
            has_more: end < total,
            next_offset: if end < total { end as u64 } else { total as u64 },
        })
    })
}

// User verification functions
pub fn verify_user(principal: Principal) -> SquareResult<()> {
    STORAGE.with(|storage| {
        let mut storage = storage.borrow_mut();
        
        // Check if user exists
        let user = storage.users.get_mut(&principal)
            .ok_or_else(|| SquareError::NotFound("User not found".to_string()))?;
        
        // Check if user is already active (verified)
        if user.status == StorageUserStatus::Active {
            return Err(SquareError::InvalidOperation("User is already verified".to_string()));
        }
        
        // Update user status to active (verified)
        user.status = StorageUserStatus::Active;
        
        // Update in sharded storage
        let user_clone = user.clone();
        SHARDED_USERS.with(|users| {
            let mut users_store = users.borrow_mut();
            users_store.insert(principal.to_string(), user_clone);
        });
        
        Ok(())
    })
}

// Notification functions
pub fn create_notification(user_principal: Principal, notification_type: NotificationType, content: String, related_entity_id: Option<String>, related_user: Option<Principal>) -> SquareResult<()> {
    STORAGE.with(|storage| {
        let mut storage = storage.borrow_mut();
        
        // Generate notification ID
        let notification_id = format!("notification-{}-{}", user_principal.to_string(), time());
        
        // Create notification
        let notification = UserNotification {
            id: notification_id.clone(),
            user_principal,
            notification_type,
            content,
            related_entity_id,
            related_user,
            is_read: false,
            created_at: time(),
        };
        
        // Add to user's notifications
        if !storage.user_notifications.contains_key(&user_principal) {
            storage.user_notifications.insert(user_principal, Vec::new());
        }
        
        if let Some(notifications) = storage.user_notifications.get_mut(&user_principal) {
            notifications.push(notification.clone());
            
            // Sort notifications by created_at (newest first)
            notifications.sort_by(|a, b| b.created_at.cmp(&a.created_at));
            
            // Limit to 100 notifications per user
            if notifications.len() > 100 {
                *notifications = notifications[0..100].to_vec();
            }
        }
        
        Ok(())
    })
}

pub fn get_user_notifications(principal: Principal, pagination: PaginationParams) -> SquareResult<NotificationsResponse> {
    STORAGE.with(|storage| {
        let mut storage = storage.borrow_mut();
        
        // Get user's notifications
        let notifications = storage.user_notifications.entry(principal).or_insert_with(Vec::new);
        
        // Calculate total and unread counts
        let total_count = notifications.len() as u64;
        let unread_count = notifications.iter().filter(|n| !n.is_read).count() as u64;
        
        // Apply pagination
        let start = pagination.offset.min(notifications.len());
        let end = (pagination.offset + pagination.limit).min(notifications.len());
        let paginated_notifications = &notifications[start..end];
        
        // Convert to response format
        let mut notification_responses = Vec::new();
        for notification in paginated_notifications {
            let related_user_info = if let Some(user_principal) = notification.related_user {
                match get_user_social_info(user_principal, Some(principal)) {
                    Ok(info) => Some(info),
                    Err(_) => None,
                }
            } else {
                None
            };
            
            notification_responses.push(NotificationResponse {
                id: notification.id.clone(),
                notification_type: notification.notification_type.clone(),
                content: notification.content.clone(),
                related_entity_id: notification.related_entity_id.clone(),
                related_user: related_user_info,
                is_read: notification.is_read,
                created_at: notification.created_at,
            });
        }
        
        Ok(NotificationsResponse {
            notifications: notification_responses,
            total_count,
            unread_count,
            has_more: end < notifications.len(),
            next_offset: if end < notifications.len() { end as u64 } else { total_count },
        })
    })
}

pub fn mark_notification_as_read(principal: Principal, notification_id: String) -> SquareResult<()> {
    STORAGE.with(|storage| {
        let mut storage = storage.borrow_mut();
        
        // Get user's notifications
        let notifications = storage.user_notifications.get_mut(&principal)
            .ok_or_else(|| SquareError::NotFound("User notifications not found".to_string()))?;
        
        // Find and update the notification
        let notification = notifications.iter_mut()
            .find(|n| n.id == notification_id)
            .ok_or_else(|| SquareError::NotFound(format!("Notification not found: {}", notification_id)))?;
        
        notification.is_read = true;
        
        Ok(())
    })
}

pub fn mark_all_notifications_as_read(principal: Principal) -> SquareResult<()> {
    STORAGE.with(|storage| {
        let mut storage = storage.borrow_mut();
        
        // Get user's notifications
        if let Some(notifications) = storage.user_notifications.get_mut(&principal) {
            // Mark all as read
            for notification in notifications.iter_mut() {
                notification.is_read = true;
            }
        }
        
        Ok(())
    })
}

// Helper functions
// Data synchronization functions
// Privacy settings functions
pub fn update_privacy_settings(principal: Principal, privacy_settings: UserPrivacySettings) -> SquareResult<()> {
    STORAGE.with(|storage| {
        let mut storage = storage.borrow_mut();
        
        // Get user profile
        let profile = storage.user_profiles.get_mut(&principal)
            .ok_or_else(|| SquareError::NotFound("User profile not found".to_string()))?;
        
        // Update privacy settings
        profile.privacy_settings = Some(privacy_settings.clone());
        profile.updated_at = time();
        
        // Sync to sharded storage
        let profile_clone = profile.clone();
        SHARDED_USER_PROFILES.with(|profiles| {
            let mut profiles_store = profiles.borrow_mut();
            profiles_store.insert(principal.to_string(), profile_clone);
        });
        
        Ok(())
    })
}

pub fn get_privacy_settings(principal: Principal) -> SquareResult<UserPrivacySettings> {
    STORAGE.with(|storage| {
        let storage = storage.borrow();
        
        // Get user profile
        let profile = storage.user_profiles.get(&principal)
            .ok_or_else(|| SquareError::NotFound("User profile not found".to_string()))?;
        
        // Return privacy settings or default
        Ok(profile.privacy_settings.clone().unwrap_or_else(|| UserPrivacySettings {
            profile_visibility: ProfileVisibility::Public,
            content_visibility: ContentVisibility::Public,
            interaction_preferences: InteractionPreferences {
                allow_comments: true,
                allow_mentions: true,
                allow_follows: true,
                show_likes: true,
            },
            notification_preferences: NotificationPreferences {
                likes: true,
                comments: true,
                follows: true,
                mentions: true,
                shares: true,
                system: true,
            },
        }))
    })
}

pub fn sync_user_data(principal: Principal) -> SquareResult<()> {
    STORAGE.with(|storage| {
        let storage = storage.borrow();
        
        // Sync user data to sharded storage
        if let Some(user) = storage.users.get(&principal) {
            let user_clone = user.clone();
            SHARDED_USERS.with(|users| {
                let mut users_store = users.borrow_mut();
                users_store.insert(principal.to_string(), user_clone);
            });
        }
        
        // Sync user profile to sharded storage
        if let Some(profile) = storage.user_profiles.get(&principal) {
            let profile_clone = profile.clone();
            SHARDED_USER_PROFILES.with(|profiles| {
                let mut profiles_store = profiles.borrow_mut();
                profiles_store.insert(principal.to_string(), profile_clone);
            });
        }
        
        // Sync user stats to sharded storage
        if let Some(stats) = storage.user_stats.get(&principal) {
            let stats_clone = stats.clone();
            SHARDED_USER_STATS.with(|stats_store| {
                let mut stats_store = stats_store.borrow_mut();
                stats_store.insert(principal.to_string(), stats_clone);
            });
        }
        
        Ok(())
    })
}

pub fn get_user_social_info(principal: Principal, caller: Option<Principal>) -> SquareResult<UserSocialResponse> {
    const MODULE: &str = "services::user";
    const FUNCTION: &str = "get_user_social_info";
    
    
    STORAGE.with(|storage| {
        let storage = storage.borrow();
        
        // Get user profile
        let profile = match storage.user_profiles.get(&principal) {
            Some(p) => p,
            None => return log_and_return(not_found_error(
                "UserProfile", 
                &principal.to_string(), 
                MODULE, 
                FUNCTION
            ).with_details("User profile not found")),
        };
        
        // Check if caller follows this user
        let is_followed_by_caller = if let Some(caller_principal) = caller {
            storage.user_profiles.get(&caller_principal)
                .map(|caller_profile| caller_profile.followed_users.contains(&principal))
                .unwrap_or(false)
        } else {
            false
        };
        
        // Create response
        let response = UserSocialResponse {
            principal,
            username: profile.username.clone(),
            handle: profile.handle.clone(),
            bio: profile.bio.clone(),
            avatar: profile.avatar.clone(),
            followers_count: profile.followers.len() as u64,
            following_count: profile.followed_users.len() as u64,
            is_following: false, // Default to false, can be modified based on actual needs
            is_followed_by_caller,
        };
        
        Ok(response)
    })
}
