use candid::Principal;
use ic_cdk::api::time;
use std::collections::{HashMap, HashSet};

use crate::auth::is_admin;
use crate::models::content::{
    CreatePostRequest, UpdatePostRequest, ContentStatus, ContentVisibility,
    ContentType, PostResponse, PostsResponse, PaginationParams,
    MAX_MEDIA_URLS, MAX_TOKEN_MENTIONS, MAX_HASHTAGS, MAX_POST_LENGTH,
};
use crate::models::storage::Storage;
use crate::utils::content_utils::calculate_content_length_excluding_base64_and_html;
use crate::{SquareError, SquareResult};
use crate::storage::{Post, STORAGE};
use crate::utils::error_handler::*;
use crate::services::user::social::get_user_social_info;


pub fn create_post(request: CreatePostRequest, caller: Principal) -> SquareResult<PostResponse> {
    const MODULE: &str = "services::content::posts";
    const FUNCTION: &str = "create_post";
    
    // Validate content length (excluding HTML tags and base64 images/videos)
    let content_length = calculate_content_length_excluding_base64_and_html(&request.content);
    if content_length > MAX_POST_LENGTH {
        return log_and_return(content_too_long_error(
            "Post", 
            MAX_POST_LENGTH, 
            content_length, 
            MODULE, 
            FUNCTION
        ));
    }
    
    // Validate hashtags count
    if request.hashtags.len() > MAX_HASHTAGS {
        return log_and_return(validation_error(
            &format!("Too many hashtags. Maximum allowed is {}", MAX_HASHTAGS),
            MODULE,
            FUNCTION
        ));
    }
    
    // Validate token mentions count
    if let Some(token_mentions) = &request.token_mentions {
        if token_mentions.len() > MAX_TOKEN_MENTIONS {
            return log_and_return(validation_error(
                &format!("Too many token mentions. Maximum allowed is {}", MAX_TOKEN_MENTIONS),
                MODULE,
                FUNCTION
            ));
        }
    }
    
    // Validate media URLs count
    let media_urls = request.media_urls;
    if media_urls.len() > MAX_MEDIA_URLS {
        return log_and_return(validation_error(
            &format!("Too many media URLs. Maximum allowed is {}", MAX_MEDIA_URLS),
            MODULE,
            FUNCTION
        ));
    }
    
    let now = time() / 1_000_000;
    let post_id = format!("post_{}", now);
    
    let post = Post {
        id: post_id.clone(),
        author: caller,
        content: request.content,
        hashtags: request.hashtags,
        token_mentions: request.token_mentions.unwrap_or_default(),
        media_urls: media_urls,
        tags: request.tags.unwrap_or_default(),
        created_at: now,
        updated_at: now,
        status: ContentStatus::Active,
        visibility: request.visibility.unwrap_or(ContentVisibility::Public),
        news_reference: request.news_reference,
    };
    
    // Store post in main storage
    STORAGE.with(|storage| {
        let mut store = storage.borrow_mut();
        store.posts.insert(post_id.clone(), post.clone());
    });
    
    Ok(PostResponse {
        id: post.id,
        author: post.author,
        content: post.content,
        media_urls: post.media_urls,
        hashtags: post.hashtags,
        token_mentions: post.token_mentions,
        tags: post.tags,
        created_at: post.created_at,
        updated_at: post.updated_at,
        status: post.status,
        visibility: post.visibility,
        likes_count: 0, // TODO: Get from likes storage
        comments_count: 0, // TODO: Get from comments storage
        author_info: get_user_social_info(post.author.to_string(), None)?,
        news_reference: post.news_reference,
    })
}

pub fn get_post(id: String) -> SquareResult<PostResponse> {
    const MODULE: &str = "services::content::posts";
    const FUNCTION: &str = "get_post";
    
    STORAGE.with(|storage| {
        let store = storage.borrow();
        let post = store.posts
            .get(&id)
            .cloned()
            .ok_or_else(|| not_found_error("Post", &id, MODULE, FUNCTION))?;
        
        Ok(PostResponse {
            id: post.id.clone(),
            author: post.author,
            content: post.content.clone(),
            media_urls: post.media_urls.clone(),
            hashtags: post.hashtags.clone(),
            token_mentions: post.token_mentions.clone(),
            tags: post.tags.clone(),
            created_at: post.created_at,
            updated_at: post.updated_at,
            status: post.status.clone(),
            visibility: post.visibility.clone(),
            likes_count: 0, // TODO: Get from likes storage
            comments_count: 0, // TODO: Get from comments storage
            author_info: get_user_social_info(post.author.to_string(), None)?,
            news_reference: post.news_reference.clone(),
        })
    })
}

pub fn get_posts(pagination: PaginationParams) -> SquareResult<PostsResponse> {
    const MODULE: &str = "services::content::posts";
    const FUNCTION: &str = "get_posts";
    
    let limit = pagination.limit;
    let offset = pagination.offset;
    
    // Get posts from storage
    let mut posts = STORAGE.with(|storage| {
        let store = storage.borrow();
        store.posts.values().cloned().collect::<Vec<Post>>()
    });
    
    // Sort by creation time (newest first)
    posts.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    
    // Apply pagination
    let _total = posts.len() as u64;
    let start = offset.unwrap_or(0);
    let limit = limit.unwrap_or(10);
    let end = (start + limit).min(posts.len());
    let posts_slice = posts[start..end].to_vec();
    let posts_len = posts.len();
    
    Ok(PostsResponse {
        posts: posts_slice.into_iter().map(|p| -> Result<PostResponse, SquareError> {
            Ok(PostResponse {
            id: p.id,
            author: p.author,
            content: p.content,
            media_urls: p.media_urls,
            hashtags: p.hashtags,
            token_mentions: p.token_mentions,
            tags: p.tags,
            created_at: p.created_at,
            updated_at: p.updated_at,
            status: p.status,
            visibility: p.visibility,
            likes_count: 0, // TODO: Get from likes storage
            comments_count: 0, // TODO: Get from comments storage
            author_info: get_user_social_info(p.author.to_string(), None)?,
            news_reference: p.news_reference,
        })
        }).collect::<Result<Vec<_>, _>>()?,
        next_offset: if (start + limit) < posts_len { start + limit } else { posts_len },
        total: posts_len as u64,
    })
}

pub fn update_post(request: UpdatePostRequest, caller: Principal) -> SquareResult<PostResponse> {
    const MODULE: &str = "services::content::posts";
    const FUNCTION: &str = "update_post";
    
    STORAGE.with(|storage| {
        let mut store = storage.borrow_mut();
        
        let post = store.posts.get_mut(&request.id).ok_or_else(|| {
            not_found_error("Post", &request.id, MODULE, FUNCTION)
        })?;
        
        // Check if caller is the author or admin
        if post.author != caller {
            match is_admin() {
                Ok(_) => {},
                Err(_) => {
                    return log_and_return(unauthorized_error(
                        "Only the author or admin can update this post",
                        MODULE,
                        FUNCTION
                    ));
                }
            }
        }
        
        // Validate content length
        let content_length = calculate_content_length_excluding_base64_and_html(&request.content);
        if content_length > MAX_POST_LENGTH {
            return log_and_return(content_too_long_error(
                "Post",
                MAX_POST_LENGTH,
                content_length,
                MODULE,
                FUNCTION
            ));
        }
        post.content = request.content;
        
        if let Some(hashtags) = request.hashtags {
            if hashtags.len() > MAX_HASHTAGS {
                return log_and_return(validation_error(
                    &format!("Too many hashtags. Maximum allowed is {}", MAX_HASHTAGS),
                    MODULE,
                    FUNCTION
                ));
            }
            post.hashtags = hashtags;
        }
        
        if let Some(token_mentions) = request.token_mentions {
            if token_mentions.len() > MAX_TOKEN_MENTIONS {
                return log_and_return(validation_error(
                    &format!("Too many token mentions. Maximum allowed is {}", MAX_TOKEN_MENTIONS),
                    MODULE,
                    FUNCTION
                ));
            }
            post.token_mentions = token_mentions;
        }
        
        if let Some(media_urls) = request.media_urls {
            if media_urls.len() > MAX_MEDIA_URLS {
                return log_and_return(validation_error(
                    &format!("Too many media URLs. Maximum allowed is {}", MAX_MEDIA_URLS),
                    MODULE,
                    FUNCTION
                ));
            }
            post.media_urls = media_urls;
        }
        
        if let Some(tags) = request.tags {
            post.tags = tags;
        }
        
        if let Some(visibility) = request.visibility {
            post.visibility = visibility;
        }
        
        if let Some(news_reference) = request.news_reference {
            post.news_reference = Some(news_reference);
        }
        
        post.updated_at = time() / 1_000_000;
        
        Ok(PostResponse {
            id: post.id.clone(),
            author: post.author,
            content: post.content.clone(),
            media_urls: post.media_urls.clone(),
            hashtags: post.hashtags.clone(),
            token_mentions: post.token_mentions.clone(),
            tags: post.tags.clone(),
            created_at: post.created_at,
            updated_at: post.updated_at,
            status: post.status.clone(),
            visibility: post.visibility.clone(),
            likes_count: 0, // TODO: Get from likes storage
            comments_count: 0, // TODO: Get from comments storage
            author_info: get_user_social_info(post.author.to_string(), None)?,
            news_reference: post.news_reference.clone(),
        })
    })
}

pub fn delete_post(id: String, caller: Principal) -> SquareResult<()> {
    const MODULE: &str = "services::content::posts";
    const FUNCTION: &str = "delete_post";
    
    STORAGE.with(|storage| {
        let mut store = storage.borrow_mut();
        
        let post = store.posts.get(&id).ok_or_else(|| {
            not_found_error("Post", &id, MODULE, FUNCTION)
        })?;
        
        // Check if caller is the author or admin
        if post.author != caller && is_admin().is_err() {
            return log_and_return(unauthorized_error(
                "Only the author or admin can delete this post",
                MODULE,
                FUNCTION
            ));
        }
        
        // Remove post from storage
        store.posts.remove(&id);
        
        Ok(())
    })
}
