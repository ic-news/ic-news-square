use std::collections::{HashMap, HashSet};
use std::cell::RefCell;
use candid::{CandidType, Deserialize, Principal};
use ic_cdk::api::time;

// Shard configuration
const DEFAULT_SHARD_SIZE: usize = 100; // Default number of items per shard
const MAX_SHARDS_IN_MEMORY: usize = 5; // Maximum number of shards to keep in memory

// Generic sharded storage for any type that can be serialized
#[derive(CandidType, Deserialize, Clone)]
pub struct ShardedStorage<T: CandidType + Clone> {
    pub shard_map: HashMap<u64, Vec<(String, T)>>, // shard_id -> [(key, value)]
    pub key_to_shard: HashMap<String, u64>,        // key -> shard_id
    pub shard_size: usize,                         // Maximum items per shard
    pub last_accessed: HashMap<u64, u64>,          // shard_id -> last access timestamp
}

impl<T: CandidType + Clone> Default for ShardedStorage<T> {
    fn default() -> Self {
        ShardedStorage {
            shard_map: HashMap::new(),
            key_to_shard: HashMap::new(),
            shard_size: DEFAULT_SHARD_SIZE,
            last_accessed: HashMap::new(),
        }
    }
}

impl<T: CandidType + Clone> ShardedStorage<T> {
    // Create a new sharded storage with custom shard size
    pub fn new(shard_size: usize) -> Self {
        ShardedStorage {
            shard_map: HashMap::new(),
            key_to_shard: HashMap::new(),
            shard_size,
            last_accessed: HashMap::new(),
        }
    }

    // Insert an item into the sharded storage
    pub fn insert(&mut self, key: String, value: T) {
        // Check if key already exists
        if let Some(shard_id) = self.key_to_shard.get(&key) {
            // Key exists, update the value in the shard
            if let Some(shard) = self.shard_map.get_mut(shard_id) {
                // Update the value
                for item in shard.iter_mut() {
                    if item.0 == key {
                        item.1 = value;
                        break;
                    }
                }
            }
            // Update last accessed time
            self.last_accessed.insert(*shard_id, time() / 1_000_000);
            return;
        }

        // Key doesn't exist, find a shard with space or create a new one
        let mut target_shard_id = None;
        
        // Find a shard with space
        for (shard_id, shard) in &self.shard_map {
            if shard.len() < self.shard_size {
                target_shard_id = Some(*shard_id);
                break;
            }
        }

        // If no shard with space, create a new one
        let shard_id = target_shard_id.unwrap_or_else(|| {
            let new_id = self.shard_map.len() as u64;
            self.shard_map.insert(new_id, Vec::new());
            new_id
        });

        // Insert the item into the shard
        if let Some(shard) = self.shard_map.get_mut(&shard_id) {
            shard.push((key.clone(), value));
        }

        // Update key to shard mapping
        self.key_to_shard.insert(key, shard_id);
        
        // Update last accessed time
        self.last_accessed.insert(shard_id, time() / 1_000_000);
        
        // Clean up least recently used shards if we have too many in memory
        self.cleanup_lru_shards();
    }

    // Get an item from the sharded storage
    pub fn get(&mut self, key: &str) -> Option<T> {
        // Find which shard contains the key
        let shard_id = self.key_to_shard.get(key)?;
        
        // Get the shard
        let shard = self.shard_map.get(shard_id)?;
        
        // Find the item in the shard
        for (k, v) in shard {
            if k == key {
                // Update last accessed time
                self.last_accessed.insert(*shard_id, time() / 1_000_000);
                return Some(v.clone());
            }
        }
        
        None
    }

    // Remove an item from the sharded storage
    pub fn remove(&mut self, key: &str) -> Option<T> {
        // Find which shard contains the key
        let shard_id_opt = self.key_to_shard.get(key).cloned();
        let shard_id = shard_id_opt?;
        
        // Get the shard
        let shard = self.shard_map.get_mut(&shard_id)?;
        
        // Find the item index in the shard
        let mut item_index = None;
        let mut removed_value = None;
        
        for (i, (k, v)) in shard.iter().enumerate() {
            if k == key {
                item_index = Some(i);
                removed_value = Some(v.clone());
                break;
            }
        }
        
        // Remove the item if found
        if let Some(index) = item_index {
            shard.remove(index);
            self.key_to_shard.remove(key);
            
            // Update last accessed time
            self.last_accessed.insert(shard_id, time() / 1_000_000);
        }
        
        removed_value
    }

    // Check if the storage contains a key
    pub fn contains_key(&self, key: &str) -> bool {
        self.key_to_shard.contains_key(key)
    }

    // Get all keys in the storage
    pub fn keys(&self) -> Vec<String> {
        self.key_to_shard.keys().cloned().collect()
    }

    // Get all items in the storage (expensive operation)
    pub fn items(&mut self) -> Vec<(String, T)> {
        let mut result = Vec::new();
        
        // Update access time for all shards
        let current_time = time() / 1_000_000;
        
        for (shard_id, shard) in &self.shard_map {
            for (key, value) in shard {
                result.push((key.clone(), value.clone()));
            }
            self.last_accessed.insert(*shard_id, current_time);
        }
        
        result
    }

    // Get a specific shard (for batch processing)
    pub fn get_shard(&mut self, shard_id: u64) -> Option<Vec<(String, T)>> {
        let shard = self.shard_map.get(&shard_id)?;
        
        // Update last accessed time
        self.last_accessed.insert(shard_id, time() / 1_000_000);
        
        Some(shard.clone())
    }

    // Get all shard IDs
    pub fn shard_ids(&self) -> Vec<u64> {
        self.shard_map.keys().cloned().collect()
    }

    // Clean up least recently used shards to save memory
    fn cleanup_lru_shards(&mut self) {
        if self.shard_map.len() <= MAX_SHARDS_IN_MEMORY {
            return;
        }
        
        // Find the least recently used shards
        let mut shard_access_times: Vec<(u64, u64)> = self.last_accessed
            .iter()
            .map(|(shard_id, timestamp)| (*shard_id, *timestamp))
            .collect();
        
        // Sort by access time (oldest first)
        shard_access_times.sort_by(|a, b| a.1.cmp(&b.1));
        
        // Remove oldest shards until we're under the limit
        let shards_to_remove = self.shard_map.len() - MAX_SHARDS_IN_MEMORY;
        for i in 0..shards_to_remove {
            if i < shard_access_times.len() {
                let shard_id = shard_access_times[i].0;
                self.shard_map.remove(&shard_id);
                // Note: We don't remove from key_to_shard to maintain the mapping
                // The shard will be loaded again when needed
            }
        }
    }
}

// Specialized sharded storage for likes (Principal sets)
#[derive(CandidType, Deserialize, Clone)]
pub struct ShardedLikes {
    pub shard_map: HashMap<u64, HashMap<String, HashSet<Principal>>>, // shard_id -> content_id -> principals
    pub content_to_shard: HashMap<String, u64>,                      // content_id -> shard_id
    pub shard_size: usize,                                           // Maximum items per shard
    pub last_accessed: HashMap<u64, u64>,                            // shard_id -> last access timestamp
}

impl Default for ShardedLikes {
    fn default() -> Self {
        ShardedLikes {
            shard_map: HashMap::new(),
            content_to_shard: HashMap::new(),
            shard_size: DEFAULT_SHARD_SIZE,
            last_accessed: HashMap::new(),
        }
    }
}

impl ShardedLikes {
    // Get all content IDs with likes
    pub fn content_ids(&self) -> Vec<String> {
        self.content_to_shard.keys().cloned().collect()
    }
    
    // Add a like
    pub fn add_like(&mut self, content_id: &str, user: Principal) -> bool {
        // Find or create the shard for this content
        let shard_id = self.get_or_create_shard(content_id);
        
        // Get the shard
        if let Some(shard) = self.shard_map.get_mut(&shard_id) {
            // Get or create the likes set for this content
            let likes = shard.entry(content_id.to_string()).or_insert_with(HashSet::new);
            
            // Add the like and return whether it was newly added
            let result = likes.insert(user);
            
            // Update last accessed time
            self.last_accessed.insert(shard_id, time() / 1_000_000);
            
            return result;
        }
        
        false
    }

    // Remove a like
    pub fn remove_like(&mut self, content_id: &str, user: Principal) -> bool {
        // Find the shard for this content
        if let Some(shard_id) = self.content_to_shard.get(content_id) {
            // Get the shard
            if let Some(shard) = self.shard_map.get_mut(shard_id) {
                // Get the likes set for this content
                if let Some(likes) = shard.get_mut(content_id) {
                    // Remove the like and return whether it was removed
                    let result = likes.remove(&user);
                    
                    // Update last accessed time
                    self.last_accessed.insert(*shard_id, time() / 1_000_000);
                    
                    return result;
                }
            }
        }
        
        false
    }

    // Check if a user has liked content
    pub fn has_liked(&mut self, content_id: &str, user: Principal) -> bool {
        // Find the shard for this content
        if let Some(shard_id) = self.content_to_shard.get(content_id) {
            // Get the shard
            if let Some(shard) = self.shard_map.get(shard_id) {
                // Get the likes set for this content
                if let Some(likes) = shard.get(content_id) {
                    // Update last accessed time
                    self.last_accessed.insert(*shard_id, time() / 1_000_000);
                    
                    // Check if the user has liked
                    return likes.contains(&user);
                }
            }
        }
        
        false
    }

    // Get all users who liked content
    pub fn get_likes(&mut self, content_id: &str) -> HashSet<Principal> {
        // Find the shard for this content
        if let Some(shard_id) = self.content_to_shard.get(content_id) {
            // Get the shard
            if let Some(shard) = self.shard_map.get(shard_id) {
                // Get the likes set for this content
                if let Some(likes) = shard.get(content_id) {
                    // Update last accessed time
                    self.last_accessed.insert(*shard_id, time() / 1_000_000);
                    
                    // Return a clone of the likes set
                    return likes.clone();
                }
            }
        }
        
        HashSet::new()
    }

    // Get the number of likes for content
    pub fn count_likes(&mut self, content_id: &str) -> usize {
        self.get_likes(content_id).len()
    }

    // Find or create a shard for content
    fn get_or_create_shard(&mut self, content_id: &str) -> u64 {
        // Check if content already has a shard
        if let Some(shard_id) = self.content_to_shard.get(content_id) {
            return *shard_id;
        }

        // Find a shard with space
        let mut target_shard_id = None;
        
        for (shard_id, shard) in &self.shard_map {
            if shard.len() < self.shard_size {
                target_shard_id = Some(*shard_id);
                break;
            }
        }

        // If no shard with space, create a new one
        let shard_id = target_shard_id.unwrap_or_else(|| {
            let new_id = self.shard_map.len() as u64;
            self.shard_map.insert(new_id, HashMap::new());
            new_id
        });

        // Update content to shard mapping
        self.content_to_shard.insert(content_id.to_string(), shard_id);
        
        // Update last accessed time
        self.last_accessed.insert(shard_id, time() / 1_000_000);
        
        // Clean up least recently used shards if we have too many in memory
        self.cleanup_lru_shards();
        
        shard_id
    }

    // Clean up least recently used shards to save memory
    fn cleanup_lru_shards(&mut self) {
        if self.shard_map.len() <= MAX_SHARDS_IN_MEMORY {
            return;
        }
        
        // Find the least recently used shards
        let mut shard_access_times: Vec<(u64, u64)> = self.last_accessed
            .iter()
            .map(|(shard_id, timestamp)| (*shard_id, *timestamp))
            .collect();
        
        // Sort by access time (oldest first)
        shard_access_times.sort_by(|a, b| a.1.cmp(&b.1));
        
        // Remove oldest shards until we're under the limit
        let shards_to_remove = self.shard_map.len() - MAX_SHARDS_IN_MEMORY;
        for i in 0..shards_to_remove {
            if i < shard_access_times.len() {
                let shard_id = shard_access_times[i].0;
                self.shard_map.remove(&shard_id);
                // Note: We don't remove from content_to_shard to maintain the mapping
                // The shard will be loaded again when needed
            }
        }
    }
}

// Thread-local storage for sharded data
thread_local! {
    // Content storage
    pub static SHARDED_POSTS: RefCell<ShardedStorage<crate::storage::Post>> = 
        RefCell::new(ShardedStorage::default());
    
    pub static SHARDED_ARTICLES: RefCell<ShardedStorage<crate::storage::Article>> = 
        RefCell::new(ShardedStorage::default());
    
    pub static SHARDED_COMMENTS: RefCell<ShardedStorage<crate::storage::Comment>> = 
        RefCell::new(ShardedStorage::default());
    
    // User data storage
    pub static SHARDED_USERS: RefCell<ShardedStorage<crate::storage::User>> = 
        RefCell::new(ShardedStorage::default());
        
    pub static SHARDED_USER_PROFILES: RefCell<ShardedStorage<crate::storage::UserProfile>> = 
        RefCell::new(ShardedStorage::default());
        
    pub static SHARDED_USER_STATS: RefCell<ShardedStorage<crate::storage::UserStats>> = 
        RefCell::new(ShardedStorage::default());
    
    // Reward storage
    pub static SHARDED_USER_REWARDS: RefCell<ShardedStorage<crate::storage::UserRewards>> = 
        RefCell::new(ShardedStorage::default());
        
    pub static SHARDED_USER_TASKS: RefCell<ShardedStorage<crate::storage::UserTasks>> = 
        RefCell::new(ShardedStorage::default());
    
    // Interaction storage
    pub static SHARDED_LIKES: RefCell<ShardedLikes> = 
        RefCell::new(ShardedLikes::default());
}
