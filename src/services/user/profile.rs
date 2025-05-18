use candid::Principal;
use ic_cdk::api::time;
use std::collections::{HashSet, HashMap};
use std::borrow::{Borrow, BorrowMut};

use crate::models::user::*;
use crate::models::error::{SquareError, SquareResult};
use crate::storage::{STORAGE};
use crate::utils::error_handler::*;
use super::utils::{map_storage_status_to_model, map_storage_role_to_model};

// User registration and profile management
pub fn register_user(request: RegisterUserRequest, caller: Principal) -> SquareResult<()> {
    const MODULE: &str = "services::user::profile";
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
    
    // Check if user already exists in main storage
    let user_exists = STORAGE.with(|storage| {
        let store = storage.borrow();
        store.users.contains_key(&caller)
    });
    
    if user_exists {
        return log_and_return(already_exists_error(
            "User", 
            &caller.to_string(), 
            MODULE, 
            FUNCTION
        ).with_details("User already registered"));
    }
    
    // Check if username or handle is already taken in main storage
    let username_taken = STORAGE.with(|storage| {
        let store = storage.borrow();
        if let Some(profiles) = &store.user_profiles {
            profiles.values().any(|profile| profile.username == request.username)
        } else {
            false
        }
    });
        
    if username_taken {
        return log_and_return(already_exists_error(
            "Username", 
            &request.username, 
            MODULE, 
            FUNCTION
        ).with_details(format!("Username '{}' is already taken", request.username)));
    }
    
    let handle_taken = STORAGE.with(|storage| {
        let store = storage.borrow();
        if let Some(profiles) = &store.user_profiles {
            profiles.values().any(|profile| profile.handle == request.handle)
        } else {
            false
        }
    });
        
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
        interests: request.interests.clone().unwrap_or_default(),
        status: UserStatus::Active,
        role: UserRole::User,
    };
    
    // Create user profile
    let profile = UserProfile {
        principal: caller,
        username: request.username,
        handle: request.handle,
        bio: request.bio,
        avatar: request.avatar,
        social_links: request.social_links.unwrap_or_default(),
        interests: request.interests.unwrap_or_default(),
        followed_users: HashSet::new(),
        followers: HashSet::new(),
        followed_topics: HashSet::new(),
        privacy_settings: None,
        updated_at: now,
        created_at: now,
        followers_count: 0,
        following_count: 0
    };
    
    // Create user stats
    let stats = UserStats {
        principal: caller,
        post_count: 0,
        comment_count: 0,
        like_count: 0,
        points: 0,
        reputation: 0,
    };
    
    // Store user data in main storage
    STORAGE.with(|storage| {
        let mut store = storage.borrow_mut();
        
        // Store user
        store.users.insert(caller, user);
        
        // Store user profile
        if store.user_profiles.is_none() {
            store.user_profiles = Some(HashMap::new());
        }
        if let Some(profiles) = &mut store.user_profiles {
            profiles.insert(caller, profile);
        }
        
        // Store user stats
        if store.user_stats.is_none() {
            store.user_stats = Some(HashMap::new());
        }
        if let Some(user_stats) = &mut store.user_stats {
            user_stats.insert(caller, stats);
        }
    });
    
    Ok(())
}

pub fn update_user_profile(request: UpdateProfileRequest, caller: Principal) -> SquareResult<String> {
    const MODULE: &str = "services::user::profile";
    const FUNCTION: &str = "update_user_profile";
    
    let now = time() / 1_000_000;
    // Get user profile from main storage
    let profile_result = STORAGE.with(|storage| {
        let store = storage.borrow();
        if let Some(profiles) = &store.user_profiles {
            profiles.get(&caller).cloned()
        } else {
            None
        }
    });
    
    // Check if user exists
    let profile = match profile_result {
        Some(profile) => profile,
        None => return log_and_return(not_found_error(
            "UserProfile", 
            &caller.to_string(), 
            MODULE, 
            FUNCTION
        ).with_details("User profile not found")),
    };
    
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
            
            // Check if username is taken in main storage
            let username_taken = STORAGE.with(|storage| {
                let store = storage.borrow();
                if let Some(profiles) = &store.user_profiles {
                    profiles.values().any(|p| p.username != current_username && p.username == *username)
                } else {
                    false
                }
            });
            
            if username_taken {
                return log_and_return(already_exists_error(
                    "Username", 
                    username, 
                    MODULE, 
                    FUNCTION
                ).with_details(format!("Username '{}' is already taken", username)));
            }
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
    
    // Update handle if provided
    if let Some(ref handle) = request.handle {
        // Validate handle format
        let handle_regex = regex::Regex::new(HANDLE_PATTERN).unwrap();
        if !handle_regex.is_match(handle) {
            return log_and_return(validation_error(
                "Handle must start with a letter and can only contain letters, numbers, and underscores",
                MODULE,
                FUNCTION
            ));
        }
        
        // Check if handle is already taken in main storage
        let handle_taken = STORAGE.with(|storage| {
            let store = storage.borrow();
            if let Some(profiles) = &store.user_profiles {
                profiles.iter().any(|(principal, p)| {
                    *principal != caller && p.handle == *handle
                })
            } else {
                false
            }
        });
        
        if handle_taken {
            return log_and_return(already_exists_error(
                "Handle", 
                handle, 
                MODULE, 
                FUNCTION
            ).with_details(format!("Handle '{}' is already taken", handle)));
        }
    }
    
    // Create updated profile
    let mut updated_profile = profile.clone();
    
    // Apply username update
    if let Some(username) = request.username {
        updated_profile.username = username;
    }
    
    // Apply bio update
    if let Some(bio) = request.bio {
        updated_profile.bio = bio;
    }
    
    // Update avatar URL
    if let Some(avatar) = request.avatar {
        updated_profile.avatar = avatar;
    }
    
    // Update handle if provided
    if let Some(handle) = request.handle {
        updated_profile.handle = handle;
    }
    
    // Update social links if provided
    if let Some(links) = request.social_links {
        updated_profile.social_links = links;
    }
    
    // Update the profile in main storage
    STORAGE.with(|storage| {
        let mut store = storage.borrow_mut();
        
        // Update user profile
        if let Some(profiles) = &mut store.user_profiles {
            profiles.insert(caller, updated_profile);
        }
        
        // Update last login time in user record
        if let Some(mut user) = store.users.get(&caller).cloned() {
            user.last_login = now;
            store.users.insert(caller, user);
        }
    });
    
    Ok("Profile updated successfully".to_string())
}

pub fn get_user_profile(user_identifier: String) -> SquareResult<UserProfileResponse> {
    const MODULE: &str = "services::user::profile";
    const FUNCTION: &str = "get_user_profile";
    
    ic_cdk::println!("[{}::{}] Getting profile for user: {}", MODULE, FUNCTION, user_identifier);
    
    let principal = if let Ok(principal) = Principal::from_text(&user_identifier) {
        // It's a valid Principal
        principal
    } else {
        // Try to find user by handle
        let principal = super::utils::find_user_by_handle(&user_identifier)?;
        principal
    };
    
    // Get user from main storage
    let user_result = STORAGE.with(|storage| {
        let store = storage.borrow();
        store.users.get(&principal).cloned()
    });
    
    // Get user profile from main storage
    let profile_result = STORAGE.with(|storage| {
        let store = storage.borrow();
        if let Some(profiles) = &store.user_profiles {
            profiles.get(&principal).cloned()
        } else {
            None
        }
    });
    
    // Get user stats from main storage
    let stats_result = STORAGE.with(|storage| {
        let store = storage.borrow();
        if let Some(stats) = &store.user_stats {
            stats.get(&principal).cloned()
        } else {
            None
        }
    });
    
    // Check if user exists
    let user = match user_result {
        Some(user) => user,
        None => return log_and_return(not_found_error(
            "User", 
            &principal.to_string(), 
            MODULE, 
            FUNCTION
        ).with_details("User not found")),
    };
    
    // Check if profile exists
    let profile = match profile_result {
        Some(profile) => profile,
        None => return log_and_return(not_found_error(
            "UserProfile", 
            &principal.to_string(), 
            MODULE, 
            FUNCTION
        ).with_details("User profile not found")),
    };
    
    // Get stats (if available)
    let _stats = stats_result.unwrap_or(UserStats {
        principal,
        post_count: 0,
        comment_count: 0,
        like_count: 0,
        points: 0,
        reputation: 0,
    });
    
    Ok(UserProfileResponse {
        principal,
        username: profile.username,
        handle: profile.handle,
        bio: profile.bio,
        avatar: profile.avatar,
        social_links: profile.social_links,
        interests: profile.interests,
        followers_count: profile.followers_count,
        following_count: profile.following_count,
        is_following: false,
        status: user.status,
        role: user.role,
        registered_at: user.registered_at,
        last_login: user.last_login,
        created_at: profile.created_at,
        updated_at: profile.updated_at,
        privacy_settings: profile.privacy_settings,
    })
}

pub fn get_user_full_profile(user_identifier: String) -> SquareResult<UserResponse> {
    const MODULE: &str = "services::user::profile";
    const FUNCTION: &str = "get_user_full_profile";
    
    // Get user profile
    let profile = get_user_profile(user_identifier.clone())?;
    
    // Get followed users
    let _followed_users = super::social::get_following(user_identifier.clone(), None)?;
    
    // Get followers
    let _followers = super::social::get_followers(user_identifier, None)?;
    
    Ok(UserResponse {
        principal: profile.principal,
        username: profile.username,
        handle: profile.handle,
        bio: profile.bio,
        avatar: profile.avatar,
        interests: profile.interests,
        social_links: profile.social_links,
        followers_count: profile.followers_count,
        following_count: profile.following_count,
        created_at: profile.created_at,
        updated_at: profile.updated_at,
        stats: UserStatsResponse {
            post_count: 0, // TODO: Get from stats
            comment_count: 0,
            like_count: 0,
            points: 0,
            reputation: 0,
        },
        status: profile.status,
        role: profile.role,
        is_following: profile.is_following,
    })
}

pub fn debug_fix_user_profile(user_identifier: String) -> SquareResult<String> {
    const MODULE: &str = "services::user::profile";
    const FUNCTION: &str = "debug_fix_user_profile";
    
    // Parse user identifier
    let principal = if let Ok(principal) = Principal::from_text(&user_identifier) {
        principal
    } else {
        return log_and_return(validation_error(
            "Invalid principal format", 
            MODULE, 
            FUNCTION
        ));
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
        
        return Ok(format!("Created default profile for user {}", principal));
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
        
        return Ok(format!("Created default stats for user {}", principal));
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
    
    Ok(format!("Fixed profile for user {}", principal))
}
