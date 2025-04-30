use std::cell::RefCell;
use crate::storage::STORAGE;
use crate::storage::sharded::*;
use crate::storage::sharded::ShardedStorage;
use ic_cdk::api::time;
use ic_cdk::storage::{stable_save, stable_restore};
use ic_cdk::api::trap;
use candid::{CandidType, Deserialize};
use crate::storage::migration_sync::{synchronize_storage_before_upgrade, save_main_storage_backup, restore_from_main_storage_backup, synchronize_storage_after_upgrade};

// Stable storage version for compatibility checks
const STABLE_STORAGE_VERSION: u32 = 1;

#[derive(CandidType, Deserialize)]
struct StableStorageHeader {
    version: u32,
    timestamp: u64,
}

// This function migrates all data to the sharded storage system
pub fn migrate_all() -> bool {
    // Check if migration has already been performed
    let migration_needed = STORAGE.with(|storage| {
        let store = storage.borrow();
        // If storage is empty, we don't need migration
        // If it has data but sharded storage is empty, we need migration
        !store.users.is_empty() && SHARDED_POSTS.with(|posts| {
            posts.borrow().key_to_shard.is_empty()
        })
    });

    if migration_needed {
        let start_time = time();
        
        migrate_users();
        migrate_content();
        migrate_interactions();
        migrate_rewards();
        migrate_discovery();
        
        let end_time = time();
        let _duration_ms = (end_time - start_time) / 1_000_000;
    }
    
    true
}

// Migrate user data to sharded storage
pub fn migrate_users() {
    // Migrate user profiles
    STORAGE.with(|storage| {
        let store = storage.borrow();
        
        // Migrate users
        SHARDED_USERS.with(|users| {
            let mut users_store = users.borrow_mut();
            for (principal, user) in &store.users {
                users_store.insert(principal.to_string(), user.clone());
            }
        });
        
        // Migrate user profiles
        SHARDED_USER_PROFILES.with(|profiles| {
            let mut profiles_store = profiles.borrow_mut();
            for (principal, profile) in &store.user_profiles {
                profiles_store.insert(principal.to_string(), profile.clone());
            }
        });
        
        // Migrate user stats
        SHARDED_USER_STATS.with(|stats| {
            let mut stats_store = stats.borrow_mut();
            for (principal, user_stats) in &store.user_stats {
                stats_store.insert(principal.to_string(), user_stats.clone());
            }
        });
    });
}

// Migrate content data to sharded storage
fn migrate_content() {
    
    STORAGE.with(|storage| {
        let store = storage.borrow();
        
        // Migrate posts
        SHARDED_POSTS.with(|posts| {
            let mut posts_store = posts.borrow_mut();
            for (post_id, post) in &store.posts {
                posts_store.insert(post_id.clone(), post.clone());
            }
        });
        
        // Migrate comments
        SHARDED_COMMENTS.with(|comments| {
            let mut comments_store = comments.borrow_mut();
            for (comment_id, comment) in &store.comments {
                comments_store.insert(comment_id.clone(), comment.clone());
            }
        });
    });
}

// Migrate interaction data to sharded storage
fn migrate_interactions() {
    STORAGE.with(|storage| {
        let store = storage.borrow();
        
        // Migrate likes
        SHARDED_LIKES.with(|likes| {
            let mut likes_store = likes.borrow_mut();
            for (content_id, users) in &store.likes {
                for user in users {
                    likes_store.add_like(content_id, *user);
                }
            }
        });
    });
}

// Migrate reward data to sharded storage
fn migrate_rewards() {
    STORAGE.with(|storage| {
        let store = storage.borrow();
        
        // Migrate user rewards
        SHARDED_USER_REWARDS.with(|rewards| {
            let mut rewards_store = rewards.borrow_mut();
            for (principal, user_rewards) in &store.user_rewards {
                rewards_store.insert(principal.to_string(), user_rewards.clone());
            }
        });
        
        // Migrate user tasks
        SHARDED_USER_TASKS.with(|tasks| {
            let mut tasks_store = tasks.borrow_mut();
            for (principal, user_tasks) in &store.user_tasks {
                tasks_store.insert(principal.to_string(), user_tasks.clone());
            }
        });
    });
}

// Migrate discovery data to sharded storage
fn migrate_discovery() {
    // Create a new sharded storage for trending topics if needed
    thread_local! {
        static SHARDED_TRENDING_TOPICS: RefCell<ShardedStorage<u64>> = 
            RefCell::new(ShardedStorage::default());
            
        static SHARDED_TRENDING_CONTENT: RefCell<ShardedStorage<String>> = 
            RefCell::new(ShardedStorage::default());
    }
    
    STORAGE.with(|storage| {
        let store = storage.borrow();
        
        // Migrate trending topics
        for (topic, count) in &store.trending_topics {
            SHARDED_TRENDING_TOPICS.with(|topics| {
                let mut topics_store = topics.borrow_mut();
                topics_store.insert(topic.clone(), *count);
            });
        }
        
        // Migrate trending content
        for (index, content_id) in store.trending_content.iter().enumerate() {
            SHARDED_TRENDING_CONTENT.with(|content| {
                let mut content_store = content.borrow_mut();
                content_store.insert(index.to_string(), content_id.clone());
            });
        }
        
        // Migrate last trending update timestamp
        thread_local! {
            static LAST_TRENDING_UPDATE: RefCell<u64> = RefCell::new(0);
        }
        
        LAST_TRENDING_UPDATE.with(|last_update| {
            *last_update.borrow_mut() = store.last_trending_update;
        });
    });
}

// Save state to stable storage before canister upgrade
pub fn save_state_for_upgrade() {
    ic_cdk::println!("========== SAVING STATE TO STABLE STORAGE ==========");
    
    let stable_size_before = ic_cdk::api::stable::stable_size();
    ic_cdk::println!("Initial stable storage size: {} pages ({} bytes)", 
        stable_size_before, 
        stable_size_before * 65536);
    
    // Save header with version and timestamp
    let header = StableStorageHeader {
        version: STABLE_STORAGE_VERSION,
        timestamp: time(),
    };
    
    crate::utils::logger::log(&format!("Saving storage header with version {} and timestamp {}", 
        header.version, header.timestamp));
    
    // Save header first
    match stable_save((header,)) {
        Ok(_) => crate::utils::logger::log("✅ Saved storage header successfully"),
        Err(e) => {
            let error_msg = format!("❌ CRITICAL ERROR: Failed to save storage header: {:?}", e);
            crate::utils::logger::log(&error_msg);
            trap(&error_msg);
        }
    }
    
    crate::utils::logger::log("Checking data state before synchronization:");
    STORAGE.with(|storage| {
        let store = storage.borrow();
        crate::utils::logger::log(&format!("Main storage: {} users, {} posts, {} comments", 
            store.users.len(), store.posts.len(), store.comments.len()));
    });
    
    let sharded_users_count = SHARDED_USERS.with(|users| users.borrow().keys().len());
    let sharded_posts_count = SHARDED_POSTS.with(|posts| posts.borrow().keys().len());
    let sharded_comments_count = SHARDED_COMMENTS.with(|comments| comments.borrow().keys().len());
    
    crate::utils::logger::log(&format!("Sharded storage: {} users, {} posts, {} comments", 
        sharded_users_count, sharded_posts_count, sharded_comments_count));
    
    // First, ensure all data is synchronized between main storage and sharded storage
    crate::utils::logger::log("Synchronizing data between main storage and sharded storage...");
    synchronize_storage_before_upgrade();
    
    let sharded_users_count_after = SHARDED_USERS.with(|users| users.borrow().keys().len());
    let sharded_posts_count_after = SHARDED_POSTS.with(|posts| posts.borrow().keys().len());
    let sharded_comments_count_after = SHARDED_COMMENTS.with(|comments| comments.borrow().keys().len());
    
    crate::utils::logger::log(&format!("After synchronization - Sharded storage: {} users, {} posts, {} comments", 
        sharded_users_count_after, sharded_posts_count_after, sharded_comments_count_after));
    
    // Save users data
    crate::utils::logger::log("Saving users data...");
    let users_saved = save_users_data();
    crate::utils::logger::log(&format!("Users data saved: {}", if users_saved { "✅ Success" } else { "❌ Failed" }));
    
    // Save content data
    crate::utils::logger::log("Saving content data...");
    let content_saved = save_content_data();
    crate::utils::logger::log(&format!("Content data saved: {}", if content_saved { "✅ Success" } else { "❌ Failed" }));
    
    // Save interaction data
    crate::utils::logger::log("Saving interaction data...");
    let interaction_saved = save_interaction_data();
    crate::utils::logger::log(&format!("Interaction data saved: {}", if interaction_saved { "✅ Success" } else { "❌ Failed" }));
    
    // Save rewards data
    crate::utils::logger::log("Saving rewards data...");
    let rewards_saved = save_rewards_data();
    crate::utils::logger::log(&format!("Rewards data saved: {}", if rewards_saved { "✅ Success" } else { "❌ Failed" }));
    
    // Also save main storage directly as a backup
    crate::utils::logger::log("Saving main storage backup...");
    save_main_storage_backup();
    crate::utils::logger::log("Main storage backup saved");
    
    let stable_size_after = ic_cdk::api::stable::stable_size();
    crate::utils::logger::log(&format!("Final stable storage size: {} pages ({} bytes)", 
        stable_size_after, 
        stable_size_after * 65536));
    
    crate::utils::logger::log("========== STATE SAVED SUCCESSFULLY ==========");
}

// Restore state from stable storage after canister upgrade
pub fn restore_state_after_upgrade() {
    crate::utils::logger::log("========== RESTORING STATE FROM STABLE STORAGE ==========");
    
    let stable_size = ic_cdk::api::stable::stable_size();
    crate::utils::logger::log(&format!("Current stable storage size: {} pages ({} bytes)", 
        stable_size, 
        stable_size * 65536));
    
    crate::utils::logger::log("Data state before restoration:");
    STORAGE.with(|storage| {
        let store = storage.borrow();
        crate::utils::logger::log(&format!("Main storage: {} users, {} posts, {} comments", 
            store.users.len(), store.posts.len(), store.comments.len()));
    });
    
    let sharded_users_count = SHARDED_USERS.with(|users| users.borrow().keys().len());
    let sharded_posts_count = SHARDED_POSTS.with(|posts| posts.borrow().keys().len());
    let sharded_comments_count = SHARDED_COMMENTS.with(|comments| comments.borrow().keys().len());
    
    crate::utils::logger::log(&format!("Sharded storage: {} users, {} posts, {} comments", 
        sharded_users_count, sharded_posts_count, sharded_comments_count));
    
    // Restore header first to check version compatibility
    crate::utils::logger::log("Restoring storage header...");
    let header: Result<(StableStorageHeader,), String> = stable_restore();
    
    match header {
        Ok((header,)) => {
            if header.version != STABLE_STORAGE_VERSION {
                let msg = format!("Stable storage version mismatch: expected {}, found {}", STABLE_STORAGE_VERSION, header.version);
                crate::utils::logger::log(&format!("⚠️ WARNING: {}", msg));
            }
            crate::utils::logger::log(&format!("✅ Restored storage header, timestamp: {}", header.timestamp));
            
            let current_time = time() / 1_000_000;
            let time_diff_seconds = (current_time - header.timestamp / 1_000_000) as i64;
            let time_diff_minutes = time_diff_seconds / 60;
            let time_diff_hours = time_diff_minutes / 60;
            
            crate::utils::logger::log(&format!("Data was saved {} hours {} minutes ago ({})", 
                time_diff_hours, time_diff_minutes % 60, header.timestamp / 1_000_000));
        },
        Err(e) => {
            let msg = format!("❌ CRITICAL ERROR: Failed to restore storage header: {:?}", e);
            crate::utils::logger::log(&msg);
            crate::utils::logger::log("Attempting to restore from main storage backup...");
            if !restore_from_main_storage_backup() {
                crate::utils::logger::log("❌ Failed to restore from backup, initializing empty storage");
                return;
            }
            return;
        }
    }
    
    // Try to restore data from sharded storage first
    crate::utils::logger::log("Attempting to restore data from sharded storage...");
    let mut restoration_successful = true;
    
    crate::utils::logger::log("Restoring users data...");
    if !restore_users_data() {
        crate::utils::logger::log("❌ Failed to restore users data");
        restoration_successful = false;
    } else {
        crate::utils::logger::log("✅ Successfully restored users data");
    }
    
    crate::utils::logger::log("Restoring content data...");
    if !restore_content_data() {
        crate::utils::logger::log("❌ Failed to restore content data");
        restoration_successful = false;
    } else {
        crate::utils::logger::log("✅ Successfully restored content data");
    }
    
    crate::utils::logger::log("Restoring interaction data...");
    if !restore_interaction_data() {
        crate::utils::logger::log("❌ Failed to restore interaction data");
        restoration_successful = false;
    } else {
        crate::utils::logger::log("✅ Successfully restored interaction data");
    }
    
    crate::utils::logger::log("Restoring rewards data...");
    if !restore_rewards_data() {
        crate::utils::logger::log("❌ Failed to restore rewards data");
        restoration_successful = false;
    } else {
        crate::utils::logger::log("✅ Successfully restored rewards data");
    }
    
    crate::utils::logger::log("Data state after sharded storage restoration:");
    let sharded_users_count_after = SHARDED_USERS.with(|users| users.borrow().keys().len());
    let sharded_posts_count_after = SHARDED_POSTS.with(|posts| posts.borrow().keys().len());
    let sharded_comments_count_after = SHARDED_COMMENTS.with(|comments| comments.borrow().keys().len());
    
    crate::utils::logger::log(&format!("Sharded storage: {} users, {} posts, {} comments", 
        sharded_users_count_after, sharded_posts_count_after, sharded_comments_count_after));
    
    // If any restoration failed, try to restore from main storage backup
    if !restoration_successful {
        crate::utils::logger::log("⚠️ Some data restoration failed, attempting to restore from main storage backup...");
        restore_from_main_storage_backup();
    } else {
        crate::utils::logger::log("✅ State restored successfully from sharded storage");
    }
    
    crate::utils::logger::log("Synchronizing storage after upgrade...");
    // Synchronize storage after restoration to ensure consistency
    ic_cdk::println!("Synchronizing storage after restoration...");
    synchronize_storage_after_upgrade();
    
    ic_cdk::println!("Data state after synchronization:");
    STORAGE.with(|storage| {
        let store = storage.borrow();
        ic_cdk::println!("Main storage: {} users, {} posts, {} comments", 
            store.users.len(), store.posts.len(), store.comments.len());
    });
    
    let final_sharded_users_count = SHARDED_USERS.with(|users| users.borrow().keys().len());
    let final_sharded_posts_count = SHARDED_POSTS.with(|posts| posts.borrow().keys().len());
    let final_sharded_comments_count = SHARDED_COMMENTS.with(|comments| comments.borrow().keys().len());
    
    ic_cdk::println!("Sharded storage: {} users, {} posts, {} comments", 
        final_sharded_users_count, final_sharded_posts_count, final_sharded_comments_count);
    
    ic_cdk::println!("========== STATE RESTORATION COMPLETED ==========");
}

// Save users data to stable storage
fn save_users_data() -> bool {
    // Save users
    let users_data = SHARDED_USERS.with(|users| {
        users.borrow_mut().items()
    });
    
    let users_count = users_data.len();
    match stable_save((users_data,)) {
        Ok(_) => ic_cdk::println!("Saved {} users", users_count),
        Err(e) => {
            let error_msg = format!("Failed to save users: {:?}", e);
            ic_cdk::println!("{}", error_msg);
        }
    }
    
    // Save user profiles
    let profiles_data = SHARDED_USER_PROFILES.with(|profiles| {
        profiles.borrow_mut().items()
    });
    
    let profiles_count = profiles_data.len();
    match stable_save((profiles_data,)) {
        Ok(_) => ic_cdk::println!("Saved {} user profiles", profiles_count),
        Err(e) => {
            let error_msg = format!("Failed to save user profiles: {:?}", e);
            ic_cdk::println!("{}", error_msg);
        }
    }
    
    // Save user stats
    let stats_data = SHARDED_USER_STATS.with(|stats| {
        stats.borrow_mut().items()
    });
    
    let stats_count = stats_data.len();
    match stable_save((stats_data,)) {
        Ok(_) => ic_cdk::println!("Saved {} user stats", stats_count),
        Err(e) => {
            let error_msg = format!("Failed to save user stats: {:?}", e);
            ic_cdk::println!("{}", error_msg);
        }
    }
    
    true
}

// Restore users data from stable storage
fn restore_users_data() -> bool {
    // Restore users
    let users_result: Result<(Vec<(String, crate::storage::User)>,), String> = stable_restore();
    
    match users_result {
        Ok((users_data,)) => {
            let users_count = users_data.len();
            SHARDED_USERS.with(|users| {
                let mut users_store = users.borrow_mut();
                for (key, value) in users_data {
                    users_store.insert(key, value);
                }
            });
            ic_cdk::println!("Restored {} users", users_count);
        },
        Err(e) => {
            let error_msg = format!("Failed to restore users: {:?}", e);
            ic_cdk::println!("{}", error_msg);
        }
    }
    
    // Restore user profiles
    let profiles_result: Result<(Vec<(String, crate::storage::UserProfile)>,), String> = stable_restore();
    
    match profiles_result {
        Ok((profiles_data,)) => {
            let profiles_count = profiles_data.len();
            SHARDED_USER_PROFILES.with(|profiles| {
                let mut profiles_store = profiles.borrow_mut();
                for (key, value) in profiles_data {
                    profiles_store.insert(key, value);
                }
            });
            ic_cdk::println!("Restored {} user profiles", profiles_count);
        },
        Err(e) => {
            let error_msg = format!("Failed to restore user profiles: {:?}", e);
            ic_cdk::println!("{}", error_msg);
        }
    }
    
    // Restore user stats
    let stats_result: Result<(Vec<(String, crate::storage::UserStats)>,), String> = stable_restore();
    
    match stats_result {
        Ok((stats_data,)) => {
            let stats_count = stats_data.len();
            SHARDED_USER_STATS.with(|stats| {
                let mut stats_store = stats.borrow_mut();
                for (key, value) in stats_data {
                    stats_store.insert(key, value);
                }
            });
            ic_cdk::println!("Restored {} user stats", stats_count);
        },
        Err(e) => {
            let error_msg = format!("Failed to restore user stats: {:?}", e);
            ic_cdk::println!("{}", error_msg);
        }
    }
    
    true
}

// Save content data to stable storage
fn save_content_data() -> bool {
    // Save posts
    let posts_data = SHARDED_POSTS.with(|posts| {
        posts.borrow_mut().items()
    });
    
    let posts_count = posts_data.len();
    match stable_save((posts_data,)) {
        Ok(_) => ic_cdk::println!("Saved {} posts", posts_count),
        Err(e) => {
            let error_msg = format!("Failed to save posts: {:?}", e);
            ic_cdk::println!("{}", error_msg);
        }
    }
    
    // Save comments
    let comments_data = SHARDED_COMMENTS.with(|comments| {
        comments.borrow_mut().items()
    });
    
    let comments_count = comments_data.len();
    match stable_save((comments_data,)) {
        Ok(_) => ic_cdk::println!("Saved {} comments", comments_count),
        Err(e) => {
            let error_msg = format!("Failed to save comments: {:?}", e);
            ic_cdk::println!("{}", error_msg);
        }
    }
    
    true
}

// Restore content data from stable storage
fn restore_content_data() -> bool {
    // Restore posts
    let posts_result: Result<(Vec<(String, crate::storage::Post)>,), String> = stable_restore();
    
    match posts_result {
        Ok((posts_data,)) => {
            let posts_count = posts_data.len();
            SHARDED_POSTS.with(|posts| {
                let mut posts_store = posts.borrow_mut();
                for (key, value) in posts_data {
                    posts_store.insert(key, value);
                }
            });
            ic_cdk::println!("Restored {} posts", posts_count);
        },
        Err(e) => {
            let error_msg = format!("Failed to restore posts: {:?}", e);
            ic_cdk::println!("{}", error_msg);
        }
    }
    
    // Restore comments
    let comments_result: Result<(Vec<(String, crate::storage::Comment)>,), String> = stable_restore();
    
    match comments_result {
        Ok((comments_data,)) => {
            let comments_count = comments_data.len();
            SHARDED_COMMENTS.with(|comments| {
                let mut comments_store = comments.borrow_mut();
                for (key, value) in comments_data {
                    comments_store.insert(key, value);
                }
            });
            ic_cdk::println!("Restored {} comments", comments_count);
        },
        Err(e) => {
            let error_msg = format!("Failed to restore comments: {:?}", e);
            ic_cdk::println!("{}", error_msg);
        }
    }
    
    true
}

// Save interaction data to stable storage
fn save_interaction_data() -> bool {
    // For likes, we need a different approach since ShardedLikes has a custom structure
    // We'll extract all content IDs and their liked users
    let mut likes_data: Vec<(String, Vec<String>)> = Vec::new();
    
    // Get all content IDs with likes
    SHARDED_LIKES.with(|likes| {
        let mut likes_store = likes.borrow_mut();
        
        // For each content ID in the content_to_shard map
        for content_id in likes_store.content_ids() {
            // Get all users who liked this content
            let users = likes_store.get_likes(&content_id);
            
            // Convert Principal to String for storage
            let user_strings: Vec<String> = users.iter()
                .map(|principal| principal.to_string())
                .collect();
            
            // Add to our data structure
            likes_data.push((content_id, user_strings));
        }
    });
    
    let likes_count = likes_data.len();
    match stable_save((likes_data,)) {
        Ok(_) => ic_cdk::println!("Saved likes for {} content items", likes_count),
        Err(e) => {
            let error_msg = format!("Failed to save likes: {:?}", e);
            ic_cdk::println!("{}", error_msg);
        }
    }
    
    true
}

// Restore interaction data from stable storage
fn restore_interaction_data() -> bool {
    // Restore likes
    let likes_result: Result<(Vec<(String, Vec<String>)>,), String> = stable_restore();
    
    match likes_result {
        Ok((likes_data,)) => {
            let likes_count = likes_data.len();
            SHARDED_LIKES.with(|likes| {
                let mut likes_store = likes.borrow_mut();
                
                for (content_id, user_strings) in likes_data {
                    // Convert strings back to Principals
                    for user_str in user_strings {
                        match candid::Principal::from_text(&user_str) {
                            Ok(principal) => {
                                likes_store.add_like(&content_id, principal);
                            },
                            Err(e) => {
                                ic_cdk::println!("Error converting principal: {:?}", e);
                            }
                        }
                    }
                }
            });
            ic_cdk::println!("Restored likes for {} content items", likes_count);
        },
        Err(e) => {
            let error_msg = format!("Failed to restore likes: {:?}", e);
            ic_cdk::println!("{}", error_msg);
        }
    }
    
    true
}

// Save rewards data to stable storage
fn save_rewards_data() -> bool {
    // Save user rewards
    let rewards_data = SHARDED_USER_REWARDS.with(|rewards| {
        rewards.borrow_mut().items()
    });
    
    let rewards_count = rewards_data.len();
    match stable_save((rewards_data,)) {
        Ok(_) => ic_cdk::println!("Saved {} user rewards", rewards_count),
        Err(e) => {
            let error_msg = format!("Failed to save user rewards: {:?}", e);
            ic_cdk::println!("{}", error_msg);
        }
    }
    
    // Save user tasks
    let tasks_data = SHARDED_USER_TASKS.with(|tasks| {
        tasks.borrow_mut().items()
    });
    
    let tasks_count = tasks_data.len();
    match stable_save((tasks_data,)) {
        Ok(_) => ic_cdk::println!("Saved {} user tasks", tasks_count),
        Err(e) => {
            let error_msg = format!("Failed to save user tasks: {:?}", e);
            ic_cdk::println!("{}", error_msg);
        }
    }
    
    true
}

// Restore rewards data from stable storage
fn restore_rewards_data() -> bool {
    // Restore user rewards
    let rewards_result: Result<(Vec<(String, crate::storage::UserRewards)>,), String> = stable_restore();
    
    match rewards_result {
        Ok((rewards_data,)) => {
            let rewards_count = rewards_data.len();
            SHARDED_USER_REWARDS.with(|rewards| {
                let mut rewards_store = rewards.borrow_mut();
                for (key, value) in rewards_data {
                    rewards_store.insert(key, value);
                }
            });
            ic_cdk::println!("Restored {} user rewards", rewards_count);
        },
        Err(e) => {
            let error_msg = format!("Failed to restore user rewards: {:?}", e);
            ic_cdk::println!("{}", error_msg);
        }
    }
    
    // Restore user tasks
    let tasks_result: Result<(Vec<(String, crate::storage::UserTasks)>,), String> = stable_restore();
    
    match tasks_result {
        Ok((tasks_data,)) => {
            let tasks_count = tasks_data.len();
            SHARDED_USER_TASKS.with(|tasks| {
                let mut tasks_store = tasks.borrow_mut();
                for (key, value) in tasks_data {
                    tasks_store.insert(key, value);
                }
            });
            ic_cdk::println!("Restored {} user tasks", tasks_count);
        },
        Err(e) => {
            let error_msg = format!("Failed to restore user tasks: {:?}", e);
            ic_cdk::println!("{}", error_msg);
        }
    }
    
    true
}
