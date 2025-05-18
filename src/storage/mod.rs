pub mod migration;

use std::cell::RefCell;
use std::collections::{HashMap, HashSet, BTreeMap};
use candid::{CandidType, Deserialize, Principal};

use crate::models::reward::TaskDefinition;
use crate::models::interaction::ContentReport;
use crate::models::content::NewsReference;
use crate::models::storage::Storage;
// Re-export models for backward compatibility
pub use crate::models::content::{Post, Comment, ContentStatus, ParentType, ContentVisibility};
pub use crate::models::user::{User, UserProfile, UserStats, UserStatus, UserRole};
pub use crate::models::reward::{UserRewards, UserTasks};


thread_local! {
    pub static STORAGE: RefCell<Storage> = RefCell::new(Storage::default());
}

impl Default for Storage {
    fn default() -> Self {
        Storage {
            bark_api_key: String::new(),
            users: HashMap::new(),
            user_profiles: Some(HashMap::new()),
            user_stats: Some(HashMap::new()),
            user_rewards: HashMap::new(),
            user_tasks: HashMap::new(),
            tasks: Some(HashMap::new()),
            posts: HashMap::new(),
            comments: HashMap::new(),
            likes: HashMap::new(),
            user_posts: HashMap::new(),
            user_comments: HashMap::new(),
            trending_topics: BTreeMap::new(),
            previous_trending_topics: BTreeMap::new(),
            trending_content: Vec::new(),
            content_counter: Some(0),
            last_trending_update: Some(0),
            community_guidelines: Some(String::from("Default community guidelines")),
            terms_of_service: Some(String::from("Default terms of service")),
            admin: None,
            managers: Some(HashSet::new()),
            reports: HashMap::new(),
            user_notifications: HashMap::new(),
            heartbeat_interval_hours: 6, // Default to 6 hours
        }
    }
}
