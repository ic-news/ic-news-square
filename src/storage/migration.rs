use std::cell::RefCell;
use crate::storage::STORAGE;
use crate::storage::sharded::*;
use crate::storage::sharded::ShardedStorage;
use ic_cdk::api::time;

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
