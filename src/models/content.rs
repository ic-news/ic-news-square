use candid::{CandidType, Deserialize, Principal};
use crate::storage::{ContentStatus, ContentVisibility, ParentType};

// News reference for referencing news in posts
#[derive(CandidType, Deserialize, Clone)]
pub struct NewsReferenceRequest {
    pub metadata: Vec<(String, String)>,
    pub canister_id: Principal,
}

// News reference response for returning news references in responses
#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct NewsReferenceResponse {
    pub metadata: Vec<(String, String)>,
    pub canister_id: Principal,
}
#[derive(CandidType, Deserialize, Clone)]
pub struct CreatePostRequest {
    pub id: Option<String>,
    pub content: String,
    pub media_urls: Vec<String>,
    pub hashtags: Vec<String>,
    pub mentions: Option<Vec<String>>,
    pub token_mentions: Option<Vec<String>>,
    pub tags: Option<Vec<String>>,  // Max 5 tags
    pub is_nsfw: Option<bool>,
    pub visibility: Option<ContentVisibility>,
    pub news_reference: Option<NewsReferenceRequest>,  // Optional reference to news
}

#[derive(CandidType, Deserialize, Clone)]
pub struct CreateCommentRequest {
    pub id: Option<String>,
    pub content: String,
    pub parent_id: String,
    pub parent_type: ParentType,
}

#[derive(CandidType, Deserialize, Clone)]
pub struct UpdatePostRequest {
    pub id: String,
    pub content: String,
    pub media_urls: Option<Vec<String>>,
    pub hashtags: Option<Vec<String>>,
    pub token_mentions: Option<Vec<String>>,
    pub tags: Option<Vec<String>>,  // Max 5 tags
    pub visibility: Option<ContentVisibility>,
    pub news_reference: Option<NewsReferenceRequest>,  // Optional reference to news
}

#[derive(CandidType, Deserialize, Clone)]
pub struct UpdateCommentRequest {
    pub id: String,
    pub content: String,
}

// Response DTOs
#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct PostResponse {
    pub id: String,
    pub author: Principal,
    pub content: String,
    pub media_urls: Vec<String>,
    pub hashtags: Vec<String>,
    pub token_mentions: Vec<String>,
    pub tags: Vec<String>,  // Max 5 tags
    pub created_at: u64,
    pub updated_at: u64,
    pub status: ContentStatus,
    pub visibility: ContentVisibility,
    pub likes_count: u64,
    pub comments_count: u64,
    pub author_info: crate::models::user::UserSocialResponse,
    pub news_reference: Option<NewsReferenceResponse>,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct CommentResponse {
    pub id: String,
    pub author: Principal,
    pub content: String,
    pub parent_id: String,
    pub parent_type: ParentType,
    pub created_at: u64,
    pub updated_at: u64,
    pub status: ContentStatus,
    pub likes_count: u64,
    pub comments_count: u64,
    pub visibility: ContentVisibility,
    pub child_comments: Vec<Box<CommentResponse>>,
    pub author_info: crate::models::user::UserSocialResponse,
    pub is_liked: bool,
}

#[derive(CandidType, Deserialize, Clone)]
pub struct CommentsResponse {
    pub comments: Vec<CommentResponse>,
    pub total: u64,
    pub has_more: bool,
    pub next_offset: usize,
}

#[derive(CandidType, Deserialize, Clone)]
pub struct PostsResponse {
    pub posts: Vec<PostResponse>,
    pub total: u64,
    pub next_offset: usize,
}

// Query parameters
#[derive(CandidType, Deserialize, Clone)]
pub struct ContentFilter {
    pub author: Option<Principal>,
    pub hashtag: Option<String>,
    pub token_mention: Option<String>,
    pub created_after: Option<u64>,
    pub created_before: Option<u64>,
}

#[derive(CandidType, Deserialize, Clone)]
pub struct PaginationParams {
    pub offset: usize,
    pub limit: usize,
}

// Content moderation
#[derive(CandidType, Deserialize, Clone)]
pub struct ContentModerationRequest {
    pub content_id: String,
    pub content_type: ContentType,
    pub status: ContentStatus,
    pub reason: String,
}

// Alias for ModerateContentRequest to match the API function name
pub type ModerateContentRequest = ContentModerationRequest;

#[derive(CandidType, Deserialize, Clone, PartialEq, Eq, Debug)]
pub enum ContentType {
    Post,
    Comment,
}

// Content constants
pub const MAX_POST_LENGTH: usize = 2100;
pub const MAX_COMMENT_LENGTH: usize = 1000;
pub const MAX_TITLE_LENGTH: usize = 200;
pub const MAX_HASHTAGS: usize = 10;
pub const MAX_TOKEN_MENTIONS: usize = 10;
pub const MAX_MEDIA_URLS: usize = 5;
