use candid::Principal;
use std::collections::HashSet;

use crate::models::content::{ContentType, PostResponse};
use crate::models::discovery::*;
use crate::models::display::FeedResponse;
use crate::models::error::{SquareResult, SquareError};
use crate::storage::{STORAGE, Post};
use std::borrow::Borrow;
use crate::utils::error_handler::*;
use crate::models::user::UserSocialResponse;
use crate::services::user::social::get_user_social_info;

pub fn discover_content(request: DiscoverContentRequest) -> SquareResult<FeedResponse> {
    const MODULE: &str = "services::discovery::search";
    const FUNCTION: &str = "discover_content";
    
    let limit = request.pagination.limit;
    let offset = request.pagination.offset;
    let _content_type = request.content_types.unwrap_or_else(|| vec![ContentType::Post])[0].clone();
    let tags = request.tags.unwrap_or_default();
    
    // Get all posts from storage
    let mut posts = STORAGE.with(|storage| {
        let store = storage.borrow();
        store.posts.values().cloned().collect::<Vec<Post>>()
    });
    
    // Filter by content type
    // Filter by content type
    posts.retain(|post| post.hashtags.iter().any(|tag| tag.starts_with("#")));
    
    // Filter by tags if specified
    if !tags.is_empty() {
        posts.retain(|post| {
            post.tags.iter().any(|tag| tags.contains(tag))
        });
    }
    
    // Sort by creation time (newest first)
    posts.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    
    // Apply pagination
    let total = posts.len() as u64;
    let start = offset.unwrap_or(0);
    let limit_value = limit.unwrap_or(10);
    let end = (start + limit_value).min(posts.len());
    let posts = posts[start..end].to_vec();
    
    // Convert to response format
    let feed_items = posts.into_iter()
        .map(|post| Ok::<_, SquareError>(PostResponse {
            id: post.id.clone(),
            author: post.author,
            content: post.content.clone(),
            media_urls: post.media_urls.clone(),
            hashtags: post.hashtags.clone(),
            token_mentions: post.token_mentions.clone(),
            tags: post.tags.clone(),
            created_at: post.created_at,
            updated_at: post.updated_at,
            status: post.status,
            visibility: post.visibility,
            likes_count: 0, // TODO: Get from likes storage
            comments_count: 0, // TODO: Get from comments storage
            author_info: get_user_social_info(post.author.to_string(), None).unwrap_or_else(|_| UserSocialResponse {
                principal: post.author,
                username: String::from("Unknown"),
                handle: String::from("unknown"),
                avatar: String::new(),
                bio: String::new(),
                followers_count: 0,
                following_count: 0,
                is_following: false,
                interests: vec![],
                is_followed_by_caller: false
            }),
            news_reference: post.news_reference.clone(),
        }))
        .collect::<Result<Vec<_>, _>>()?;
    
    Ok(FeedResponse {
        posts: feed_items,
        comments: vec![],
        has_more: (start as u64 + limit_value as u64) < total,
        next_offset: start + limit_value,
        total,
    })
}

pub fn search_content(request: SearchRequest) -> SquareResult<Vec<SearchResultResponse>> {
    const MODULE: &str = "services::discovery::search";
    const FUNCTION: &str = "search_content";
    
    let query = request.query.to_lowercase();
    
    // Get pagination parameters with defaults
    let limit = request.pagination.limit.unwrap_or(10);
    
    let _content_type = request.content_types.unwrap_or_else(|| vec![ContentType::Post])[0].clone();
    let tags: HashSet<String> = HashSet::new(); // We don't filter by tags in search
    
    // Get all posts from storage
    let mut results = STORAGE.with(|storage| {
        let store = storage.borrow();
        let mut search_results = Vec::new();
        
        for post in store.posts.values() {
            // Skip if content type doesn't match
            if !post.hashtags.iter().any(|tag| tag.starts_with("#")) {
                continue;
            }
            
            // Skip if tags don't match (if tags are specified)
            if !tags.is_empty() && !post.tags.iter().any(|tag| tags.contains(tag)) {
                continue;
            }
            
            // Check if query matches content or tags
            let content_match = post.content.to_lowercase().contains(&query);
            let tag_match = post.tags.iter().any(|tag| tag.to_lowercase().contains(&query));
            
            if content_match || tag_match {
                let snippet = if content_match {
                    create_snippet(&post.content, &query)
                } else {
                    post.content[..100.min(post.content.len())].to_string()
                };
                
                search_results.push(SearchResultResponse {
                    id: post.id.clone(),
                    title: None,
                    snippet,
                    content_type: ContentType::Post,
                    created_at: post.created_at,
                    author: get_user_social_info(post.author.to_string(), None).unwrap_or_else(|_| UserSocialResponse {
                        principal: post.author,
                        username: String::from("Unknown"),
                        handle: String::from("unknown"),
                        avatar: String::new(),
                        bio: String::new(),
                        followers_count: 0,
                        following_count: 0,
                        is_following: false,
                        interests: vec![],
                        is_followed_by_caller: false
                    }),
                    relevance_score: calculate_relevance_score(
                        false,
                        content_match,
                        tag_match,
                        post.created_at,
                    ),
                });
            }
        }
        
        search_results
    });
    
    // Sort by relevance score
    results.sort_by(|a, b| b.relevance_score.partial_cmp(&a.relevance_score).unwrap());
    
    // Apply limit
    results.truncate(limit as usize);
    
    Ok(results)
}

// Helper function to create a snippet from content with query highlighted
fn create_snippet(content: &str, query: &str) -> String {
    let content_lower = content.to_lowercase();
    let query_pos = content_lower.find(query).unwrap_or(0);
    
    // Get surrounding context (50 chars before and after)
    let start = query_pos.saturating_sub(50);
    let end = (query_pos + query.len() + 50).min(content.len());
    
    let mut snippet = String::new();
    if start > 0 {
        snippet.push_str("...");
    }
    snippet.push_str(&content[start..end]);
    if end < content.len() {
        snippet.push_str("...");
    }
    
    snippet
}

// Helper function to calculate relevance score
fn calculate_relevance_score(title_match: bool, content_match: bool, tag_match: bool, created_at: u64) -> f64 {
    let base_score = if title_match { 1.0 } else { 0.0 }
        + if content_match { 0.5 } else { 0.0 }
        + if tag_match { 0.3 } else { 0.0 };
    
    // Apply time decay
    let now = ic_cdk::api::time() / 1_000_000;
    let age_days = (now - created_at) as f64 / (24.0 * 60.0 * 60.0 * 1000.0);
    let time_decay = 1.0 / (1.0 + age_days / 7.0); // Half-life of 7 days
    
    base_score * time_decay
}
