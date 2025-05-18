use std::cell::RefCell;
use std::collections::{HashMap, HashSet, BTreeMap};
use candid::{CandidType, Deserialize, Principal};

use crate::models::reward::TaskDefinition;
use crate::models::interaction::ContentReport;
use crate::models::content::{NewsReference, Post, Comment};
use crate::models::user::{User, UserProfile, UserStats};
use crate::models::reward::{UserRewards, UserTasks};
use crate::models::notification::UserNotification;

#[derive(CandidType, Deserialize, Clone)]
pub struct Storage {
    // Admin and managers
    pub admin: Option<Principal>,
    #[serde(default)]
    pub managers: Option<HashSet<Principal>>,
    pub bark_api_key: String,
    
    // Content storage
    pub posts: HashMap<String, Post>,
    pub comments: HashMap<String, Comment>,
    
    // User data
    pub users: HashMap<Principal, User>,
    #[serde(default)]
    pub user_profiles: Option<HashMap<Principal, UserProfile>>,
    #[serde(default)]
    pub user_stats: Option<HashMap<Principal, UserStats>>,
    
    // Content indexing
    pub user_posts: HashMap<Principal, Vec<String>>,
    pub user_comments: HashMap<Principal, Vec<String>>,
    
    // Interactions
    pub likes: HashMap<String, HashSet<Principal>>,
    pub reports: HashMap<String, ContentReport>,
    
    // Discovery
    pub trending_topics: BTreeMap<String, u64>, // hashtag -> count
    pub previous_trending_topics: BTreeMap<String, u64>, // previous period hashtag -> count
    pub trending_content: Vec<String>, // content IDs
    
    // Rewards and tasks
    pub user_rewards: HashMap<Principal, UserRewards>,
    pub user_tasks: HashMap<Principal, UserTasks>,
    #[serde(default)]
    pub tasks: Option<HashMap<String, TaskDefinition>>,
    
    // System data
    #[serde(default)]
    pub content_counter: Option<u64>,
    #[serde(default)]
    pub last_trending_update: Option<u64>,
    #[serde(default)]
    pub community_guidelines: Option<String>,
    #[serde(default)]
    pub terms_of_service: Option<String>,
    pub heartbeat_interval_hours: u64, // Configurable heartbeat interval in hours
    
    // Notifications
    pub user_notifications: HashMap<Principal, Vec<UserNotification>>,
}
