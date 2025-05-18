use ic_cdk::api::time;
use candid::Principal;
use std::collections::{HashMap, HashSet, BTreeMap};

use crate::models::discovery::*;
use crate::models::error::SquareResult;
use crate::models::tag::TagType;
use crate::storage::STORAGE;
use std::borrow::{Borrow, BorrowMut};
use crate::utils::error_handler::*;

// Constants
const ONE_DAY: u64 = 24 * 60 * 60 * 1000; // 24 hours in milliseconds

// Define TrendingTopics struct
#[derive(candid::CandidType, candid::Deserialize, Clone, Default)]
pub struct TrendingTopics {
    pub trending_topics: Vec<String>,
    pub trending_content: HashSet<String>,
    pub current_trending_topics: HashMap<String, u64>,
    pub previous_trending_topics: HashMap<String, u64>,
    pub last_updated: u64,
    pub last_update: u64,
}

// This function initializes trending topics in storage if it doesn't exist
pub fn init_trending_topics() {
    STORAGE.with(|storage| {
        let mut store = storage.borrow_mut();
        // No need to check if trending_topics is none since it's not an Option type
        // Just ensure it's initialized if empty
        if store.trending_topics.is_empty() {
            // Initialize with empty BTreeMap
            store.trending_topics = BTreeMap::new();
        }
    });
}

// Helper function to calculate trending score for content
fn calculate_trending_score(content_id: &str) -> f64 {
    // Calculate a trending score based on likes, comments, and recency
    let now = time() / 1_000_000; // Convert nanoseconds to milliseconds
    
    // Get likes count from main storage
    let likes_count = STORAGE.with(|storage| {
        let store = storage.borrow();
        store.likes.get(content_id).map_or(0, |likes| likes.len() as u64)
    });
    
    // Get comments count from main storage
    let comments_count = STORAGE.with(|storage| {
        let store = storage.borrow();
        store.comments.get(content_id).map_or(0, |comments| comments.child_comments.len() as u64)
    });
    
    // Get content creation time from main storage
    let creation_time = STORAGE.with(|storage| {
        let store = storage.borrow();
        store.posts.get(content_id).map_or(now, |post| post.created_at)
    });
    
    // Calculate time decay factor (1 day half-life)
    let time_diff = now - creation_time;
    let time_decay = 1.0 / (1.0 + (time_diff as f64 / ONE_DAY as f64));
    
    // Calculate engagement score
    let engagement_score = (likes_count * 2 + comments_count * 3) as f64;
    
    // Final trending score
    engagement_score * time_decay
}

pub fn get_trending_topics(request: GetTrendingTopicsRequest) -> SquareResult<Vec<TrendingTopicResponse>> {
    const MODULE: &str = "services::discovery::trending";
    const FUNCTION: &str = "get_trending_topics";
    
    let limit = request.limit.unwrap_or(10);
    let _now = time() / 1_000_000;
    
    // Get trending topics from storage
    let mut trending_topics: Vec<TrendingTopicResponse> = STORAGE.with(|storage| {
        let store = storage.borrow();
        let mut topics: Vec<(String, u64)> = store.trending_topics
            .iter()
            .map(|(topic, count)| (topic.clone(), *count))
            .collect();
        
        // Sort by count in descending order
        topics.sort_by(|a, b| b.1.cmp(&a.1));
        
        // Take only the requested number of topics
        topics.truncate(limit as usize);
        
        // Convert to response format
        topics.into_iter()
            .map(|(topic, count)| TrendingTopicResponse {
                topic,
                count,
                trend_direction: TrendDirection::Stable, // TODO: Calculate trend direction based on previous data
            })
            .collect()
    });
    
    // Sort by count
    trending_topics.sort_by(|a, b| b.count.cmp(&a.count));
    
    Ok(trending_topics)
}

pub fn get_hot_tags(request: GetHotTagsRequest) -> SquareResult<HotTagsResponse> {
    const MODULE: &str = "services::discovery::trending";
    const FUNCTION: &str = "get_hot_tags";
    
    let limit = request.limit.unwrap_or(10);
    
    let hot_tags = STORAGE.with(|storage| {
        let store = storage.borrow();
        let mut tags: Vec<(String, u64)> = store.posts
            .values()
            .flat_map(|post| post.hashtags.clone())
            .fold(std::collections::HashMap::new(), |mut acc, tag| {
                *acc.entry(tag).or_insert(0) += 1;
                acc
            })
            .into_iter()
            .map(|(tag, count)| (tag, count))
            .collect();
        
        // Sort by count in descending order
        tags.sort_by(|a, b| b.1.cmp(&a.1));
        
        // Take only the requested number of tags
        tags.truncate(limit as usize);
        
        tags.into_iter()
            .map(|(tag, count)| HotTagInfo {
                name: tag,
                count: count as u64,
                tag_type: TagType::Topic
            })
            .collect()
    });
    
    Ok(HotTagsResponse {
        tags: hot_tags,
        updated_at: time() / 1_000_000,
    })
}

pub fn update_trending_content(topics: Vec<String>) -> SquareResult<()> {
    const MODULE: &str = "services::discovery::trending";
    const FUNCTION: &str = "update_trending_content";
    
    STORAGE.with(|storage| {
        let mut store = storage.borrow_mut();
        
        // Update trending topics
        for topic in topics {
            let count = store.trending_topics.entry(topic).or_insert(0);
            *count += 1;
        }
    });
    
    Ok(())
}
