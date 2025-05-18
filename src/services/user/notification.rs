use candid::Principal;
use ic_cdk::api::time;
use std::collections::HashMap;
use std::borrow::{Borrow, BorrowMut};

use crate::models::notification::*;
use crate::models::error::{SquareError, SquareResult};
use crate::models::content::PaginationParams;
use crate::storage::{STORAGE};
use crate::utils::error_handler::*;

// Notification functions
pub fn create_notification(
    user_principal: Principal,
    notification_type: NotificationType,
    content: String,
    related_entity_id: Option<String>,
    related_user: Option<Principal>
) -> SquareResult<()> {
    const MODULE: &str = "services::user::notification";
    const FUNCTION: &str = "create_notification";
    
    let now = time() / 1_000_000;
    let notification_id = format!("{}-{}", now, user_principal);
    
    let notification = UserNotification {
        id: notification_id,
        user_id: user_principal,
        notification_type,
        content,
        related_content_id: related_entity_id,
        related_user,
        read: false,
        created_at: now,
    };
    
    // Store notification in main storage
    STORAGE.with(|storage| {
        let mut store = storage.borrow_mut();
        
        // Get user notifications
        if store.user_notifications.contains_key(&user_principal) {
            // User already has notifications
        } else {
            // Create empty notifications list for user
            store.user_notifications.insert(user_principal, Vec::new());
        }
        
        // Get user notifications
        if let Some(user_notifications) = store.user_notifications.get_mut(&user_principal) {
            // Add notification to user's notifications list
            
            // Add notification
            user_notifications.push(notification);
            
            // Sort notifications by creation time (newest first)
            user_notifications.sort_by(|a, b| b.created_at.cmp(&a.created_at));
            
            // Limit to max notifications per user
            const MAX_NOTIFICATIONS: usize = 100;
            if user_notifications.len() > MAX_NOTIFICATIONS {
                *user_notifications = user_notifications[0..MAX_NOTIFICATIONS].to_vec();
            }
        }
    });
    
    Ok(())
}

pub fn get_user_notifications(principal: Principal, pagination: PaginationParams) -> SquareResult<NotificationsResponse> {
    const MODULE: &str = "services::user::notification";
    const FUNCTION: &str = "get_user_notifications";
    
    // Get notifications from main storage
    let notifications_result = STORAGE.with(|storage| {
        let store = storage.borrow();
        store.user_notifications.get(&principal).cloned()
    });
    
    // Get notifications
    let notifications = notifications_result.unwrap_or_default();
    
    // Apply pagination
    let total = notifications.len() as u64;
    let start = pagination.offset.unwrap_or(0);
    let limit = pagination.limit.unwrap_or(10);
    let end = std::cmp::min(start + limit, notifications.len());
    
    // Get paginated notifications
    let paginated_notifications = if start < notifications.len() {
        notifications[start..end].to_vec()
    } else {
        Vec::new()
    };
    
    // Count unread notifications
    let unread_count = notifications.iter().filter(|n| !n.read).count() as u64;
    
    Ok(NotificationsResponse {
        notifications: paginated_notifications.into_iter().map(|n| NotificationResponse {
            id: n.id,
            notification_type: n.notification_type,
            content: n.content,
            related_user: n.related_user,
            related_content_id: n.related_content_id,
            created_at: n.created_at,
            read: n.read,
        }).collect(),
        total,
        unread_count,
        has_more: (start + limit) < notifications.len(),
        next_cursor: Some(if (start + limit) < notifications.len() {
            (start + limit).to_string()
        } else {
            start.to_string()
        }),
    })
}

pub fn mark_notification_as_read(principal: Principal, notification_id: String) -> SquareResult<()> {
    const MODULE: &str = "services::user::notification";
    const FUNCTION: &str = "mark_notification_as_read";
    
    // Update notification in main storage
    STORAGE.with(|storage| {
        let mut store = storage.borrow_mut();
        
        if let Some(user_notifications) = store.user_notifications.get_mut(&principal) {
            for notification in user_notifications {
                if notification.id == notification_id {
                    notification.read = true;
                    break;
                }
            }
        }
        Ok(())
    })
}

pub fn mark_all_notifications_as_read(principal: Principal) -> SquareResult<()> {
    const MODULE: &str = "services::user::notification";
    const FUNCTION: &str = "mark_all_notifications_as_read";
    
    // Update notifications in main storage
    STORAGE.with(|storage| {
        let mut store = storage.borrow_mut();
        
        if let Some(user_notifications) = store.user_notifications.get_mut(&principal) {
            for notification in user_notifications {
                notification.read = true;
            }
        }
        Ok(())
    })
}