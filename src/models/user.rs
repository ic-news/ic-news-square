use candid::{CandidType, Deserialize, Principal};
use std::collections::{HashSet, HashMap};
use crate::models::notification::NotificationType;

// Constants for validation
pub const MIN_USERNAME_LENGTH: usize = 3;
pub const MAX_USERNAME_LENGTH: usize = 30;
pub const MAX_BIO_LENGTH: usize = 500;
pub const HANDLE_PATTERN: &str = r"^[a-zA-Z0-9_]{3,30}$";

// User data structures
#[derive(CandidType, Deserialize, Clone)]
pub struct User {
    pub principal: Principal,
    pub registered_at: u64,
    pub last_login: u64,
    pub interests: Vec<String>,
    pub status: UserStatus,
    pub role: UserRole,
}



#[derive(CandidType, Deserialize, Clone)]
pub struct UserProfile {
    pub principal: Principal,
    pub username: String,
    pub handle: String,
    pub bio: String,
    pub avatar: String,
    pub interests: Vec<String>,
    pub social_links: Vec<(String, String)>,
    pub followers: HashSet<Principal>,
    pub followed_users: HashSet<Principal>,
    pub followed_topics: HashSet<String>,
    pub followers_count: u64,
    pub following_count: u64,
    pub created_at: u64,
    pub updated_at: u64,
    pub privacy_settings: Option<UserPrivacySettings>,
}

#[derive(CandidType, Deserialize, Clone)]
pub struct UserStats {
    pub principal: Principal,
    pub post_count: u64,
    pub comment_count: u64,
    pub like_count: u64,
    pub points: u64,
    pub reputation: u64,
}

// User Identifier
#[derive(CandidType, Deserialize, Clone)]
pub struct UserIdentifier {
    pub principal: Option<Principal>,
    pub handle: Option<String>,
}

// Request DTOs
#[derive(CandidType, Deserialize, Clone)]
pub struct RegisterUserRequest {
    pub username: String,
    pub handle: String,
    pub bio: String,
    pub avatar: String,
    pub social_links: Option<Vec<(String, String)>>,
    pub interests: Option<Vec<String>>,
}

#[derive(CandidType, Deserialize, Clone)]
pub struct UpdateProfileRequest {
    pub username: Option<String>,
    pub handle: Option<String>,
    pub bio: Option<String>,
    pub avatar: Option<String>,
    pub social_links: Option<Vec<(String, String)>>,
    pub interests: Option<Vec<String>>,
    pub privacy_settings: Option<UserPrivacySettings>,
}

#[derive(CandidType, Deserialize, Clone, PartialEq)]
pub struct UserPrivacySettings {
    pub profile_visibility: ProfileVisibility,
    pub content_visibility: ContentVisibility,
    pub interaction_preferences: InteractionPreferences,
    pub notification_preferences: NotificationPreferences,
}

impl Default for UserPrivacySettings {
    fn default() -> Self {
        Self {
            profile_visibility: ProfileVisibility::Public,
            content_visibility: ContentVisibility::Public,
            interaction_preferences: InteractionPreferences::default(),
            notification_preferences: NotificationPreferences::default(),
        }
    }
}

#[derive(CandidType, Deserialize, Clone, PartialEq)]
pub enum ProfileVisibility {
    Public,
    FollowersOnly,
    Private,
}

#[derive(CandidType, Deserialize, Clone, PartialEq)]
pub enum ContentVisibility {
    Public,
    FollowersOnly,
    Private,
}

#[derive(CandidType, Deserialize, Clone, PartialEq)]
pub struct InteractionPreferences {
    pub allow_comments: bool,
    pub allow_mentions: bool,
    pub allow_follows: bool,
    pub show_likes: bool,
}

impl Default for InteractionPreferences {
    fn default() -> Self {
        Self {
            allow_comments: true,
            allow_mentions: true,
            allow_follows: true,
            show_likes: true,
        }
    }
}

#[derive(CandidType, Deserialize, Clone, PartialEq)]
pub struct NotificationPreferences {
    pub likes: bool,
    pub comments: bool,
    pub follows: bool,
    pub mentions: bool,
    pub system: bool,
}

impl Default for NotificationPreferences {
    fn default() -> Self {
        Self {
            likes: true,
            comments: true,
            follows: true,
            mentions: true,
            system: true,
        }
    }
}

#[derive(CandidType, Deserialize, Clone)]
pub struct FollowUserRequest {
    pub user_to_follow: Principal,
}

#[derive(CandidType, Deserialize, Clone)]
pub struct FollowTopicRequest {
    pub topic: String,
}

#[derive(CandidType, Deserialize, Clone)]
pub struct UserStatusUpdateRequest {
    pub principal: Principal,
    pub status: UserStatus,
}

#[derive(CandidType, Deserialize, Clone)]
pub struct UserRoleUpdateRequest {
    pub principal: Principal,
    pub role: UserRole,
}

// Response DTOs
#[derive(CandidType, Deserialize, Clone)]
pub struct UserProfileResponse {
    pub principal: Principal,
    pub username: String,
    pub handle: String,
    pub bio: String,
    pub avatar: String,
    pub social_links: Vec<(String, String)>,
    pub followers_count: u64,
    pub following_count: u64,
    pub registered_at: u64,
    pub last_login: u64,
    pub status: UserStatus,
    pub role: UserRole,
    pub is_following: bool,
    pub interests: Vec<String>,
    pub created_at: u64,
    pub updated_at: u64,
    pub privacy_settings: Option<UserPrivacySettings>,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct UserSocialResponse {
    pub principal: Principal,
    pub username: String,
    pub handle: String,
    pub avatar: String,
    pub bio: String,
    pub interests: Vec<String>,
    pub followers_count: u64,
    pub following_count: u64,
    pub is_following: bool,
    pub is_followed_by_caller: bool,
}

impl Default for UserSocialResponse {
    fn default() -> Self {
        Self {
            principal: Principal::anonymous(),
            username: String::new(),
            handle: String::new(),
            avatar: String::new(),
            bio: String::new(),
            interests: Vec::new(),
            followers_count: 0,
            following_count: 0,
            is_following: false,
            is_followed_by_caller: false,
        }
    }
}

#[derive(CandidType, Deserialize, Clone)]
pub struct UserStatsResponse {
    pub post_count: u64,
    pub comment_count: u64,
    pub like_count: u64,
    pub points: u64,
    pub reputation: u64,
}

#[derive(CandidType, Deserialize, Clone)]
pub struct UserResponse {
    pub principal: Principal,
    pub username: String,
    pub handle: String,
    pub bio: String,
    pub avatar: String,
    pub social_links: Vec<(String, String)>,
    pub interests: Vec<String>,
    pub followers_count: u64,
    pub following_count: u64,
    pub created_at: u64,
    pub updated_at: u64,
    pub stats: UserStatsResponse,
    pub status: UserStatus,
    pub role: UserRole,
    pub is_following: bool,
}

#[derive(CandidType, Deserialize, Clone)]
pub struct UserLeaderboardItem {
    pub principal: Principal,
    pub username: String,
    pub handle: String,
    pub avatar: String,
    pub rank: u64,
    pub last_claim_date: u64,
    pub consecutive_daily_logins: u64,
    pub followers_count: u64,
    pub post_count: u64,
    pub comment_count: u64,
    pub like_count: u64,
    pub reputation: u64,
}

#[derive(CandidType, Deserialize, Clone)]
pub struct UserLeaderboardResponse {
    pub users: Vec<UserLeaderboardItem>,
    pub total_users: u64,
    pub has_more: bool,
    pub next_offset: u64,
}

#[derive(CandidType, Deserialize, Clone, PartialEq, Default)]
pub enum UserStatus {
    #[default]
    Active,
    Suspended,
    Banned,
    Restricted,
}

#[derive(CandidType, Deserialize, Clone, PartialEq, Default)]
pub enum UserRole {
    #[default]
    User,
    Admin,
    Moderator,
    Creator,
}
