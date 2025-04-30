use crate::storage::Post;
use crate::storage::sharded::SHARDED_POSTS;

// Insert a post into sharded storage
pub fn insert_post_sharded(id: String, post: Post) {
    SHARDED_POSTS.with(|posts| {
        let mut posts_store = posts.borrow_mut();
        posts_store.insert(id, post);
    });
}

// Get a post from sharded storage
pub fn get_post_sharded(id: &str) -> Option<Post> {
    SHARDED_POSTS.with(|posts| {
        let mut posts_store = posts.borrow_mut();
        posts_store.get(id)
    })
}

// Remove a post from sharded storage
pub fn remove_post_sharded(id: &str) -> Option<Post> {
    SHARDED_POSTS.with(|posts| {
        let mut posts_store = posts.borrow_mut();
        posts_store.remove(id)
    })
}

// Check if a post exists in sharded storage
pub fn post_exists_sharded(id: &str) -> bool {
    SHARDED_POSTS.with(|posts| {
        let posts_store = posts.borrow();
        posts_store.contains_key(id)
    })
}

// Get all post IDs from sharded storage
pub fn get_all_post_ids_sharded() -> Vec<String> {
    SHARDED_POSTS.with(|posts| {
        let posts_store = posts.borrow();
        posts_store.keys()
    })
}
