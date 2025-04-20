use candid::{CandidType, Deserialize, Principal};
use crate::models::content::{ContentType, PaginationParams};
use crate::models::tag::TagType;

// Request DTOs
#[derive(CandidType, Deserialize, Clone)]
pub struct DiscoverContentRequest {
    pub content_types: Option<Vec<ContentType>>,
    pub tags: Option<Vec<String>>,
    pub pagination: PaginationParams,
    pub sort_by: Option<SortOption>,
    pub filter: Option<ContentFilter>
}

#[derive(CandidType, Deserialize, Clone)]
pub struct ContentFilter {
    pub hashtag: Option<String>,
    pub token_mention: Option<String>,
    pub created_after: Option<u64>,
    pub created_before: Option<u64>,
    pub author: Option<Principal>
}

#[derive(CandidType, Deserialize, Clone, PartialEq, Debug)]
pub enum SortOption {
    MostShared,
    MostCommented,
    Trending,
    MostLiked,
    Latest
}

#[derive(CandidType, Deserialize, Clone)]
pub struct SearchRequest {
    pub query: String,
    pub content_types: Option<Vec<ContentType>>,
    pub pagination: PaginationParams
}

#[derive(CandidType, Deserialize, Clone)]
pub struct GetTrendingTopicsRequest {
    pub limit: Option<u32>,
    pub time_range_hours: Option<u32>
}

#[derive(CandidType, Deserialize, Clone)]
pub struct GetHotTagsRequest {
    pub tag_type: Option<TagType>,
    pub limit: Option<u32>
}

#[derive(CandidType, Deserialize, Clone)]
pub struct PersonalizedRecommendationsRequest {
    pub content_types: Option<Vec<ContentType>>,
    pub pagination: PaginationParams,
    pub include_followed_users: Option<bool>,
    pub include_followed_topics: Option<bool>,
    pub include_trending: Option<bool>,
    pub include_similar_to_liked: Option<bool>,
    pub diversity_factor: Option<f64>,
    pub recency_weight: Option<f64>
}

// Response DTOs
#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct SearchResultResponse {
    pub id: String,
    pub content_type: ContentType,
    pub title: Option<String>,
    pub snippet: String,
    pub author: crate::models::user::UserSocialResponse,
    pub created_at: u64,
    pub relevance_score: f64
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct TrendingTopicResponse {
    pub topic: String,
    pub count: u64,
    pub trend_direction: TrendDirection
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct HotTagsResponse {
    pub tags: Vec<HotTagInfo>,
    pub updated_at: u64
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct HotTagInfo {
    pub name: String,
    pub count: u64,
    pub tag_type: TagType
}

// Enums
#[derive(CandidType, Deserialize, Clone, PartialEq, Debug)]
pub enum TrendDirection {
    New,
    Stable,
    Rising,
    Falling
}
