use candid::Principal;
use std::collections::HashSet;

use crate::models::content::{ContentType, ContentStatus, ContentVisibility, PostResponse, PaginationParams};
use crate::models::display::FeedResponse;
use crate::models::display::*;
use crate::models::error::{SquareResult, SquareError};
use crate::storage::{STORAGE, Post};
use std::borrow::Borrow;
use crate::utils::error_handler::*;
use crate::services::user::social::get_user_social_info;

pub fn get_user_content(user_identifier: String, content_type: Option<ContentType>, pagination: PaginationParams) -> SquareResult<FeedResponse> {
    const MODULE: &str = "services::content::display";
    const FUNCTION: &str = "get_user_content";
    
    let limit = pagination.limit;
    let offset = pagination.offset;
    let content_type = content_type.unwrap_or(ContentType::Post);
    
    // Get user's content from storage
    let mut feed_items = STORAGE.with(|storage| -> SquareResult<Vec<PostResponse>> {
        let store = storage.borrow();
        
        // Try to find user by principal or handle
        let user_principal = if let Ok(principal) = Principal::from_text(&user_identifier) {
            principal
        } else {
            // If not a valid principal, try to find by handle
            store.user_profiles.as_ref().and_then(|profiles| profiles.values().find(|profile| profile.handle == user_identifier)).map(|profile| profile.principal).ok_or_else(|| not_found_error("User", &user_identifier, MODULE, FUNCTION))?

        };
        
        // Get user's content
        match content_type {
            ContentType::Post => {
                let posts: Vec<SquareResult<PostResponse>> = store.posts
                    .values()
                    .filter(|post| post.author == user_principal)
                    .map(|post| -> SquareResult<PostResponse> {
                        let author_info = get_user_social_info(post.author.to_string(), None)?;
                        Ok(PostResponse {
                            hashtags: post.hashtags.clone(),
                            status: post.status.clone(),
                            tags: post.tags.clone(),
                            token_mentions: vec![], // TODO: Implement token mentions
                            id: post.id.clone(),
                            author: post.author,
                            content: post.content.clone(),
                            media_urls: post.media_urls.clone(),
                            created_at: post.created_at,
                            updated_at: post.updated_at,
                            visibility: post.visibility.clone(),
                            likes_count: 0, // TODO: Get from likes storage
                            comments_count: 0, // TODO: Get from comments storage
                            author_info,
                            news_reference: post.news_reference.clone(),
                        })
                    })
                    .collect();
                
                // Convert Vec<Result> to Result<Vec>
                let posts = posts.into_iter().collect::<Result<Vec<_>, _>>()?;
                Ok(posts)
            }
            ContentType::Comment => {
                let comments: Vec<SquareResult<PostResponse>> = store.comments
                    .values()
                    .filter(|comment| comment.author == user_principal)
                    .map(|comment| -> SquareResult<PostResponse> {
                        let author_info = get_user_social_info(comment.author.to_string(), None)?;
                        Ok(PostResponse {
                            hashtags: vec![],
                            status: ContentStatus::Active,
                            tags: vec![],
                            token_mentions: vec![], // TODO: Implement token mentions
                            id: comment.id.clone(),
                            author: comment.author,
                            content: comment.content.clone(),
                            media_urls: vec![],
                            created_at: comment.created_at,
                            updated_at: comment.updated_at,
                            visibility: ContentVisibility::Public,
                            likes_count: 0, // TODO: Get from likes storage
                            comments_count: 0, // TODO: Get from comments storage
                            author_info,
                            news_reference: None,
                        })
                    })
                    .collect();
                
                // Convert Vec<Result> to Result<Vec>
                let comments = comments.into_iter().collect::<Result<Vec<_>, _>>()?;
                Ok(comments)
            }
        }
    })?;
    
    // Sort by creation time (newest first)
    feed_items.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    
    // Apply pagination
    let total = feed_items.len() as u64;
    let start = offset.unwrap_or(0);
    let limit = limit.unwrap_or(10);
    let end = (start + limit).min(feed_items.len());
    let feed_items = feed_items[start..end].to_vec();
    
    Ok(FeedResponse {
        posts: feed_items,
        comments: vec![],
        has_more: (start as u64 + limit as u64) < total,
        next_offset: start + limit,
        total,
    })
}

pub fn get_content_detail(content_id: String, content_type: ContentType, caller: Option<Principal>) -> SquareResult<ContentDetailResponse> {
    const MODULE: &str = "services::content::display";
    const FUNCTION: &str = "get_content_detail";
    
    STORAGE.with(|storage| {
        let store = storage.borrow();
        
        match content_type {
            ContentType::Post => {
                let post = store.posts.get(&content_id).ok_or_else(|| {
                    not_found_error("Post", &content_id, MODULE, FUNCTION)
                })?;
                
                // Get likes count
                let _likes_count = store.likes
                    .get(&content_id)
                    .map_or(0, |likes| likes.len() as u64);
                
                // Get comments count
                let _comments_count = store.comments
                    .values()
                    .filter(|c| c.parent_id == content_id && c.parent_type == crate::models::content::ParentType::Post)
                    .count() as u64;
                
                // Check if caller has liked the content
                let _has_liked = if let Some(caller) = caller {
                    store.likes
                        .get(&content_id)
                        .map_or(false, |likes| likes.contains(&caller))
                } else {
                    false
                };
                
                Ok(ContentDetailResponse {
                    post: Some(PostResponse {
                        hashtags: post.hashtags.clone(),
                        status: post.status.clone(),
                        tags: post.tags.clone(),
                        token_mentions: vec![], // TODO: Implement token mentions
                        id: post.id.clone(),
                        author: post.author,
                        content: post.content.clone(),
                        media_urls: post.media_urls.clone(),
                        created_at: post.created_at,
                        updated_at: post.updated_at,
                        visibility: post.visibility.clone(),
                        likes_count: 0, // TODO: Get from likes storage
                        comments_count: 0, // TODO: Get from comments storage
                        author_info: get_user_social_info(post.author.to_string(), None)?,
                        news_reference: post.news_reference.clone(),
                    }),
                    comments: vec![],
                    has_more_comments: false,
                    next_comment_offset: 0,
                })
            }
            ContentType::Comment => {
                let comment = store.comments.get(&content_id).ok_or_else(|| {
                    not_found_error("Comment", &content_id, MODULE, FUNCTION)
                })?;
                
                // Get likes count
                let _likes_count = store.likes
                    .get(&content_id)
                    .map_or(0, |likes| likes.len() as u64);
                
                // Get replies count
                let _comments_count = comment.child_comments.len() as u64;
                
                // Check if caller has liked the content
                let _has_liked = if let Some(caller) = caller {
                    store.likes
                        .get(&content_id)
                        .map_or(false, |likes| likes.contains(&caller))
                } else {
                    false
                };
                
                Ok(ContentDetailResponse {
                    post: Some(PostResponse {
                        hashtags: vec![],
                        status: ContentStatus::Active,
                        tags: vec![],
                        token_mentions: vec![], // TODO: Implement token mentions
                        id: comment.id.clone(),
                        author: comment.author,
                        content: comment.content.clone(),
                        media_urls: vec![], // Comments don't have media URLs
                        created_at: comment.created_at,
                        updated_at: comment.updated_at,
                        visibility: ContentVisibility::Public, // Comments are always public
                        likes_count: 0, // TODO: Get from likes storage
                        comments_count: 0, // TODO: Get from comments storage
                        author_info: get_user_social_info(comment.author.to_string(), None)?,
                        news_reference: None,
                    }),
                    comments: vec![],
                    has_more_comments: false,
                    next_comment_offset: 0,
                })
            }
        }
    })
}
