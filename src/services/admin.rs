use candid::Principal;
use ic_cdk::api::time;

use crate::auth;
use crate::models::error::{SquareError, SquareResult};
use crate::storage::STORAGE;
use crate::storage::{ContentStatus, UserStatus};
use crate::models::interaction::ReportStatus;
use crate::models::user::NotificationType;
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
    // Check if caller is admin
    ensure_admin()?;
    
    // Update the heartbeat interval
    STORAGE.with(|storage| {
        let mut storage = storage.borrow_mut();
        storage.heartbeat_interval_hours = request.interval_hours;
        
        Ok(HeartbeatIntervalResponse {
            interval_hours: storage.heartbeat_interval_hours
        })
    })
}

pub fn get_heartbeat_interval() -> SquareResult<HeartbeatIntervalResponse> {
    // Check if caller is admin
    ensure_admin()?;
    
    // Get the current heartbeat interval
    STORAGE.with(|storage| {
        let storage = storage.borrow();
        Ok(HeartbeatIntervalResponse {
            interval_hours: storage.heartbeat_interval_hours
        })
    })
}

// Content moderation functions
pub fn moderate_content(content_id: String, action: String) -> SquareResult<()> {
    // Check if caller is admin
    ensure_admin()?;
    
    // Implementation of content moderation
    STORAGE.with(|storage| {
        let mut storage = storage.borrow_mut();
        
        // Check if content exists
        let content_exists = storage.posts.contains_key(&content_id) || storage.comments.contains_key(&content_id);
        if !content_exists {
            return Err(SquareError::NotFound(format!("Content with ID {} not found", content_id)));
        }
        
        // Determine action to take
        match action.as_str() {
            "remove" => {
                // Handle post moderation
                if let Some(mut post) = storage.posts.get(&content_id).cloned() {
                    post.status = ContentStatus::Removed;
                    storage.posts.insert(content_id.clone(), post);
                }
                
                // Handle comment moderation
                if let Some(mut comment) = storage.comments.get(&content_id).cloned() {
                    comment.status = ContentStatus::Removed;
                    storage.comments.insert(content_id.clone(), comment);
                }
            },
            "hide" => {
                // Handle post moderation
                if let Some(mut post) = storage.posts.get(&content_id).cloned() {
                    post.status = ContentStatus::Hidden;
                    storage.posts.insert(content_id.clone(), post);
                }
                
                // Handle comment moderation
                if let Some(mut comment) = storage.comments.get(&content_id).cloned() {
                    comment.status = ContentStatus::Hidden;
                    storage.comments.insert(content_id.clone(), comment);
                }
            },
            "review" => {
                // Handle post moderation
                if let Some(mut post) = storage.posts.get(&content_id).cloned() {
                    post.status = ContentStatus::UnderReview;
                    storage.posts.insert(content_id.clone(), post);
                }
                
                // Handle comment moderation
                if let Some(mut comment) = storage.comments.get(&content_id).cloned() {
                    comment.status = ContentStatus::UnderReview;
                    storage.comments.insert(content_id.clone(), comment);
                }
            },
            "restore" => {
                // Handle post moderation
                if let Some(mut post) = storage.posts.get(&content_id).cloned() {
                    post.status = ContentStatus::Active;
                    storage.posts.insert(content_id.clone(), post);
                }
                
                // Handle comment moderation
                if let Some(mut comment) = storage.comments.get(&content_id).cloned() {
                    comment.status = ContentStatus::Active;
                    storage.comments.insert(content_id.clone(), comment);
                }
            },
            _ => return Err(SquareError::ValidationFailed(format!("Invalid action: {}", action))),
        }
        
        // Update any related reports
        for (_report_id, report) in storage.reports.iter_mut() {
            if report.content_id == content_id {
                report.status = ReportStatus::Resolved;
                report.resolved_at = Some(ic_cdk::api::time());
                report.resolver = Some(ic_cdk::api::caller());
                report.resolution_notes = Some(format!("Content moderated with action: {}", action));
            }
        }
        
        Ok(())
    })
}

// User management functions
pub fn ban_user(user_id: Principal, reason: String) -> SquareResult<()> {
    // Check if caller is admin
    ensure_admin()?;
    
    // Implementation of user banning
    STORAGE.with(|storage| {
        let mut storage = storage.borrow_mut();
        
        // Check if user exists
        if !storage.users.contains_key(&user_id) {
            return Err(SquareError::NotFound(format!("User with ID {} not found", user_id)));
        }
        
        // Update user status to banned
        if let Some(mut user) = storage.users.get(&user_id).cloned() {
            // Set user status to banned
            user.status = UserStatus::Banned;
            storage.users.insert(user_id, user);
            
            // Send notification to the user
            if let Err(e) = crate::services::user::create_notification(
                user_id,
                NotificationType::System,
                format!("Your account has been banned. Reason: {}", reason),
                None,
                None
            ) {
                ic_cdk::println!("Failed to send ban notification: {:?}", e);
            }
            
            // Set all user content to hidden
            let post_ids_to_update = if let Some(post_ids) = storage.user_posts.get(&user_id) {
                post_ids.clone()
            } else {
                Vec::new()
            };
            
            for post_id in post_ids_to_update {
                if let Some(mut post) = storage.posts.get(&post_id).cloned() {
                    post.status = ContentStatus::Hidden;
                    storage.posts.insert(post_id.clone(), post);
                }
            }
            
            let comment_ids_to_update = if let Some(comment_ids) = storage.user_comments.get(&user_id) {
                comment_ids.clone()
            } else {
                Vec::new()
            };
            
            for comment_id in comment_ids_to_update {
                if let Some(mut comment) = storage.comments.get(&comment_id).cloned() {
                    comment.status = ContentStatus::Hidden;
                    storage.comments.insert(comment_id.clone(), comment);
                }
            }
            
            // Log the ban action
            ic_cdk::println!("User {} banned by {}. Reason: {}", user_id, ic_cdk::api::caller(), reason);
            
            Ok(())
        } else {
            Err(SquareError::NotFound(format!("User with ID {} not found", user_id)))
        }
    })
}

// System configuration
pub fn update_system_config(config_key: String, config_value: String) -> SquareResult<()> {
    // Check if caller is admin
    ensure_admin()?;
    
    // Implementation of system configuration update
    STORAGE.with(|storage| {
        let mut storage = storage.borrow_mut();
        
        // Update the appropriate configuration based on the key
        match config_key.as_str() {
            "community_guidelines" => {
                storage.community_guidelines = config_value;
                ic_cdk::println!("Community guidelines updated by {}", ic_cdk::api::caller());
            },
            "terms_of_service" => {
                storage.terms_of_service = config_value;
                ic_cdk::println!("Terms of service updated by {}", ic_cdk::api::caller());
            },
            "heartbeat_interval_hours" => {
                // Parse the value as a u64
                match config_value.parse::<u64>() {
                    Ok(interval) => {
                        if interval == 0 {
                            return Err(SquareError::ValidationFailed("Heartbeat interval must be greater than 0".to_string()));
                        }
                        storage.heartbeat_interval_hours = interval;
                        ic_cdk::println!("Heartbeat interval updated to {} hours by {}", interval, ic_cdk::api::caller());
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
    })
}

// Cycles management
pub fn update_cycles_threshold(request: UpdateCyclesThresholdRequest) -> SquareResult<CyclesThresholdConfig> {
    // Check if caller is admin
    ensure_admin()?;
    
    // Get caller principal
    let caller = ic_cdk::api::caller();
    
    // Delegate to cycles service
    // 由于 update_cycles_threshold 返回 Result<(), SquareError>，我们需要转换返回类型
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
    email: Option<String>,
) -> SquareResult<()> {
    // Check if caller is admin
    ensure_admin()?;
    
    // Get caller principal
    let caller = ic_cdk::api::caller();
    
    // Delegate to cycles service
    crate::services::cycles::update_notification_settings(enabled, email, None, caller)
}

// Storage management
pub fn migrate_to_sharded_storage() -> SquareResult<String> {
    // Check if caller is admin
    ensure_admin()?;
    
    // Implementation of storage migration
    
    Ok("Storage migration started".to_string())
}
