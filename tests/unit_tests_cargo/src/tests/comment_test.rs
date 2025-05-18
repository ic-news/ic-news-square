use crate::*;
use std::time::{SystemTime, UNIX_EPOCH};

// Mock Comment structures
#[derive(CandidType, Deserialize, Clone, Debug)]
struct Comment {
    id: String,
    post_id: String,
    author: Principal,
    content: String,
    created_at: u64,
    updated_at: Option<u64>,
    likes: u64,
    dislikes: u64,
    is_deleted: bool,
    parent_id: Option<String>,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
struct CreateCommentRequest {
    post_id: String,
    content: String,
    parent_id: Option<String>,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
struct UpdateCommentRequest {
    id: String,
    content: String,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
struct CommentResponse {
    success: bool,
    message: String,
    comment: Option<Comment>,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
struct CommentsResponse {
    success: bool,
    message: String,
    comments: Vec<Comment>,
    total: usize,
}

// Mock storage for comments
thread_local! {
    static MOCK_COMMENT_STORAGE: RefCell<MockCommentStorage> = RefCell::new(MockCommentStorage::default());
}

#[derive(Default, Clone)]
struct MockCommentStorage {
    comments: HashMap<String, Comment>,
    post_comments: HashMap<String, Vec<String>>,
    user_comments: HashMap<Principal, Vec<String>>,
    next_id: u64,
}

// Mock comment functions
fn mock_create_comment(author: Principal, request: CreateCommentRequest) -> CommentResponse {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    // Validate parent comment if provided
    if let Some(parent_id) = &request.parent_id {
        let parent_exists = MOCK_COMMENT_STORAGE.with(|storage| {
            let storage = storage.borrow();
            storage.comments.contains_key(parent_id)
        });
        
        if !parent_exists {
            return CommentResponse {
                success: false,
                message: "Parent comment not found".to_string(),
                comment: None,
            };
        }
    }
    
    let comment_id = MOCK_COMMENT_STORAGE.with(|storage| {
        let mut storage = storage.borrow_mut();
        let id = format!("comment_{}", storage.next_id);
        storage.next_id += 1;
        
        let comment = Comment {
            id: id.clone(),
            post_id: request.post_id.clone(),
            author,
            content: request.content,
            created_at: now,
            updated_at: None,
            likes: 0,
            dislikes: 0,
            is_deleted: false,
            parent_id: request.parent_id,
        };
        
        // Store the comment
        storage.comments.insert(id.clone(), comment);
        
        // Add to post's comments
        let post_comments = storage.post_comments.entry(request.post_id.clone()).or_insert_with(Vec::new);
        post_comments.push(id.clone());
        
        // Add to user's comments
        let user_comments = storage.user_comments.entry(author).or_insert_with(Vec::new);
        user_comments.push(id.clone());
        
        id
    });
    
    let comment = MOCK_COMMENT_STORAGE.with(|storage| {
        let storage = storage.borrow();
        storage.comments.get(&comment_id).cloned()
    });
    
    CommentResponse {
        success: true,
        message: "Comment created successfully".to_string(),
        comment,
    }
}

fn mock_get_comment(id: String) -> CommentResponse {
    let comment = MOCK_COMMENT_STORAGE.with(|storage| {
        let storage = storage.borrow();
        storage.comments.get(&id).cloned()
    });
    
    match comment {
        Some(comment) if !comment.is_deleted => CommentResponse {
            success: true,
            message: "Comment retrieved successfully".to_string(),
            comment: Some(comment),
        },
        Some(_) => CommentResponse {
            success: false,
            message: "Comment has been deleted".to_string(),
            comment: None,
        },
        None => CommentResponse {
            success: false,
            message: "Comment not found".to_string(),
            comment: None,
        },
    }
}

fn mock_update_comment(author: Principal, request: UpdateCommentRequest) -> CommentResponse {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    let result = MOCK_COMMENT_STORAGE.with(|storage| {
        let mut storage = storage.borrow_mut();
        
        if let Some(comment) = storage.comments.get_mut(&request.id) {
            // Check if the user is the author
            if comment.author != author {
                return CommentResponse {
                    success: false,
                    message: "Only the author can update this comment".to_string(),
                    comment: None,
                };
            }
            
            // Check if the comment is deleted
            if comment.is_deleted {
                return CommentResponse {
                    success: false,
                    message: "Cannot update a deleted comment".to_string(),
                    comment: None,
                };
            }
            
            // Update the comment
            comment.content = request.content;
            comment.updated_at = Some(now);
            
            CommentResponse {
                success: true,
                message: "Comment updated successfully".to_string(),
                comment: Some(comment.clone()),
            }
        } else {
            CommentResponse {
                success: false,
                message: "Comment not found".to_string(),
                comment: None,
            }
        }
    });
    
    result
}

fn mock_delete_comment(author: Principal, id: String) -> CommentResponse {
    let result = MOCK_COMMENT_STORAGE.with(|storage| {
        let mut storage = storage.borrow_mut();
        
        if let Some(comment) = storage.comments.get_mut(&id) {
            // Check if the user is the author
            if comment.author != author {
                return CommentResponse {
                    success: false,
                    message: "Only the author can delete this comment".to_string(),
                    comment: None,
                };
            }
            
            // Check if the comment is already deleted
            if comment.is_deleted {
                return CommentResponse {
                    success: false,
                    message: "Comment is already deleted".to_string(),
                    comment: None,
                };
            }
            
            // Mark the comment as deleted
            comment.is_deleted = true;
            
            CommentResponse {
                success: true,
                message: "Comment deleted successfully".to_string(),
                comment: Some(comment.clone()),
            }
        } else {
            CommentResponse {
                success: false,
                message: "Comment not found".to_string(),
                comment: None,
            }
        }
    });
    
    result
}

fn mock_get_post_comments(post_id: String) -> CommentsResponse {
    let comments = MOCK_COMMENT_STORAGE.with(|storage| {
        let storage = storage.borrow();
        
        let comment_ids = storage.post_comments.get(&post_id).cloned().unwrap_or_default();
        let mut post_comments = Vec::new();
        
        for id in comment_ids {
            if let Some(comment) = storage.comments.get(&id) {
                if !comment.is_deleted {
                    post_comments.push(comment.clone());
                }
            }
        }
        
        post_comments
    });
    
    let total = comments.len();
    CommentsResponse {
        success: true,
        message: "Post comments retrieved successfully".to_string(),
        comments,
        total,
    }
}

fn mock_like_comment(_user: Principal, id: String) -> CommentResponse {
    let result = MOCK_COMMENT_STORAGE.with(|storage| {
        let mut storage = storage.borrow_mut();
        
        if let Some(comment) = storage.comments.get_mut(&id) {
            // Check if the comment is deleted
            if comment.is_deleted {
                return CommentResponse {
                    success: false,
                    message: "Cannot like a deleted comment".to_string(),
                    comment: None,
                };
            }
            
            // Increment likes
            comment.likes += 1;
            
            CommentResponse {
                success: true,
                message: "Comment liked successfully".to_string(),
                comment: Some(comment.clone()),
            }
        } else {
            CommentResponse {
                success: false,
                message: "Comment not found".to_string(),
                comment: None,
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
        MOCK_COMMENT_STORAGE.with(|storage| {
            *storage.borrow_mut() = MockCommentStorage::default();
        });
    }
    
    #[test]
    fn test_create_comment() {
        reset_mock_storage();
        
        let user = test_principal();
        let request = CreateCommentRequest {
            post_id: "post_1".to_string(),
            content: "This is a test comment".to_string(),
            parent_id: None,
        };
        
        let response = mock_create_comment(user, request);
        
        assert!(response.success);
        assert!(response.comment.is_some());
        let comment = response.comment.unwrap();
        assert_eq!(comment.post_id, "post_1");
        assert_eq!(comment.content, "This is a test comment");
        assert_eq!(comment.author, user);
        assert_eq!(comment.likes, 0);
        assert_eq!(comment.dislikes, 0);
        assert!(!comment.is_deleted);
        assert!(comment.parent_id.is_none());
    }
    
    #[test]
    fn test_create_reply() {
        reset_mock_storage();
        
        let user = test_principal();
        
        // Create parent comment
        let parent_request = CreateCommentRequest {
            post_id: "post_1".to_string(),
            content: "Parent comment".to_string(),
            parent_id: None,
        };
        
        let parent_response = mock_create_comment(user, parent_request);
        let parent_id = parent_response.comment.unwrap().id;
        
        // Create reply
        let reply_request = CreateCommentRequest {
            post_id: "post_1".to_string(),
            content: "Reply to parent".to_string(),
            parent_id: Some(parent_id.clone()),
        };
        
        let reply_response = mock_create_comment(user, reply_request);
        
        assert!(reply_response.success);
        assert!(reply_response.comment.is_some());
        let reply = reply_response.comment.unwrap();
        assert_eq!(reply.post_id, "post_1");
        assert_eq!(reply.content, "Reply to parent");
        assert_eq!(reply.parent_id.unwrap(), parent_id);
    }
    
    #[test]
    fn test_get_comment() {
        reset_mock_storage();
        
        let user = test_principal();
        let request = CreateCommentRequest {
            post_id: "post_1".to_string(),
            content: "This is a test comment".to_string(),
            parent_id: None,
        };
        
        let create_response = mock_create_comment(user, request);
        let comment_id = create_response.comment.unwrap().id;
        
        let get_response = mock_get_comment(comment_id);
        
        assert!(get_response.success);
        assert!(get_response.comment.is_some());
        let comment = get_response.comment.unwrap();
        assert_eq!(comment.content, "This is a test comment");
    }
    
    #[test]
    fn test_update_comment() {
        reset_mock_storage();
        
        let user = test_principal();
        let request = CreateCommentRequest {
            post_id: "post_1".to_string(),
            content: "Original content".to_string(),
            parent_id: None,
        };
        
        let create_response = mock_create_comment(user, request);
        let comment_id = create_response.comment.unwrap().id;
        
        let update_request = UpdateCommentRequest {
            id: comment_id.clone(),
            content: "Updated content".to_string(),
        };
        
        let update_response = mock_update_comment(user, update_request);
        
        assert!(update_response.success);
        assert!(update_response.comment.is_some());
        let updated_comment = update_response.comment.unwrap();
        assert_eq!(updated_comment.content, "Updated content");
        assert!(updated_comment.updated_at.is_some());
    }
    
    #[test]
    fn test_delete_comment() {
        reset_mock_storage();
        
        let user = test_principal();
        let request = CreateCommentRequest {
            post_id: "post_1".to_string(),
            content: "This is a test comment".to_string(),
            parent_id: None,
        };
        
        let create_response = mock_create_comment(user, request);
        let comment_id = create_response.comment.unwrap().id;
        
        let delete_response = mock_delete_comment(user, comment_id.clone());
        
        assert!(delete_response.success);
        assert!(delete_response.comment.is_some());
        let deleted_comment = delete_response.comment.unwrap();
        assert!(deleted_comment.is_deleted);
        
        // Try to get the deleted comment
        let get_response = mock_get_comment(comment_id);
        assert!(!get_response.success);
        assert!(get_response.comment.is_none());
    }
    
    #[test]
    fn test_get_post_comments() {
        reset_mock_storage();
        
        let user = test_principal();
        let post_id = "post_1".to_string();
        
        // Create multiple comments
        for i in 1..=3 {
            let request = CreateCommentRequest {
                post_id: post_id.clone(),
                content: format!("Comment {}", i),
                parent_id: None,
            };
            
            mock_create_comment(user, request);
        }
        
        let comments_response = mock_get_post_comments(post_id);
        
        assert!(comments_response.success);
        assert_eq!(comments_response.comments.len(), 3);
        assert_eq!(comments_response.total, 3);
    }
    
    #[test]
    fn test_like_comment() {
        reset_mock_storage();
        
        let user = test_principal();
        let request = CreateCommentRequest {
            post_id: "post_1".to_string(),
            content: "This is a test comment".to_string(),
            parent_id: None,
        };
        
        let create_response = mock_create_comment(user, request);
        let comment_id = create_response.comment.unwrap().id;
        
        let like_response = mock_like_comment(user, comment_id.clone());
        
        assert!(like_response.success);
        assert!(like_response.comment.is_some());
        let liked_comment = like_response.comment.unwrap();
        assert_eq!(liked_comment.likes, 1);
        
        // Like again
        let like_again_response = mock_like_comment(user, comment_id);
        let liked_again_comment = like_again_response.comment.unwrap();
        assert_eq!(liked_again_comment.likes, 2);
    }
    
    #[test]
    fn test_unauthorized_update() {
        reset_mock_storage();
        
        let user1 = test_principal();
        let user2 = Principal::from_text("rrkah-fqaaa-aaaaa-aaaaq-cai").unwrap();
        
        let request = CreateCommentRequest {
            post_id: "post_1".to_string(),
            content: "This is a test comment".to_string(),
            parent_id: None,
        };
        
        let create_response = mock_create_comment(user1, request);
        let comment_id = create_response.comment.unwrap().id;
        
        let update_request = UpdateCommentRequest {
            id: comment_id,
            content: "Unauthorized update".to_string(),
        };
        
        // Try to update as a different user
        let update_response = mock_update_comment(user2, update_request);
        
        assert!(!update_response.success);
        assert!(update_response.comment.is_none());
        assert!(update_response.message.contains("Only the author"));
    }
    
    #[test]
    fn test_unauthorized_delete() {
        reset_mock_storage();
        
        let user1 = test_principal();
        let user2 = Principal::from_text("rrkah-fqaaa-aaaaa-aaaaq-cai").unwrap();
        
        let request = CreateCommentRequest {
            post_id: "post_1".to_string(),
            content: "This is a test comment".to_string(),
            parent_id: None,
        };
        
        let create_response = mock_create_comment(user1, request);
        let comment_id = create_response.comment.unwrap().id;
        
        // Try to delete as a different user
        let delete_response = mock_delete_comment(user2, comment_id);
        
        assert!(!delete_response.success);
        assert!(delete_response.comment.is_none());
        assert!(delete_response.message.contains("Only the author"));
    }
    
    #[test]
    fn test_invalid_parent_id() {
        reset_mock_storage();
        
        let user = test_principal();
        let request = CreateCommentRequest {
            post_id: "post_1".to_string(),
            content: "This is a test comment".to_string(),
            parent_id: Some("non_existent_comment".to_string()),
        };
        
        let response = mock_create_comment(user, request);
        
        assert!(!response.success);
        assert!(response.comment.is_none());
        assert!(response.message.contains("Parent comment not found"));
    }
}
