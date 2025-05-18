use candid::{CandidType, Deserialize, Principal};

// News reference response for returning news references in responses
#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct NewsReference {
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
    pub news_reference: Option<NewsReference>,  // Optional reference to news
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
    pub news_reference: Option<NewsReference>,  // Optional reference to news
}

#[derive(CandidType, Deserialize, Clone)]
pub struct UpdateCommentRequest {
    pub id: String,
    pub content: String,
}

// Content types
#[derive(CandidType, Deserialize, Clone)]
pub struct Post {
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
    pub news_reference: Option<NewsReference>,
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
    pub news_reference: Option<NewsReference>,
}

#[derive(CandidType, Deserialize, Clone)]
pub struct Comment {
    pub id: String,
    pub author: Principal,
    pub content: String,
    pub parent_id: String, // ID of the post or comment this is replying to
    pub parent_type: ParentType,
    pub created_at: u64,
    pub updated_at: u64,
    pub status: ContentStatus,
    pub child_comments: Vec<String>, // IDs of child comments
    pub likes_count: u64,
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

impl From<Comment> for CommentResponse {
    fn from(comment: Comment) -> Self {
        CommentResponse {
            id: comment.id,
            author: comment.author,
            content: comment.content,
            parent_id: comment.parent_id,
            parent_type: comment.parent_type,
            created_at: comment.created_at,
            updated_at: comment.updated_at,
            status: comment.status,
            likes_count: comment.likes_count,
            comments_count: comment.child_comments.len() as u64,
            visibility: ContentVisibility::Public, // Default to public
            child_comments: Vec::new(), // Child comments need to be populated separately
            author_info: crate::models::user::UserSocialResponse::default(), // Need to be populated separately
            is_liked: false, // Need to be populated separately
        }
    }
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
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

// Content moderation
#[derive(CandidType, Deserialize, Clone)]
pub struct ContentModerationRequest {
    pub content_id: String,
    pub content_type: ContentType,
    pub status: ContentStatus,
    pub reason: String,
}

#[derive(CandidType, Deserialize, Clone, PartialEq, Debug)]
pub enum ContentStatus {
    Active,
    UnderReview,
    Removed,
    Hidden,
    Deleted,
}

#[derive(CandidType, Deserialize, Clone, PartialEq, Debug)]
pub enum ContentVisibility {
    Public,
    FollowersOnly,
    Private,
}

#[derive(CandidType, Deserialize, Clone, PartialEq, Copy, Debug)]
pub enum ParentType {
    Post,
    Comment,
}

// Alias for ModerateContentRequest to match the API function name
pub type ModerateContentRequest = ContentModerationRequest;

#[derive(CandidType, Deserialize, Clone, PartialEq, Eq, Debug)]
pub enum ContentType {
    Post,
    Comment,
}

// Enums
#[derive(CandidType, Deserialize, Clone, PartialEq)]
pub enum MarketSentiment {
    Bullish,
    Bearish,
    Neutral,
}


// Content constants
pub const MAX_POST_LENGTH: usize = 2100;
pub const MAX_COMMENT_LENGTH: usize = 1000;
pub const MAX_TITLE_LENGTH: usize = 200;
pub const MAX_HASHTAGS: usize = 10;
pub const MAX_TOKEN_MENTIONS: usize = 10;
pub const MAX_MEDIA_URLS: usize = 5;
