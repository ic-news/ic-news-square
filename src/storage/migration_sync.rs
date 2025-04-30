use std::cell::RefCell;
use crate::storage::STORAGE;
use crate::storage::sharded::*;
use ic_cdk::api::time;
use ic_cdk::storage::{stable_save, stable_restore};
use ic_cdk::api::trap;

// Synchronize data between main storage and sharded storage before upgrade
pub fn synchronize_storage_before_upgrade() {
    ic_cdk::println!("========== SYNCHRONIZING STORAGE BEFORE UPGRADE ==========");
    
    ic_cdk::println!("Storage state before synchronization:");

    STORAGE.with(|storage| {
        let store = storage.borrow();
        ic_cdk::println!("Main storage counts:");
        ic_cdk::println!("- Users: {}", store.users.len());
        ic_cdk::println!("- User profiles: {}", store.user_profiles.len());
        ic_cdk::println!("- User stats: {}", store.user_stats.len());
        ic_cdk::println!("- Posts: {}", store.posts.len());
        ic_cdk::println!("- Comments: {}", store.comments.len());
        ic_cdk::println!("- Likes: {}", store.likes.len());
        ic_cdk::println!("- User rewards: {}", store.user_rewards.len());
        ic_cdk::println!("- User tasks: {}", store.user_tasks.len());
    });
    
    
    let sharded_users_count = SHARDED_USERS.with(|users| users.borrow().keys().len());
    let sharded_profiles_count = SHARDED_USER_PROFILES.with(|profiles| profiles.borrow().keys().len());
    let sharded_stats_count = SHARDED_USER_STATS.with(|stats| stats.borrow().keys().len());
    let sharded_posts_count = SHARDED_POSTS.with(|posts| posts.borrow().keys().len());
    let sharded_comments_count = SHARDED_COMMENTS.with(|comments| comments.borrow().keys().len());
    let sharded_rewards_count = SHARDED_USER_REWARDS.with(|rewards| rewards.borrow().keys().len());
    let sharded_tasks_count = SHARDED_USER_TASKS.with(|tasks| tasks.borrow().keys().len());
    
    ic_cdk::println!("Sharded storage counts:");
    ic_cdk::println!("- Users: {}", sharded_users_count);
    ic_cdk::println!("- User profiles: {}", sharded_profiles_count);
    ic_cdk::println!("- User stats: {}", sharded_stats_count);
    ic_cdk::println!("- Posts: {}", sharded_posts_count);
    ic_cdk::println!("- Comments: {}", sharded_comments_count);
    ic_cdk::println!("- User rewards: {}", sharded_rewards_count);
    ic_cdk::println!("- User tasks: {}", sharded_tasks_count);
    
    // Synchronize from main storage to sharded storage
    ic_cdk::println!("Starting synchronization from main storage to sharded storage...");
    
    STORAGE.with(|storage| {
        let store = storage.borrow();
        
        // Synchronize users
        ic_cdk::println!("Synchronizing users...");
        let users_before = SHARDED_USERS.with(|users| users.borrow().keys().len());
        SHARDED_USERS.with(|users| {
            let mut users_store = users.borrow_mut();
            for (principal, user) in &store.users {
                users_store.insert(principal.to_string(), user.clone());
            }
        });
        let users_after = SHARDED_USERS.with(|users| users.borrow().keys().len());
        ic_cdk::println!("Users synchronized: {} before, {} after", users_before, users_after);
        
        // Synchronize user profiles
        ic_cdk::println!("Synchronizing user profiles...");
        let profiles_before = SHARDED_USER_PROFILES.with(|profiles| profiles.borrow().keys().len());
        SHARDED_USER_PROFILES.with(|profiles| {
            let mut profiles_store = profiles.borrow_mut();
            for (principal, profile) in &store.user_profiles {
                profiles_store.insert(principal.to_string(), profile.clone());
            }
        });
        let profiles_after = SHARDED_USER_PROFILES.with(|profiles| profiles.borrow().keys().len());
        ic_cdk::println!("User profiles synchronized: {} before, {} after", profiles_before, profiles_after);
        
        // Synchronize user stats
        ic_cdk::println!("Synchronizing user stats...");
        let stats_before = SHARDED_USER_STATS.with(|stats| stats.borrow().keys().len());
        SHARDED_USER_STATS.with(|stats| {
            let mut stats_store = stats.borrow_mut();
            for (principal, user_stats) in &store.user_stats {
                stats_store.insert(principal.to_string(), user_stats.clone());
            }
        });
        let stats_after = SHARDED_USER_STATS.with(|stats| stats.borrow().keys().len());
        ic_cdk::println!("User stats synchronized: {} before, {} after", stats_before, stats_after);
        
        // Synchronize posts
        ic_cdk::println!("Synchronizing posts...");
        let posts_before = SHARDED_POSTS.with(|posts| posts.borrow().keys().len());
        SHARDED_POSTS.with(|posts| {
            let mut posts_store = posts.borrow_mut();
            for (post_id, post) in &store.posts {
                posts_store.insert(post_id.clone(), post.clone());
            }
        });
        let posts_after = SHARDED_POSTS.with(|posts| posts.borrow().keys().len());
        ic_cdk::println!("Posts synchronized: {} before, {} after", posts_before, posts_after);
        
        // Synchronize comments
        ic_cdk::println!("Synchronizing comments...");
        let comments_before = SHARDED_COMMENTS.with(|comments| comments.borrow().keys().len());
        SHARDED_COMMENTS.with(|comments| {
            let mut comments_store = comments.borrow_mut();
            for (comment_id, comment) in &store.comments {
                comments_store.insert(comment_id.clone(), comment.clone());
            }
        });
        let comments_after = SHARDED_COMMENTS.with(|comments| comments.borrow().keys().len());
        ic_cdk::println!("Comments synchronized: {} before, {} after", comments_before, comments_after);
        
        // Synchronize likes
        ic_cdk::println!("Synchronizing likes...");
        let likes_content_ids_before = SHARDED_LIKES.with(|likes| likes.borrow().content_ids().len());
        for (content_id, users) in &store.likes {
            SHARDED_LIKES.with(|likes| {
                let mut likes_store = likes.borrow_mut();
                for user in users {
                    likes_store.add_like(content_id, *user);
                }
            });
        }
        let likes_content_ids_after = SHARDED_LIKES.with(|likes| likes.borrow().content_ids().len());
        ic_cdk::println!("Likes synchronized: {} content IDs before, {} content IDs after", 
            likes_content_ids_before, likes_content_ids_after);
        
        // Synchronize user rewards
        ic_cdk::println!("Synchronizing user rewards...");
        let rewards_before = SHARDED_USER_REWARDS.with(|rewards| rewards.borrow().keys().len());
        SHARDED_USER_REWARDS.with(|rewards| {
            let mut rewards_store = rewards.borrow_mut();
            for (principal, user_rewards) in &store.user_rewards {
                rewards_store.insert(principal.to_string(), user_rewards.clone());
            }
        });
        let rewards_after = SHARDED_USER_REWARDS.with(|rewards| rewards.borrow().keys().len());
        ic_cdk::println!("User rewards synchronized: {} before, {} after", rewards_before, rewards_after);
        
        // Synchronize user tasks
        ic_cdk::println!("Synchronizing user tasks...");
        let tasks_before = SHARDED_USER_TASKS.with(|tasks| tasks.borrow().keys().len());
        SHARDED_USER_TASKS.with(|tasks| {
            let mut tasks_store = tasks.borrow_mut();
            for (principal, user_tasks) in &store.user_tasks {
                tasks_store.insert(principal.to_string(), user_tasks.clone());
            }
        });
        let tasks_after = SHARDED_USER_TASKS.with(|tasks| tasks.borrow().keys().len());
        ic_cdk::println!("User tasks synchronized: {} before, {} after", tasks_before, tasks_after);
    });
    
    
    ic_cdk::println!("Storage state after synchronization:");
    
    
    let sharded_users_count_after = SHARDED_USERS.with(|users| users.borrow().keys().len());
    let sharded_profiles_count_after = SHARDED_USER_PROFILES.with(|profiles| profiles.borrow().keys().len());
    let sharded_stats_count_after = SHARDED_USER_STATS.with(|stats| stats.borrow().keys().len());
    let sharded_posts_count_after = SHARDED_POSTS.with(|posts| posts.borrow().keys().len());
    let sharded_comments_count_after = SHARDED_COMMENTS.with(|comments| comments.borrow().keys().len());
    let sharded_rewards_count_after = SHARDED_USER_REWARDS.with(|rewards| rewards.borrow().keys().len());
    let sharded_tasks_count_after = SHARDED_USER_TASKS.with(|tasks| tasks.borrow().keys().len());
    
    ic_cdk::println!("Sharded storage counts after synchronization:");
    ic_cdk::println!("- Users: {}", sharded_users_count_after);
    ic_cdk::println!("- User profiles: {}", sharded_profiles_count_after);
    ic_cdk::println!("- User stats: {}", sharded_stats_count_after);
    ic_cdk::println!("- Posts: {}", sharded_posts_count_after);
    ic_cdk::println!("- Comments: {}", sharded_comments_count_after);
    ic_cdk::println!("- User rewards: {}", sharded_rewards_count_after);
    ic_cdk::println!("- User tasks: {}", sharded_tasks_count_after);
    
    ic_cdk::println!("========== STORAGE SYNCHRONIZED BEFORE UPGRADE ==========");
}

// Synchronize data between sharded storage and main storage after upgrade
pub fn synchronize_storage_after_upgrade() {
    ic_cdk::println!("========== SYNCHRONIZING STORAGE AFTER UPGRADE ==========");
    
    ic_cdk::println!("Storage state before synchronization:");
    
    STORAGE.with(|storage| {
        let store = storage.borrow();
        ic_cdk::println!("Main storage counts:");
        ic_cdk::println!("- Users: {}", store.users.len());
        ic_cdk::println!("- User profiles: {}", store.user_profiles.len());
        ic_cdk::println!("- User stats: {}", store.user_stats.len());
        ic_cdk::println!("- Posts: {}", store.posts.len());
        ic_cdk::println!("- Comments: {}", store.comments.len());
        ic_cdk::println!("- Likes: {}", store.likes.len());
        ic_cdk::println!("- User rewards: {}", store.user_rewards.len());
        ic_cdk::println!("- User tasks: {}", store.user_tasks.len());
    });
    
    let sharded_users_count = SHARDED_USERS.with(|users| users.borrow().keys().len());
    let sharded_profiles_count = SHARDED_USER_PROFILES.with(|profiles| profiles.borrow().keys().len());
    let sharded_stats_count = SHARDED_USER_STATS.with(|stats| stats.borrow().keys().len());
    let sharded_posts_count = SHARDED_POSTS.with(|posts| posts.borrow().keys().len());
    let sharded_comments_count = SHARDED_COMMENTS.with(|comments| comments.borrow().keys().len());
    let sharded_rewards_count = SHARDED_USER_REWARDS.with(|rewards| rewards.borrow().keys().len());
    let sharded_tasks_count = SHARDED_USER_TASKS.with(|tasks| tasks.borrow().keys().len());
    
    ic_cdk::println!("Sharded storage counts:");
    ic_cdk::println!("- Users: {}", sharded_users_count);
    ic_cdk::println!("- User profiles: {}", sharded_profiles_count);
    ic_cdk::println!("- User stats: {}", sharded_stats_count);
    ic_cdk::println!("- Posts: {}", sharded_posts_count);
    ic_cdk::println!("- Comments: {}", sharded_comments_count);
    ic_cdk::println!("- User rewards: {}", sharded_rewards_count);
    ic_cdk::println!("- User tasks: {}", sharded_tasks_count);
    
    let sharded_storage_empty = sharded_users_count == 0 && 
                              sharded_posts_count == 0 && 
                              sharded_comments_count == 0;
    
    if sharded_storage_empty {
        ic_cdk::println!("⚠️ WARNING: Sharded storage is empty! Skipping synchronization to preserve main storage data.");
        ic_cdk::println!("After synchronization, main storage has:");
        STORAGE.with(|storage| {
            let store = storage.borrow();
            ic_cdk::println!("- Users: {}", store.users.len());
            ic_cdk::println!("- Posts: {}", store.posts.len());
            ic_cdk::println!("- Comments: {}", store.comments.len());
        });
        ic_cdk::println!("After synchronization, sharded users count: {}", sharded_users_count);
        ic_cdk::println!("After synchronization, sharded posts count: {}", sharded_posts_count);
        return;
    }
    
    // Synchronize from sharded storage to main storage
    ic_cdk::println!("Starting synchronization from sharded storage to main storage...");
    
    STORAGE.with(|storage| {
        let mut store = storage.borrow_mut();
        
        // Clear main storage first to avoid duplicates
        ic_cdk::println!("Clearing main storage to avoid duplicates...");
        let users_before_clear = store.users.len();
        let posts_before_clear = store.posts.len();
        let comments_before_clear = store.comments.len();
        
        store.users.clear();
        store.user_profiles.clear();
        store.user_stats.clear();
        store.posts.clear();
        store.comments.clear();
        store.likes.clear();
        store.user_rewards.clear();
        store.user_tasks.clear();
        
        ic_cdk::println!("Main storage cleared: {} users, {} posts, {} comments removed", 
            users_before_clear, posts_before_clear, comments_before_clear);
        
        // Synchronize users
        ic_cdk::println!("Synchronizing users from sharded storage...");
        let mut users_synchronized = 0;
        let mut users_failed = 0;
        
        SHARDED_USERS.with(|users| {
            let mut users_store = users.borrow_mut();
            let keys: Vec<String> = users_store.keys();
            ic_cdk::println!("Found {} user keys in sharded storage", keys.len());
            
            for principal_str in keys {
                if let Some(user) = users_store.get(&principal_str) {
                    if let Ok(principal) = candid::Principal::from_text(&principal_str) {
                        store.users.insert(principal, user.clone());
                        users_synchronized += 1;
                    } else {
                        users_failed += 1;
                        ic_cdk::println!("Failed to parse principal: {}", principal_str);
                    }
                } else {
                    users_failed += 1;
                }
            }
        });
        
        ic_cdk::println!("Users synchronized: {} successful, {} failed", users_synchronized, users_failed);
        
        // Synchronize user profiles
        ic_cdk::println!("Synchronizing user profiles from sharded storage...");
        let mut profiles_synchronized = 0;
        let mut profiles_failed = 0;
        
        SHARDED_USER_PROFILES.with(|profiles| {
            let mut profiles_store = profiles.borrow_mut();
            let keys: Vec<String> = profiles_store.keys();
            ic_cdk::println!("Found {} user profile keys in sharded storage", keys.len());
            
            for principal_str in keys {
                if let Some(profile) = profiles_store.get(&principal_str) {
                    if let Ok(principal) = candid::Principal::from_text(&principal_str) {
                        store.user_profiles.insert(principal, profile.clone());
                        profiles_synchronized += 1;
                    } else {
                        profiles_failed += 1;
                        ic_cdk::println!("Failed to parse principal for profile: {}", principal_str);
                    }
                } else {
                    profiles_failed += 1;
                }
            }
        });
        
        ic_cdk::println!("User profiles synchronized: {} successful, {} failed", profiles_synchronized, profiles_failed);
        
        // Synchronize user stats
        ic_cdk::println!("Synchronizing user stats from sharded storage...");
        let mut stats_synchronized = 0;
        let mut stats_failed = 0;
        
        SHARDED_USER_STATS.with(|stats| {
            let mut stats_store = stats.borrow_mut();
            let keys: Vec<String> = stats_store.keys();
            ic_cdk::println!("Found {} user stats keys in sharded storage", keys.len());
            
            for principal_str in keys {
                if let Some(user_stats) = stats_store.get(&principal_str) {
                    if let Ok(principal) = candid::Principal::from_text(&principal_str) {
                        store.user_stats.insert(principal, user_stats.clone());
                        stats_synchronized += 1;
                    } else {
                        stats_failed += 1;
                        ic_cdk::println!("Failed to parse principal for stats: {}", principal_str);
                    }
                } else {
                    stats_failed += 1;
                }
            }
        });
        
        ic_cdk::println!("User stats synchronized: {} successful, {} failed", stats_synchronized, stats_failed);
        
        // Synchronize posts
        ic_cdk::println!("Synchronizing posts from sharded storage...");
        let mut posts_synchronized = 0;
        let mut posts_failed = 0;
        
        SHARDED_POSTS.with(|posts| {
            let mut posts_store = posts.borrow_mut();
            let keys: Vec<String> = posts_store.keys();
            ic_cdk::println!("Found {} post keys in sharded storage", keys.len());
            
            for post_id in keys {
                if let Some(post) = posts_store.get(&post_id) {
                    store.posts.insert(post_id.clone(), post.clone());
                    posts_synchronized += 1;
                } else {
                    posts_failed += 1;
                }
            }
        });
        
        ic_cdk::println!("Posts synchronized: {} successful, {} failed", posts_synchronized, posts_failed);
        
        // Synchronize comments
        ic_cdk::println!("Synchronizing comments from sharded storage...");
        let mut comments_synchronized = 0;
        let mut comments_failed = 0;
        
        SHARDED_COMMENTS.with(|comments| {
            let mut comments_store = comments.borrow_mut();
            let keys: Vec<String> = comments_store.keys();
            ic_cdk::println!("Found {} comment keys in sharded storage", keys.len());
            
            for comment_id in keys {
                if let Some(comment) = comments_store.get(&comment_id) {
                    store.comments.insert(comment_id.clone(), comment.clone());
                    comments_synchronized += 1;
                } else {
                    comments_failed += 1;
                }
            }
        });
        
        ic_cdk::println!("Comments synchronized: {} successful, {} failed", comments_synchronized, comments_failed);
        
        // Synchronize likes (this is more complex due to the structure)
        ic_cdk::println!("Synchronizing likes from sharded storage...");
        let mut likes_content_ids_synchronized = 0;
        let mut total_likes_synchronized = 0;
        
        SHARDED_LIKES.with(|likes| {
            let mut likes_store = likes.borrow_mut();
            // Use content_ids method to get all content IDs
            let content_ids = likes_store.content_ids();
            ic_cdk::println!("Found {} content IDs with likes in sharded storage", content_ids.len());
            
            for content_id in content_ids {
                let users = likes_store.get_likes(&content_id);
                if !users.is_empty() {
                    store.likes.insert(content_id.clone(), users.clone());
                    likes_content_ids_synchronized += 1;
                    total_likes_synchronized += users.len();
                }
            }
        });
        
        ic_cdk::println!("Likes synchronized: {} content IDs, {} total likes", 
            likes_content_ids_synchronized, total_likes_synchronized);
        
        // Synchronize user rewards
        ic_cdk::println!("Synchronizing user rewards from sharded storage...");
        let mut rewards_synchronized = 0;
        let mut rewards_failed = 0;
        
        SHARDED_USER_REWARDS.with(|rewards| {
            let mut rewards_store = rewards.borrow_mut();
            let keys: Vec<String> = rewards_store.keys();
            ic_cdk::println!("Found {} user rewards keys in sharded storage", keys.len());
            
            for principal_str in keys {
                if let Some(user_rewards) = rewards_store.get(&principal_str) {
                    if let Ok(principal) = candid::Principal::from_text(&principal_str) {
                        store.user_rewards.insert(principal, user_rewards.clone());
                        rewards_synchronized += 1;
                    } else {
                        rewards_failed += 1;
                        ic_cdk::println!("Failed to parse principal for rewards: {}", principal_str);
                    }
                } else {
                    rewards_failed += 1;
                }
            }
        });
        
        ic_cdk::println!("User rewards synchronized: {} successful, {} failed", rewards_synchronized, rewards_failed);
        
        // Synchronize user tasks
        ic_cdk::println!("Synchronizing user tasks from sharded storage...");
        let mut tasks_synchronized = 0;
        let mut tasks_failed = 0;
        
        SHARDED_USER_TASKS.with(|tasks| {
            let mut tasks_store = tasks.borrow_mut();
            let keys: Vec<String> = tasks_store.keys();
            ic_cdk::println!("Found {} user tasks keys in sharded storage", keys.len());
            
            for principal_str in keys {
                if let Some(user_tasks) = tasks_store.get(&principal_str) {
                    if let Ok(principal) = candid::Principal::from_text(&principal_str) {
                        store.user_tasks.insert(principal, user_tasks.clone());
                        tasks_synchronized += 1;
                    } else {
                        tasks_failed += 1;
                        ic_cdk::println!("Failed to parse principal for tasks: {}", principal_str);
                    }
                } else {
                    tasks_failed += 1;
                }
            }
        });
        
        ic_cdk::println!("User tasks synchronized: {} successful, {} failed", tasks_synchronized, tasks_failed);
    });
    
    
    ic_cdk::println!("Storage state after synchronization:");
    
    STORAGE.with(|storage| {
        let store = storage.borrow();
        ic_cdk::println!("Main storage counts after synchronization:");
        ic_cdk::println!("- Users: {}", store.users.len());
        ic_cdk::println!("- User profiles: {}", store.user_profiles.len());
        ic_cdk::println!("- User stats: {}", store.user_stats.len());
        ic_cdk::println!("- Posts: {}", store.posts.len());
        ic_cdk::println!("- Comments: {}", store.comments.len());
        ic_cdk::println!("- Likes: {}", store.likes.len());
        ic_cdk::println!("- User rewards: {}", store.user_rewards.len());
        ic_cdk::println!("- User tasks: {}", store.user_tasks.len());
    });
    
    ic_cdk::println!("========== STORAGE SYNCHRONIZED AFTER UPGRADE ==========");
}

// Save main storage as a backup
pub fn save_main_storage_backup() {
    ic_cdk::println!("Saving main storage backup...");
    
    STORAGE.with(|storage| {
        let store = storage.borrow();
        let cloned_store = (*store).clone();
        match stable_save((cloned_store,)) {
            Ok(_) => ic_cdk::println!("Saved main storage backup successfully"),
            Err(e) => {
                let error_msg = format!("Failed to save main storage backup: {:?}", e);
                ic_cdk::println!("{}", error_msg);
            }
        }
    });
}

// Restore from main storage backup
pub fn restore_from_main_storage_backup() -> bool {
    ic_cdk::println!("Restoring from main storage backup...");
    
    let result: Result<(crate::storage::Storage,), String> = stable_restore();
    
    match result {
        Ok((backup_storage,)) => {
            STORAGE.with(|storage| {
                *storage.borrow_mut() = backup_storage;
            });
            ic_cdk::println!("Restored from main storage backup successfully");
            true
        },
        Err(e) => {
            let error_msg = format!("Failed to restore from main storage backup: {:?}", e);
            ic_cdk::println!("{}", error_msg);
            false
        }
    }
}
