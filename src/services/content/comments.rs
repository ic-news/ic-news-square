use candid::Principal;
use ic_cdk::api::time;
use std::collections::{HashMap, HashSet};

use crate::auth::is_admin;
use crate::models::content::{
    CreateCommentRequest, UpdateCommentRequest, ContentStatus, ParentType,
    ContentVisibility, ContentType, CommentResponse, CommentsResponse,
    PaginationParams, MAX_COMMENT_LENGTH,
};
use crate::models::storage::Storage;
use crate::utils::content_utils::calculate_content_length_excluding_base64_and_html;
use crate::{SquareError, SquareResult};
use crate::storage::{Comment, STORAGE};
use crate::utils::error_handler::*;
use crate::services::user::social::get_user_social_info;


pub fn create_comment(request: CreateCommentRequest, caller: Principal) -> SquareResult<CommentResponse> {
    const MODULE: &str = "services::content::comments";
    const FUNCTION: &str = "create_comment";
    
    // Validate content length
    let content_length = calculate_content_length_excluding_base64_and_html(&request.content);
    if content_length > MAX_COMMENT_LENGTH {
        return log_and_return(content_too_long_error(
            "Comment",
            MAX_COMMENT_LENGTH,
            content_length,
            MODULE,
            FUNCTION
        ));
    }
    
    let now = time() / 1_000_000;
    let comment_id = format!("comment_{}", now);
    
    let comment = Comment {
        id: comment_id.clone(),
        parent_id: request.parent_id.clone(),
        parent_type: request.parent_type,
        author: caller,
        content: request.content,
        created_at: now,
        updated_at: now,
        status: ContentStatus::Active,
        likes_count: 0,
        child_comments: Vec::new(),
    };
    
    // Store comment in main storage
    STORAGE.with(|storage| {
        let mut store = storage.borrow_mut();
        
        // Add comment to parent's child_comments list
        match request.parent_type {
            ParentType::Post => {
                if let Some(post) = store.posts.get(&request.parent_id) {
                    if post.status != ContentStatus::Active {
                        return log_and_return(validation_error(
                            "Cannot comment on an inactive post",
                            MODULE,
                            FUNCTION
                        ));
                    }
                } else {
                    return log_and_return(not_found_error(
                        "Parent post",
                        &request.parent_id,
                        MODULE,
                        FUNCTION
                    ));
                }
            }
            ParentType::Comment => {
                if let Some(parent_comment) = store.comments.get_mut(&request.parent_id) {
                    if parent_comment.status != ContentStatus::Active {
                        return log_and_return(validation_error(
                            "Cannot reply to an inactive comment",
                            MODULE,
                            FUNCTION
                        ));
                    }
                    parent_comment.child_comments.push(comment_id.clone());
                } else {
                    return log_and_return(not_found_error(
                        "Parent comment",
                        &request.parent_id,
                        MODULE,
                        FUNCTION
                    ));
                }
            }
        }
        
        store.comments.insert(comment_id.clone(), comment.clone());
        Ok(CommentResponse {
            comments_count: 0,
            is_liked: false,
            id: comment.id,
            parent_id: comment.parent_id,
            parent_type: comment.parent_type,
            author: comment.author,
            content: comment.content,
            created_at: comment.created_at,
            updated_at: comment.updated_at,
            status: comment.status.clone(),
            visibility: ContentVisibility::Public,
            likes_count: comment.likes_count,
            child_comments: Vec::new(),
            author_info: get_user_social_info(comment.author.to_string(), None)?
        })
    })
}

pub fn get_comment(id: String, caller: Option<Principal>) -> SquareResult<CommentResponse> {
    const MODULE: &str = "services::content::comments";
    const FUNCTION: &str = "get_comment";
    
    STORAGE.with(|storage| {
        let store = storage.borrow();
        
        let comment = store.comments.get(&id).ok_or_else(|| {
            not_found_error("Comment", &id, MODULE, FUNCTION)
        })?;
        
        // Check visibility
        if caller != Some(comment.author) {
            return log_and_return(unauthorized_error(
                "This comment is private",
                MODULE,
                FUNCTION
            ));
        }
        
        Ok(comment.clone().into())
    })
}

pub fn update_comment(request: UpdateCommentRequest, caller: Principal) -> SquareResult<CommentResponse> {
    const MODULE: &str = "services::content::comments";
    const FUNCTION: &str = "update_comment";
    
    STORAGE.with(|storage| {
        let mut store = storage.borrow_mut();
        
        let comment = store.comments.get_mut(&request.id).ok_or_else(|| {
            not_found_error("Comment", &request.id, MODULE, FUNCTION)
        })?;
        
        // Check if caller is the author or admin
        if comment.author != caller && is_admin().is_err() {
            return log_and_return(unauthorized_error(
                "Only the author or admin can update this comment",
                MODULE,
                FUNCTION
            ));
        }
        
        // Update content
        let content = &request.content;
        // Validate content length
        let content_length = calculate_content_length_excluding_base64_and_html(&content);
        if content_length > MAX_COMMENT_LENGTH {
            return log_and_return(content_too_long_error(
                "Comment",
                MAX_COMMENT_LENGTH,
                content_length,
                MODULE,
                FUNCTION
            ));
        }
        comment.content = content.clone();
        
        comment.updated_at = time() / 1_000_000;
        
        Ok(CommentResponse {
            comments_count: 0,
            is_liked: false,
            visibility: ContentVisibility::Public,
            id: comment.id.clone(),
            parent_id: comment.parent_id.clone(),
            parent_type: comment.parent_type,
            author: comment.author,
            content: comment.content.clone(),
            created_at: comment.created_at,
            updated_at: comment.updated_at,
            status: comment.status.clone(),
            likes_count: comment.likes_count,
            child_comments: get_child_comments(&comment.child_comments, Some(caller))?,
            author_info: get_user_social_info(comment.author.to_string(), None)?
        })
    })
}

pub fn delete_comment(id: String, caller: Principal) -> SquareResult<()> {
    const MODULE: &str = "services::content::comments";
    const FUNCTION: &str = "delete_comment";
    
    STORAGE.with(|storage| {
        let mut store = storage.borrow_mut();
        
        let comment = store.comments.get(&id).ok_or_else(|| {
            not_found_error("Comment", &id, MODULE, FUNCTION)
        })?;
        
        // Check if caller is the author or admin
        if comment.author != caller && is_admin().is_err() {
            return log_and_return(unauthorized_error(
                "Only the author or admin can delete this comment",
                MODULE,
                FUNCTION
            ));
        }
        
        let parent_id = comment.parent_id.clone();
        let parent_type = comment.parent_type;
        
        // Remove comment from storage first
        store.comments.remove(&id);
        
        // Then update parent's child_comments list
        match parent_type {
            ParentType::Comment => {
                if let Some(parent_comment) = store.comments.get_mut(&parent_id) {
                    parent_comment.child_comments.retain(|c| c != &id);
                }
            }
            _ => {}
        }
        
        Ok(())
    })
}

// Helper function to recursively get child comments
pub fn get_child_comments(comment_ids: &Vec<String>, caller: Option<Principal>) -> SquareResult<Vec<Box<CommentResponse>>> {
    STORAGE.with(|storage| {
        let store = storage.borrow();
        let mut child_comments: Vec<Box<CommentResponse>> = Vec::new();
        
        for child_id in comment_ids {
            if let Some(comment) = store.comments.get(child_id) {
                // Skip private comments if caller is not the author
                if caller.is_none() || caller != Some(comment.author) {
                    continue;
                }
                
                let child_response = CommentResponse {
                    comments_count: comment.child_comments.len() as u64,
                    is_liked: false, // TODO: Implement like check
                    visibility: ContentVisibility::Public,
                    id: comment.id.clone(),
                    parent_id: comment.parent_id.clone(),
                    parent_type: comment.parent_type,
                    author: comment.author,
                    content: comment.content.clone(),
                    created_at: comment.created_at,
                    updated_at: comment.updated_at,
                    status: comment.status.clone(),
                    likes_count: comment.likes_count,
                    child_comments: get_child_comments(&comment.child_comments, caller)?,
                    author_info: get_user_social_info(comment.author.to_string(), None)?
                };
                child_comments.push(Box::new(child_response));
            }
        }
        
        Ok(child_comments)
    })
}

pub fn get_comments(parent_id: String, parent_type_str: String, pagination: PaginationParams, caller: Option<Principal>) -> SquareResult<CommentsResponse> {
    const MODULE: &str = "services::content::comments";
    const FUNCTION: &str = "get_comments";
    
    let parent_type = match parent_type_str.as_str() {
        "post" => ParentType::Post,
        "comment" => ParentType::Comment,
        _ => return log_and_return(validation_error(
            "Invalid parent type. Must be 'post' or 'comment'",
            MODULE,
            FUNCTION
        )),
    };
    
    let limit = pagination.limit;
    let offset = pagination.offset;
    
    // Get comments from storage
    let mut comments = STORAGE.with(|storage| {
        let store = storage.borrow();
        store.comments
            .values()
            .filter(|c| c.parent_id == parent_id && c.parent_type == parent_type)
            .cloned()
            .collect::<Vec<Comment>>()
    });
    
    // Sort by creation time (newest first)
    comments.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    
    // Apply pagination
    let total = comments.len() as u64;
    let start = offset.unwrap_or(0);
    let limit = limit.unwrap_or(10);
    let end = (start + limit).min(comments.len());
    let comments = comments[start..end].to_vec();
    
    // Convert to response format and get child comments
    let mut comments_result = Vec::new();
    for c in comments {
        let mut response: CommentResponse = c.clone().into();
        response.child_comments = get_child_comments(&c.child_comments, caller)?;
        comments_result.push(response);
    }
    
    Ok(CommentsResponse {
        comments: comments_result,
        total,
        has_more: (start as u64 + limit as u64) < total,
        next_offset: start + limit,
    })
}
