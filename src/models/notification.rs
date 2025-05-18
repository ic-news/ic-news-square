use candid::{CandidType, Deserialize, Principal};

// Notification structures
#[derive(CandidType, Deserialize, Clone)]
pub struct UserNotification {
    pub id: String,
    pub user_id: Principal,
    pub notification_type: NotificationType,
    pub content: String,
    pub related_user: Option<Principal>,
    pub related_content_id: Option<String>,
    pub created_at: u64,
    pub read: bool,
}

#[derive(CandidType, Deserialize, Clone, PartialEq)]
pub enum NotificationType {
    Follow,
    Like,
    Comment,
    Reply,
    Mention,
    System,
    Achievement,
    Custom,
}

#[derive(CandidType, Deserialize, Clone)]
pub struct NotificationResponse {
    pub id: String,
    pub notification_type: NotificationType,
    pub content: String,
    pub related_user: Option<Principal>,
    pub related_content_id: Option<String>,
    pub created_at: u64,
    pub read: bool,
}

#[derive(CandidType, Deserialize, Clone)]
pub struct NotificationsResponse {
    pub notifications: Vec<NotificationResponse>,
    pub total: u64,
    pub unread_count: u64,
    pub has_more: bool,
    pub next_cursor: Option<String>,
}
