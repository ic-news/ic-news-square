use crate::*;
use std::time::{SystemTime, UNIX_EPOCH};

// Mock Post structures
#[derive(CandidType, Deserialize, Clone, Debug)]
struct Post {
    id: String,
    author: Principal,
    title: String,
    content: String,
    created_at: u64,
    updated_at: Option<u64>,
    likes: u64,
    dislikes: u64,
    tags: Vec<String>,
    is_deleted: bool,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
struct CreatePostRequest {
    title: String,
    content: String,
    tags: Vec<String>,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
struct UpdatePostRequest {
    id: String,
    title: Option<String>,
    content: Option<String>,
    tags: Option<Vec<String>>,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
struct PostResponse {
    success: bool,
    message: String,
    post: Option<Post>,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
struct PostsResponse {
    success: bool,
    message: String,
    posts: Vec<Post>,
    total: usize,
}

// Mock storage for posts
thread_local! {
    static MOCK_POST_STORAGE: RefCell<MockPostStorage> = RefCell::new(MockPostStorage::default());
}

#[derive(Default, Clone)]
struct MockPostStorage {
    posts: HashMap<String, Post>,
    user_posts: HashMap<Principal, Vec<String>>,
    next_id: u64,
}

// Mock post functions
fn mock_create_post(author: Principal, request: CreatePostRequest) -> PostResponse {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    let post_id = MOCK_POST_STORAGE.with(|storage| {
        let mut storage = storage.borrow_mut();
        let id = format!("post_{}", storage.next_id);
        storage.next_id += 1;
        
        let post = Post {
            id: id.clone(),
            author,
            title: request.title,
            content: request.content,
            created_at: now,
            updated_at: None,
            likes: 0,
            dislikes: 0,
            tags: request.tags,
            is_deleted: false,
        };
        
        // Store the post
        storage.posts.insert(id.clone(), post);
        
        // Add to user's posts
        let user_posts = storage.user_posts.entry(author).or_insert_with(Vec::new);
        user_posts.push(id.clone());
        
        id
    });
    
    let post = MOCK_POST_STORAGE.with(|storage| {
        let storage = storage.borrow();
        storage.posts.get(&post_id).cloned()
    });
    
    PostResponse {
        success: true,
        message: "Post created successfully".to_string(),
        post,
    }
}

fn mock_get_post(id: String) -> PostResponse {
    let post = MOCK_POST_STORAGE.with(|storage| {
        let storage = storage.borrow();
        storage.posts.get(&id).cloned()
    });
    
    match post {
        Some(post) if !post.is_deleted => PostResponse {
            success: true,
            message: "Post retrieved successfully".to_string(),
            post: Some(post),
        },
        Some(_) => PostResponse {
            success: false,
            message: "Post has been deleted".to_string(),
            post: None,
        },
        None => PostResponse {
            success: false,
            message: "Post not found".to_string(),
            post: None,
        },
    }
}

fn mock_update_post(author: Principal, request: UpdatePostRequest) -> PostResponse {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    let result = MOCK_POST_STORAGE.with(|storage| {
        let mut storage = storage.borrow_mut();
        
        if let Some(post) = storage.posts.get_mut(&request.id) {
            // Check if the user is the author
            if post.author != author {
                return PostResponse {
                    success: false,
                    message: "Only the author can update this post".to_string(),
                    post: None,
                };
            }
            
            // Check if the post is deleted
            if post.is_deleted {
                return PostResponse {
                    success: false,
                    message: "Cannot update a deleted post".to_string(),
                    post: None,
                };
            }
            
            // Update the post
            if let Some(title) = request.title {
                post.title = title;
            }
            
            if let Some(content) = request.content {
                post.content = content;
            }
            
            if let Some(tags) = request.tags {
                post.tags = tags;
            }
            
            post.updated_at = Some(now);
            
            PostResponse {
                success: true,
                message: "Post updated successfully".to_string(),
                post: Some(post.clone()),
            }
        } else {
            PostResponse {
                success: false,
                message: "Post not found".to_string(),
                post: None,
            }
        }
    });
    
    result
}

fn mock_delete_post(author: Principal, id: String) -> PostResponse {
    let result = MOCK_POST_STORAGE.with(|storage| {
        let mut storage = storage.borrow_mut();
        
        if let Some(post) = storage.posts.get_mut(&id) {
            // Check if the user is the author
            if post.author != author {
                return PostResponse {
                    success: false,
                    message: "Only the author can delete this post".to_string(),
                    post: None,
                };
            }
            
            // Check if the post is already deleted
            if post.is_deleted {
                return PostResponse {
                    success: false,
                    message: "Post is already deleted".to_string(),
                    post: None,
                };
            }
            
            // Mark the post as deleted
            post.is_deleted = true;
            
            PostResponse {
                success: true,
                message: "Post deleted successfully".to_string(),
                post: Some(post.clone()),
            }
        } else {
            PostResponse {
                success: false,
                message: "Post not found".to_string(),
                post: None,
            }
        }
    });
    
    result
}

fn mock_get_user_posts(user: Principal) -> PostsResponse {
    let posts = MOCK_POST_STORAGE.with(|storage| {
        let storage = storage.borrow();
        
        let post_ids = storage.user_posts.get(&user).cloned().unwrap_or_default();
        let mut user_posts = Vec::new();
        
        for id in post_ids {
            if let Some(post) = storage.posts.get(&id) {
                if !post.is_deleted {
                    user_posts.push(post.clone());
                }
            }
        }
        
        user_posts
    });
    
    let total = posts.len();
    PostsResponse {
        success: true,
        message: "User posts retrieved successfully".to_string(),
        posts,
        total,
    }
}

fn mock_like_post(_user: Principal, id: String) -> PostResponse {
    let result = MOCK_POST_STORAGE.with(|storage| {
        let mut storage = storage.borrow_mut();
        
        if let Some(post) = storage.posts.get_mut(&id) {
            // Check if the post is deleted
            if post.is_deleted {
                return PostResponse {
                    success: false,
                    message: "Cannot like a deleted post".to_string(),
                    post: None,
                };
            }
            
            // Increment likes
            post.likes += 1;
            
            PostResponse {
                success: true,
                message: "Post liked successfully".to_string(),
                post: Some(post.clone()),
            }
        } else {
            PostResponse {
                success: false,
                message: "Post not found".to_string(),
                post: None,
            }
        }
    });
    
    result
}

// Unit tests
#[cfg(test)]
mod tests {
    use super::*;
    
    // Helper function to create a test principal
    fn test_principal() -> Principal {
        Principal::from_text("2vxsx-fae").unwrap()
    }
    
    // Helper function to reset the mock storage
    fn reset_mock_storage() {
        MOCK_POST_STORAGE.with(|storage| {
            *storage.borrow_mut() = MockPostStorage::default();
        });
    }
    
    #[test]
    fn test_create_post() {
        reset_mock_storage();
        
        let user = test_principal();
        let request = CreatePostRequest {
            title: "Test Post".to_string(),
            content: "This is a test post content".to_string(),
            tags: vec!["test".to_string(), "example".to_string()],
        };
        
        let response = mock_create_post(user, request);
        
        assert!(response.success);
        assert!(response.post.is_some());
        let post = response.post.unwrap();
        assert_eq!(post.title, "Test Post");
        assert_eq!(post.content, "This is a test post content");
        assert_eq!(post.author, user);
        assert_eq!(post.likes, 0);
        assert_eq!(post.dislikes, 0);
        assert_eq!(post.tags.len(), 2);
        assert!(!post.is_deleted);
    }
    
    #[test]
    fn test_get_post() {
        reset_mock_storage();
        
        let user = test_principal();
        let request = CreatePostRequest {
            title: "Test Post".to_string(),
            content: "This is a test post content".to_string(),
            tags: vec!["test".to_string()],
        };
        
        let create_response = mock_create_post(user, request);
        let post_id = create_response.post.unwrap().id;
        
        let get_response = mock_get_post(post_id);
        
        assert!(get_response.success);
        assert!(get_response.post.is_some());
        let post = get_response.post.unwrap();
        assert_eq!(post.title, "Test Post");
    }
    
    #[test]
    fn test_update_post() {
        reset_mock_storage();
        
        let user = test_principal();
        let request = CreatePostRequest {
            title: "Original Title".to_string(),
            content: "Original content".to_string(),
            tags: vec!["original".to_string()],
        };
        
        let create_response = mock_create_post(user, request);
        let post_id = create_response.post.unwrap().id;
        
        let update_request = UpdatePostRequest {
            id: post_id.clone(),
            title: Some("Updated Title".to_string()),
            content: Some("Updated content".to_string()),
            tags: Some(vec!["updated".to_string()]),
        };
        
        let update_response = mock_update_post(user, update_request);
        
        assert!(update_response.success);
        assert!(update_response.post.is_some());
        let updated_post = update_response.post.unwrap();
        assert_eq!(updated_post.title, "Updated Title");
        assert_eq!(updated_post.content, "Updated content");
        assert_eq!(updated_post.tags, vec!["updated".to_string()]);
        assert!(updated_post.updated_at.is_some());
    }
    
    #[test]
    fn test_delete_post() {
        reset_mock_storage();
        
        let user = test_principal();
        let request = CreatePostRequest {
            title: "Test Post".to_string(),
            content: "This is a test post content".to_string(),
            tags: vec!["test".to_string()],
        };
        
        let create_response = mock_create_post(user, request);
        let post_id = create_response.post.unwrap().id;
        
        let delete_response = mock_delete_post(user, post_id.clone());
        
        assert!(delete_response.success);
        assert!(delete_response.post.is_some());
        let deleted_post = delete_response.post.unwrap();
        assert!(deleted_post.is_deleted);
        
        // Try to get the deleted post
        let get_response = mock_get_post(post_id);
        assert!(!get_response.success);
        assert!(get_response.post.is_none());
    }
    
    #[test]
    fn test_get_user_posts() {
        reset_mock_storage();
        
        let user = test_principal();
        
        // Create multiple posts
        for i in 1..=3 {
            let request = CreatePostRequest {
                title: format!("Post {}", i),
                content: format!("Content {}", i),
                tags: vec![format!("tag{}", i)],
            };
            
            mock_create_post(user, request);
        }
        
        let posts_response = mock_get_user_posts(user);
        
        assert!(posts_response.success);
        assert_eq!(posts_response.posts.len(), 3);
        assert_eq!(posts_response.total, 3);
    }
    
    #[test]
    fn test_like_post() {
        reset_mock_storage();
        
        let user = test_principal();
        let request = CreatePostRequest {
            title: "Test Post".to_string(),
            content: "This is a test post content".to_string(),
            tags: vec!["test".to_string()],
        };
        
        let create_response = mock_create_post(user, request);
        let post_id = create_response.post.unwrap().id;
        
        let like_response = mock_like_post(user, post_id.clone());
        
        assert!(like_response.success);
        assert!(like_response.post.is_some());
        let liked_post = like_response.post.unwrap();
        assert_eq!(liked_post.likes, 1);
        
        // Like again
        let like_again_response = mock_like_post(user, post_id);
        let liked_again_post = like_again_response.post.unwrap();
        assert_eq!(liked_again_post.likes, 2);
    }
    
    #[test]
    fn test_unauthorized_update() {
        reset_mock_storage();
        
        let user1 = test_principal();
        let user2 = Principal::from_text("rrkah-fqaaa-aaaaa-aaaaq-cai").unwrap();
        
        let request = CreatePostRequest {
            title: "Test Post".to_string(),
            content: "This is a test post content".to_string(),
            tags: vec!["test".to_string()],
        };
        
        let create_response = mock_create_post(user1, request);
        let post_id = create_response.post.unwrap().id;
        
        let update_request = UpdatePostRequest {
            id: post_id,
            title: Some("Unauthorized Update".to_string()),
            content: None,
            tags: None,
        };
        
        // Try to update as a different user
        let update_response = mock_update_post(user2, update_request);
        
        assert!(!update_response.success);
        assert!(update_response.post.is_none());
        assert!(update_response.message.contains("Only the author"));
    }
    
    #[test]
    fn test_unauthorized_delete() {
        reset_mock_storage();
        
        let user1 = test_principal();
        let user2 = Principal::from_text("rrkah-fqaaa-aaaaa-aaaaq-cai").unwrap();
        
        let request = CreatePostRequest {
            title: "Test Post".to_string(),
            content: "This is a test post content".to_string(),
            tags: vec!["test".to_string()],
        };
        
        let create_response = mock_create_post(user1, request);
        let post_id = create_response.post.unwrap().id;
        
        // Try to delete as a different user
        let delete_response = mock_delete_post(user2, post_id);
        
        assert!(!delete_response.success);
        assert!(delete_response.post.is_none());
        assert!(delete_response.message.contains("Only the author"));
    }
}
