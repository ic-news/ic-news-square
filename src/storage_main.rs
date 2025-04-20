use std::cell::RefCell;
use std::collections::{HashMap, HashSet, BTreeMap};
use candid::{CandidType, Deserialize, Principal};

use crate::models::reward::TaskDefinition;
use crate::models::interaction::ContentReport;
use crate::models::content::NewsReferenceRequest;

// News reference for storage
#[derive(CandidType, Deserialize, Clone)]
pub struct NewsReference {
    pub metadata: Vec<(String, String)>,
    pub canister_id: Principal,
}

thread_local! {
    pub static STORAGE: RefCell<Storage> = RefCell::new(Storage::default());
}

impl Default for Storage {
    fn default() -> Self {
        Storage {
            users: HashMap::new(),
            user_profiles: HashMap::new(),
            user_stats: HashMap::new(),
            user_rewards: HashMap::new(),
            user_tasks: HashMap::new(),
            tasks: HashMap::new(),
            posts: HashMap::new(),
            articles: HashMap::new(),
            comments: HashMap::new(),
            likes: HashMap::new(),
            shares: HashMap::new(),
            user_posts: HashMap::new(),
            user_articles: HashMap::new(),
            user_comments: HashMap::new(),
            trending_topics: BTreeMap::new(),
            previous_trending_topics: BTreeMap::new(),
            trending_content: Vec::new(),
            content_counter: 0,
            last_trending_update: 0,
            community_guidelines: String::new(),
            terms_of_service: String::new(),
            admin: None,
            managers: HashSet::new(),
            reports: HashMap::new(),
            user_notifications: HashMap::new(),
        }
    }
}

#[derive(CandidType, Deserialize)]
pub struct Storage {
    // Admin and managers
    pub admin: Option<Principal>,
    pub managers: HashSet<Principal>,
    
    // Content storage
    pub posts: HashMap<String, Post>,
    pub articles: HashMap<String, Article>,
    pub comments: HashMap<String, Comment>,
    
    // User data
    pub users: HashMap<Principal, User>,
    pub user_profiles: HashMap<Principal, UserProfile>,
    pub user_stats: HashMap<Principal, UserStats>,
    
    // Content indexing
    pub user_posts: HashMap<Principal, Vec<String>>,
    pub user_articles: HashMap<Principal, Vec<String>>,
    pub user_comments: HashMap<Principal, Vec<String>>,
    
    // Interactions
    pub likes: HashMap<String, HashSet<Principal>>,
    pub shares: HashMap<String, u64>,
    pub reports: HashMap<String, ContentReport>,
    
    // Discovery
    pub trending_topics: BTreeMap<String, u64>, // hashtag -> count
    pub previous_trending_topics: BTreeMap<String, u64>, // previous period hashtag -> count
    pub trending_content: Vec<String>, // content IDs
    
    // Rewards and tasks
    pub user_rewards: HashMap<Principal, UserRewards>,
    pub user_tasks: HashMap<Principal, UserTasks>,
    pub tasks: HashMap<String, TaskDefinition>,
    
    // System data
    pub content_counter: u64,
    pub last_trending_update: u64,
    pub community_guidelines: String,
    pub terms_of_service: String,
    
    // Notifications
    pub user_notifications: HashMap<Principal, Vec<crate::models::user::UserNotification>>,
}

// Content types
#[derive(CandidType, Deserialize, Clone)]
pub struct Post {
    pub id: String,
    pub author: Principal,
    pub content: String,
    pub media_urls: Vec<String>,
    pub hashtags: Vec<String>,
    pub token_mentions: Vec<String>,
    pub tags: Vec<String>,  // Max 5 tags
    pub created_at: u64,
    pub updated_at: u64,
    pub status: ContentStatus,
    pub visibility: ContentVisibility,
    pub news_reference: Option<NewsReference>,
}

#[derive(CandidType, Deserialize, Clone)]
pub struct Article {
    pub id: String,
    pub author: Principal,
    pub content: String,
    pub media_urls: Vec<String>,
    pub hashtags: Vec<String>,
    pub token_mentions: Vec<String>,
    pub created_at: u64,
    pub updated_at: u64,
    pub status: ContentStatus,
    pub visibility: ContentVisibility,
    pub news_reference: Option<NewsReference>,
}

#[derive(CandidType, Deserialize, Clone)]
pub struct Comment {
    pub id: String,
    pub author: Principal,
    pub content: String,
    pub parent_id: String, // ID of the post, article, or comment this is replying to
    pub parent_type: ParentType,
    pub created_at: u64,
    pub updated_at: u64,
    pub status: ContentStatus,
    pub child_comments: Vec<String>, // IDs of child comments
    pub likes_count: u64,
}

// User data structures
#[derive(CandidType, Deserialize, Clone)]
pub struct User {
    pub principal: Principal,
    pub registered_at: u64,
    pub last_login: u64,
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
    pub social_links: Option<Vec<(String, String)>>,
    pub interests: Vec<String>,
    pub followed_users: HashSet<Principal>,
    pub followers: HashSet<Principal>,
    pub followed_topics: HashSet<String>,
    pub privacy_settings: Option<crate::models::user::UserPrivacySettings>,
    pub updated_at: u64,
}

#[derive(CandidType, Deserialize, Clone)]
pub struct UserStats {
    pub principal: Principal,
    pub post_count: u64,
    pub article_count: u64,
    pub comment_count: u64,
    pub likes_received: u64,
    pub shares_received: u64,
    pub views_received: u64,
    pub share_count: u64,
}

// Rewards and tasks
#[derive(CandidType, Deserialize, Clone)]
pub struct UserRewards {
    pub principal: Principal,
    pub points: u64,
    pub points_history: Vec<PointsTransaction>,
    pub last_claim_date: Option<u64>,
    // Note: consecutive_daily_logins field has been moved to daily_checkin_task canister
    // and is now tracked as consecutive_days there
    pub transactions: Vec<PointsTransaction>,
    pub last_updated: u64,
}

#[derive(CandidType, Deserialize, Clone)]
pub struct PointsTransaction {
    pub amount: i64,
    pub reason: String,
    pub timestamp: u64,
    pub reference_id: Option<String>, // Content ID or task ID
    pub points: u64,
}

#[derive(CandidType, Deserialize, Clone)]
pub struct UserTasks {
    pub principal: Principal,
    pub completed_tasks: HashMap<String, u64>, // task_id -> completion timestamp
    pub daily_tasks_reset: u64, // Timestamp when daily tasks were last reset
    pub last_check_in: Option<u64>, // Last daily check-in timestamp
    pub last_updated: u64,
}

// Enums
#[derive(CandidType, Deserialize, Clone, PartialEq)]
pub enum MarketSentiment {
    Bullish,
    Bearish,
    Neutral,
}

#[derive(CandidType, Deserialize, Clone, PartialEq, Debug)]
pub enum ContentStatus {
    Active,
    UnderReview,
    Removed,
    Hidden,
    Deleted,
}

#[derive(CandidType, Deserialize, Clone, PartialEq, Debug)]
pub enum ContentVisibility {
    Public,
    FollowersOnly,
    Private,
}

#[derive(CandidType, Deserialize, Clone, PartialEq, Copy, Debug)]
pub enum ParentType {
    Post,
    Article,
    Comment,
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
