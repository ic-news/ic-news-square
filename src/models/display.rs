use candid::{CandidType, Deserialize};
use crate::models::content::{PostResponse, CommentResponse};
use crate::models::user::UserSocialResponse;
use crate::models::discovery::TrendingTopicResponse;

// Feed display models
#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct FeedResponse {
    pub posts: Vec<PostResponse>,
    pub comments: Vec<CommentResponse>,
    pub has_more: bool,
    pub next_offset: usize,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct UserFeedResponse {
    pub posts: Vec<PostResponse>,
    pub user: UserSocialResponse,
    pub has_more: bool,
    pub next_offset: usize,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct ContentDetailResponse {
    pub post: Option<PostResponse>,
    pub comments: Vec<CommentResponse>,
    pub has_more_comments: bool,
    pub next_comment_offset: usize,
}

// Dashboard display models
#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct DashboardResponse {
    pub trending_topics: Vec<TrendingTopicResponse>,
    pub trending_content: FeedResponse,
    pub followed_users_content: FeedResponse,
    pub personalized_recommendations: FeedResponse,
}

// Creator center display models
#[derive(CandidType, Deserialize, Clone)]
pub struct CreatorCenterResponse {
    pub recent_posts: Vec<PostResponse>,
    pub content_stats: ContentStatsResponse,
    pub available_tasks: Vec<CreatorTaskResponse>,
}

#[derive(CandidType, Deserialize, Clone)]
pub struct ContentStatsResponse {
    pub total_posts: u64,
    pub total_comments: u64,
    pub total_likes_received: u64,
    pub total_views: u64,
    pub engagement_rate: f64,
}

#[derive(CandidType, Deserialize, Clone)]
pub struct CreatorTaskResponse {
    pub id: String,
    pub title: String,
    pub description: String,
    pub points_reward: u64,
    pub deadline: Option<u64>,
    pub is_completed: bool,
}

// Notification display models
#[derive(CandidType, Deserialize, Clone)]
pub struct NotificationResponse {
    pub id: String,
    pub notification_type: NotificationType,
    pub actor: Option<UserSocialResponse>,
    pub content_id: Option<String>,
    pub content_snippet: Option<String>,
    pub created_at: u64,
    pub is_read: bool,
}

#[derive(CandidType, Deserialize, Clone, PartialEq)]
pub enum NotificationType {
    Like,
    Comment,
    Follow,
    Mention,
    TaskCompleted,
    RewardEarned,
    ContentFeatured,
    SystemAnnouncement,
}

// Pagination wrapper
#[derive(CandidType, Deserialize, Clone)]
pub struct PaginatedResponse<T> {
    pub items: Vec<T>,
    pub total: usize,
    pub has_more: bool,
    pub next_offset: usize,
}
