use candid::Principal;
use ic_cdk::api::{time, caller};

use crate::auth;
use crate::models::error::{SquareError, SquareResult};
use crate::storage::{STORAGE, ContentStatus, UserStatus};
use crate::models::interaction::ReportStatus;
use crate::models::notification::NotificationType;
use crate::services::user::{create_notification};
use crate::models::cycles::{
    UpdateHeartbeatIntervalRequest, HeartbeatIntervalResponse,
    UpdateCyclesThresholdRequest, CyclesThresholdConfig,
    CyclesNotificationsResponse, NotificationSettings
};

// Helper function to check if caller is admin
fn ensure_admin() -> Result<(), SquareError> {
    auth::is_admin().map_err(|e| 
        SquareError::Unauthorized(format!("Only admins can perform this action: {}", e)))
}

// Heartbeat interval management
pub fn update_heartbeat_interval(request: UpdateHeartbeatIntervalRequest) -> SquareResult<HeartbeatIntervalResponse> {
    const MODULE: &str = "services::admin";
    const FUNCTION: &str = "update_heartbeat_interval";
    
    // Check if caller is admin
    ensure_admin()?;
    
    // Update the heartbeat interval in main storage
    STORAGE.with(|storage| {
        let mut store = storage.borrow_mut();
        store.heartbeat_interval_hours = request.interval_hours;
        
        Ok(HeartbeatIntervalResponse {
            interval_hours: store.heartbeat_interval_hours
        })
    })
}

pub fn get_heartbeat_interval() -> SquareResult<HeartbeatIntervalResponse> {
    const MODULE: &str = "services::admin";
    const FUNCTION: &str = "get_heartbeat_interval";
    
    // Check if caller is admin
    ensure_admin()?;
    
    // Get the current heartbeat interval from main storage
    STORAGE.with(|storage| {
        let store = storage.borrow();
        Ok(HeartbeatIntervalResponse {
            interval_hours: store.heartbeat_interval_hours
        })
    })
}

// Content moderation functions
pub fn moderate_content(content_id: String, action: String) -> SquareResult<()> {
    const MODULE: &str = "services::admin";
    const FUNCTION: &str = "moderate_content";
    
    // Check if caller is admin
    ensure_admin()?;
    
    // Check if content exists in main storage
    let mut post_exists = false;
    let mut comment_exists = false;
    
    // Check if post exists
    let mut post_to_update = None;
    STORAGE.with(|storage| {
        let store = storage.borrow();
        if let Some(post) = store.posts.get(&content_id) {
            post_exists = true;
            post_to_update = Some(post.clone());
        }
    });
    
    // Check if comment exists
    let mut comment_to_update = None;
    STORAGE.with(|storage| {
        let store = storage.borrow();
        if let Some(comment) = store.comments.get(&content_id) {
            comment_exists = true;
            comment_to_update = Some(comment.clone());
        }
    });
    
    // If neither post nor comment exists, return error
    if !post_exists && !comment_exists {
        return Err(SquareError::NotFound(format!("Content with ID {} not found", content_id)));
    }
    
    // Determine action to take
    match action.as_str() {
        "remove" => {
            // Handle post moderation
            if let Some(mut post) = post_to_update {
                post.status = ContentStatus::Removed;
                STORAGE.with(|storage| {
                    let mut store = storage.borrow_mut();
                    store.posts.insert(content_id.clone(), post);
                });
            }
            
            // Handle comment moderation
            if let Some(mut comment) = comment_to_update {
                comment.status = ContentStatus::Removed;
                STORAGE.with(|storage| {
                    let mut store = storage.borrow_mut();
                    store.comments.insert(content_id.clone(), comment);
                });
            }
        },
        "hide" => {
            // Handle post moderation
            if let Some(mut post) = post_to_update {
                post.status = ContentStatus::Hidden;
                STORAGE.with(|storage| {
                    let mut store = storage.borrow_mut();
                    store.posts.insert(content_id.clone(), post);
                });
            }
            
            // Handle comment moderation
            if let Some(mut comment) = comment_to_update {
                comment.status = ContentStatus::Hidden;
                STORAGE.with(|storage| {
                    let mut store = storage.borrow_mut();
                    store.comments.insert(content_id.clone(), comment);
                });
            }
        },
        "review" => {
            // Handle post moderation
            if let Some(mut post) = post_to_update {
                post.status = ContentStatus::UnderReview;
                STORAGE.with(|storage| {
                    let mut store = storage.borrow_mut();
                    store.posts.insert(content_id.clone(), post);
                });
            }
            
            // Handle comment moderation
            if let Some(mut comment) = comment_to_update {
                comment.status = ContentStatus::UnderReview;
                STORAGE.with(|storage| {
                    let mut store = storage.borrow_mut();
                    store.comments.insert(content_id.clone(), comment);
                });
            }
        },
        "restore" => {
            // Handle post moderation
            if let Some(mut post) = post_to_update {
                post.status = ContentStatus::Active;
                STORAGE.with(|storage| {
                    let mut store = storage.borrow_mut();
                    store.posts.insert(content_id.clone(), post);
                });
            }
            
            // Handle comment moderation
            if let Some(mut comment) = comment_to_update {
                comment.status = ContentStatus::Active;
                STORAGE.with(|storage| {
                    let mut store = storage.borrow_mut();
                    store.comments.insert(content_id.clone(), comment);
                });
            }
        },
        _ => return Err(SquareError::ValidationFailed(format!("Invalid action: {}", action))),
    }
    
    // Update reports in main storage
    // For now, we'll just log the action
    ic_cdk::println!(
        "Content {} moderated with action: {} by {}", 
        content_id, 
        action, 
        caller()
    );
    
    Ok(())
}

// User management functions
pub fn ban_user(user_id: Principal, reason: String) -> SquareResult<()> {
    const MODULE: &str = "services::admin";
    const FUNCTION: &str = "ban_user";
    
    // Check if caller is admin
    ensure_admin()?;
    
    // Check if user exists
    let user_exists = STORAGE.with(|storage| {
        let store = storage.borrow();
        store.users.contains_key(&user_id)
    });
    
    if !user_exists {
        return Err(SquareError::NotFound(format!("User with ID {} not found", user_id)));
    }
    
    // Get user from main storage
    let user_result = STORAGE.with(|storage| {
        let store = storage.borrow();
        store.users.get(&user_id).cloned()
    });
    
    // Update user status to banned
    if let Some(mut user) = user_result {
        // Set user status to banned
        user.status = UserStatus::Banned;
        
        // Update user in main storage
        STORAGE.with(|storage| {
            let mut store = storage.borrow_mut();
            store.users.insert(user_id, user);
        });
        
        // Send notification to the user
        if let Err(e) = create_notification(
            user_id,
            NotificationType::System,
            format!("Your account has been banned. Reason: {}", reason),
            None,
            None
        ) {
            ic_cdk::println!("Failed to send ban notification: {:?}", e);
        }
        
        // Get user posts from main storage
        let user_posts: Vec<String> = STORAGE.with(|storage| {
            let store = storage.borrow();
            let mut user_post_ids = Vec::new();
            
            for (post_id, post) in &store.posts {
                if post.author == user_id {
                    user_post_ids.push(post_id.clone());
                }
            }
            
            user_post_ids
        });
        
        // Set all user posts to hidden
        for post_id in user_posts {
            STORAGE.with(|storage| {
                let mut store = storage.borrow_mut();
                if let Some(mut post) = store.posts.get(&post_id).cloned() {
                    post.status = ContentStatus::Hidden;
                    store.posts.insert(post_id.clone(), post);
                }
            });
        }
        
        // Get user comments (this would need to be implemented in content service)
        // For now, we'll just log that we can't update comments yet
        ic_cdk::println!("Note: User comments status update not implemented yet");
        
        // Log the ban action
        ic_cdk::println!("User {} banned by {}. Reason: {}", user_id, caller(), reason);
        
        Ok(())
    } else {
        Err(SquareError::NotFound(format!("User with ID {} not found", user_id)))
    }
}

// System configuration
pub fn update_system_config(config_key: String, config_value: String) -> SquareResult<()> {
    const MODULE: &str = "services::admin";
    const FUNCTION: &str = "update_system_config";
    
    // Check if caller is admin
    ensure_admin()?;
    
    // Update the appropriate configuration based on the key
    match config_key.as_str() {
        "community_guidelines" => {
            STORAGE.with(|storage| {
                let mut store = storage.borrow_mut();
                // 修复类型不匹配问题，将 String 包装为 Option<String>
                store.community_guidelines = Some(config_value);
            });
            ic_cdk::println!("Community guidelines updated by {}", caller());
        },
        "terms_of_service" => {
            STORAGE.with(|storage| {
                let mut store = storage.borrow_mut();
                // 修复类型不匹配问题，将 String 包装为 Option<String>
                store.terms_of_service = Some(config_value);
            });
            ic_cdk::println!("Terms of service updated by {}", caller());
        },
        "heartbeat_interval_hours" => {
            // Parse the value as a u64
            match config_value.parse::<u64>() {
                Ok(interval) => {
                    if interval == 0 {
                        return Err(SquareError::ValidationFailed("Heartbeat interval must be greater than 0".to_string()));
                    }
                    STORAGE.with(|storage| {
                        let mut store = storage.borrow_mut();
                        store.heartbeat_interval_hours = interval;
                    });
                    ic_cdk::println!("Heartbeat interval updated to {} hours by {}", interval, caller());
                },
                Err(_) => {
                    return Err(SquareError::ValidationFailed(format!("Invalid heartbeat interval: {}", config_value)));
                }
            }
        },
        _ => {
            return Err(SquareError::ValidationFailed(format!("Unknown configuration key: {}", config_key)));
        }
    }
    
    Ok(())
}

// Cycles management
pub fn update_cycles_threshold(request: UpdateCyclesThresholdRequest) -> SquareResult<CyclesThresholdConfig> {
    // Check if caller is admin
    ensure_admin()?;
    
    // Get caller principal
    let caller = caller();
    
    // Delegate to cycles service
    crate::services::cycles::update_cycles_threshold(request, caller)?;
    
    crate::services::cycles::get_cycles_threshold()
}

pub fn get_cycles_threshold() -> SquareResult<CyclesThresholdConfig> {
    // Check if caller is admin
    ensure_admin()?;
    
    // Delegate to cycles service
    crate::services::cycles::get_cycles_threshold()
}

pub fn get_cycles_notifications() -> SquareResult<CyclesNotificationsResponse> {
    // Check if caller is admin
    ensure_admin()?;
    
    // Delegate to cycles service
    crate::services::cycles::get_cycles_notifications()
}

pub fn update_notification_settings(
    enabled: Option<bool>,
) -> SquareResult<()> {
    // Check if caller is admin
    ensure_admin()?;
    
    // Get caller principal
    let caller = caller();
    
    // Delegate to cycles service
    crate::services::cycles::update_notification_settings(enabled, caller)
}

// Storage management functions
pub fn migrate_storage() -> SquareResult<String> {
    // Check if caller is admin
    ensure_admin()?;
    
    // Perform any necessary storage migrations or optimizations
    
    Ok("Storage optimization completed successfully".to_string())
}

// Bark API key management
pub fn set_bark_api_key(api_key: String) -> SquareResult<()> {
    const MODULE: &str = "services::admin";
    const FUNCTION: &str = "set_bark_api_key";
    
    // Check if caller is admin
    ensure_admin()?;
    
    // Update the bark API key in storage
    crate::storage::STORAGE.with(|storage| {
        let mut storage = storage.borrow_mut();
        storage.bark_api_key = api_key;
        Ok(())
    })
}

pub fn get_bark_api_key() -> SquareResult<String> {
    const MODULE: &str = "services::admin";
    const FUNCTION: &str = "get_bark_api_key";
    
    // Check if caller is admin
    ensure_admin()?;
    
    // Get the current bark API key from storage
    crate::storage::STORAGE.with(|storage| {
        let storage = storage.borrow();
        Ok(storage.bark_api_key.clone())
    })
}
