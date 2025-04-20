use std::cell::RefCell;
use crate::storage::STORAGE;
use crate::storage::sharded::*;
use crate::storage::sharded::ShardedStorage;
use ic_cdk::api::time;
use ic_cdk::storage::{stable_save, stable_restore};
use ic_cdk::api::trap;
use candid::{CandidType, Deserialize};

// Stable storage version for compatibility checks
const STABLE_STORAGE_VERSION: u32 = 1;

#[derive(CandidType, Deserialize)]
struct StableStorageHeader {
    version: u32,
    timestamp: u64,
}

// This function migrates all data to the sharded storage system
pub fn migrate_all() {
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
        
        // Migrate articles
        SHARDED_ARTICLES.with(|articles| {
            let mut articles_store = articles.borrow_mut();
            for (article_id, article) in &store.articles {
                articles_store.insert(article_id.clone(), article.clone());
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
    ic_cdk::println!("Saving state to stable storage...");
    
    // Save header with version and timestamp
    let header = StableStorageHeader {
        version: STABLE_STORAGE_VERSION,
        timestamp: time(),
    };
    
    // Save header first
    match stable_save((header,)) {
        Ok(_) => ic_cdk::println!("Saved storage header successfully"),
        Err(e) => {
            let error_msg = format!("Failed to save storage header: {:?}", e);
            ic_cdk::println!("{}", error_msg);
            trap(&error_msg);
        }
    }
    
    // Save users data
    save_users_data();
    
    // Save content data
    save_content_data();
    
    // Save interaction data
    save_interaction_data();
    
    // Save rewards data
    save_rewards_data();
    
    ic_cdk::println!("State saved successfully");
}

// Restore state from stable storage after canister upgrade
pub fn restore_state_after_upgrade() {
    ic_cdk::println!("Restoring state from stable storage...");
    
    // Restore header first to check version compatibility
    let header: Result<(StableStorageHeader,), String> = stable_restore();
    
    match header {
        Ok((header,)) => {
            if header.version != STABLE_STORAGE_VERSION {
                let msg = format!("Stable storage version mismatch: expected {}, found {}", 
                                  STABLE_STORAGE_VERSION, header.version);
                ic_cdk::println!("{}", msg);
                // Continue anyway, but log the warning
            }
            ic_cdk::println!("Restored storage header, timestamp: {}", header.timestamp);
        },
        Err(e) => {
            let msg = format!("Failed to restore storage header: {:?}", e);
            ic_cdk::println!("{}", msg);
            // If we can't restore the header, we shouldn't try to restore the rest
            return;
        }
    }
    
    // Restore users data
    restore_users_data();
    
    // Restore content data
    restore_content_data();
    
    // Restore interaction data
    restore_interaction_data();
    
    // Restore rewards data
    restore_rewards_data();
    
    ic_cdk::println!("State restored successfully");
}

// Save users data to stable storage
fn save_users_data() {
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
}

// Restore users data from stable storage
fn restore_users_data() {
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
}

// Save content data to stable storage
fn save_content_data() {
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
    
    // Save articles
    let articles_data = SHARDED_ARTICLES.with(|articles| {
        articles.borrow_mut().items()
    });
    
    let articles_count = articles_data.len();
    match stable_save((articles_data,)) {
        Ok(_) => ic_cdk::println!("Saved {} articles", articles_count),
        Err(e) => {
            let error_msg = format!("Failed to save articles: {:?}", e);
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
}

// Restore content data from stable storage
fn restore_content_data() {
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
    
    // Restore articles
    let articles_result: Result<(Vec<(String, crate::storage::Article)>,), String> = stable_restore();
    
    match articles_result {
        Ok((articles_data,)) => {
            let articles_count = articles_data.len();
            SHARDED_ARTICLES.with(|articles| {
                let mut articles_store = articles.borrow_mut();
                for (key, value) in articles_data {
                    articles_store.insert(key, value);
                }
            });
            ic_cdk::println!("Restored {} articles", articles_count);
        },
        Err(e) => {
            let error_msg = format!("Failed to restore articles: {:?}", e);
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
}

// Save interaction data to stable storage
fn save_interaction_data() {
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
}

// Restore interaction data from stable storage
fn restore_interaction_data() {
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
}

// Save rewards data to stable storage
fn save_rewards_data() {
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
}

// Restore rewards data from stable storage
fn restore_rewards_data() {
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
}
