use candid::Principal;
use ic_cdk::api::caller;
use ic_cdk_macros::*;

// Import modules
mod auth;
mod models;
mod services;
mod storage;
mod storage_main;
mod utils;

// Import specific types
use models::content::{ContentType, PaginationParams, CreatePostRequest, UpdatePostRequest, PostResponse, PostsResponse, CreateArticleRequest, UpdateArticleRequest, ArticleResponse, CreateCommentRequest, UpdateCommentRequest, CommentResponse, CommentsResponse, ModerateContentRequest};
use models::user::{RegisterUserRequest, UpdateProfileRequest, UserProfileResponse, UserSocialResponse, UserLeaderboardResponse, FollowUserRequest};
use models::interaction::{LikeContentRequest, LikesResponse, SharesResponse, ReportContentRequest};
use models::reward::{CompleteTaskRequest, TaskCompletionResponse, TaskResponse, AwardPointsRequest, CreateTaskRequest, UpdateTaskRequest, UserRewardsResponse, Value};
use models::discovery::{DiscoverContentRequest, SearchRequest, SearchResultResponse, GetTrendingTopicsRequest, TrendingTopicResponse, PersonalizedRecommendationsRequest};
use models::display::{FeedResponse};
use models::error::{SquareError, SquareResult, ErrorCode};
use models::tag::{};
use models::cycles::{CyclesBalanceResponse, CyclesConsumptionResponse, UpdateCyclesThresholdRequest, CyclesThresholdConfig, CyclesNotificationsResponse, NotificationSettings};
use utils::middleware::{ApiResponse, with_error_handling};

use models::discovery::GetHotTagsRequest as DiscoveryGetHotTagsRequest;
use models::discovery::HotTagsResponse as DiscoveryHotTagsResponse;

// Initialize the canister
#[init]
fn init() {
    // Register global error handler
    utils::error_interceptor::register_global_error_handler();
    
    auth::init_admin();
    services::cycles::init_cycles_monitoring();
    
    // Initialize sharded storage (will be empty for new deployments)
    storage::migration::migrate_all();
    
    // Initialize default tasks
    services::reward::init_default_tasks_all_enabled();
}

// User API
#[update]
fn register_user(request: RegisterUserRequest) -> ApiResponse<()> {
    with_error_handling(|| services::user::register_user(request, caller()))()
}

#[update]
fn update_user_profile(request: UpdateProfileRequest) -> ApiResponse<()> {
    with_error_handling(|| services::user::update_user_profile(request, caller()))()
}

#[query]
fn get_user_profile(principal: Option<Principal>) -> ApiResponse<UserProfileResponse> {
    with_error_handling(|| {
        let target = auth::get_target_or_caller(principal)?;
        services::user::get_user_profile(target)
    })()
}

#[update]
fn follow_user(principal: Principal) -> ApiResponse<()> {
    with_error_handling(|| {
        let request = FollowUserRequest {
            user_to_follow: principal
        };
        services::user::follow_user(request, caller())
    })()
}

#[update]
fn unfollow_user(principal: Principal) -> ApiResponse<()> {
    with_error_handling(|| {
        let request = FollowUserRequest {
            user_to_follow: principal
        };
        services::user::unfollow_user(request, caller())
    })()
}

#[query]
fn get_followers(principal: Option<Principal>) -> ApiResponse<Vec<UserSocialResponse>> {
    with_error_handling(|| {
        let current_caller = auth::get_authenticated_caller()?;
        let target = principal.unwrap_or(current_caller);
        services::user::get_followers(target, Some(current_caller))
    })()
}

#[query]
fn get_following(principal: Option<Principal>) -> ApiResponse<Vec<UserSocialResponse>> {
    with_error_handling(|| {
        let current_caller = auth::get_authenticated_caller()?;
        let target = principal.unwrap_or(current_caller);
        services::user::get_following(target, Some(current_caller))
    })()
}

#[query]
fn get_user_leaderboard(pagination: PaginationParams) -> ApiResponse<UserLeaderboardResponse> {
    with_error_handling(|| {
        // Only verify caller is not anonymous
        auth::get_authenticated_caller()?;
        services::user::get_user_leaderboard(pagination)
    })()
}

// Content API
#[update]
fn create_post(request: CreatePostRequest) -> ApiResponse<PostResponse> {
    with_error_handling(|| services::content::create_post(request, caller()))()
}

#[update]
fn update_post(request: UpdatePostRequest) -> ApiResponse<PostResponse> {
    with_error_handling(|| services::content::update_post(request, caller()))()
}

#[query]
fn get_post(post_id: String) -> SquareResult<PostResponse> {
    services::content::get_post(post_id)
}

#[query]
fn get_posts(pagination: PaginationParams) -> SquareResult<PostsResponse> {
    services::content::get_posts(pagination)
}

#[update]
fn delete_post(post_id: String) -> SquareResult<()> {
    services::content::delete_post(post_id, caller())
}

#[update]
fn create_article(request: CreateArticleRequest) -> SquareResult<ArticleResponse> {
    services::content::create_article(request, caller())
}

#[update]
fn update_article(request: UpdateArticleRequest) -> SquareResult<ArticleResponse> {
    services::content::update_article(request, caller())
}

#[query]
fn get_article(article_id: String) -> SquareResult<ArticleResponse> {
    services::content::get_article(article_id)
}

#[update]
fn delete_article(article_id: String) -> SquareResult<()> {
    services::content::delete_article(article_id, caller())
}

#[update]
fn create_comment(request: CreateCommentRequest) -> SquareResult<CommentResponse> {
    services::content::create_comment(request, caller())
}

#[update]
fn update_comment(request: UpdateCommentRequest) -> SquareResult<CommentResponse> {
    services::content::update_comment(request, caller())
}

#[query]
fn get_comment(comment_id: String) -> SquareResult<CommentResponse> {
    services::content::get_comment(comment_id, Some(caller()))
}

#[update]
fn delete_comment(comment_id: String) -> SquareResult<()> {
    services::content::delete_comment(comment_id, caller())
}

#[query]
fn get_comments(parent_id: String, parent_type: String, pagination: PaginationParams) -> SquareResult<CommentsResponse> {
    services::content::get_comments(parent_id, parent_type, pagination, Some(caller()))
}

#[query]
fn get_user_content(principal: Option<Principal>, content_type: Option<ContentType>, pagination: PaginationParams) -> SquareResult<FeedResponse> {
    let target = principal.unwrap_or_else(caller);
    services::content::get_user_content(target, content_type, pagination)
}

// Interaction API
#[update]
fn like_content(request: LikeContentRequest) -> SquareResult<()> {
    services::interaction::like_content(request.content_id, request.content_type, caller())
}

#[update]
fn unlike_content(request: LikeContentRequest) -> SquareResult<()> {
    services::interaction::unlike_content(request.content_id, request.content_type, caller())
}

#[query]
fn get_likes(content_id: String, content_type: ContentType) -> SquareResult<LikesResponse> {
    services::interaction::get_likes(content_id, content_type)
}

#[update]
fn share_content(content_id: String, content_type: ContentType) -> SquareResult<()> {
    services::interaction::share_content(content_id, content_type, caller())
}

#[query]
fn get_shares(content_id: String, content_type: ContentType) -> SquareResult<SharesResponse> {
    services::interaction::get_shares(content_id, content_type)
}

#[update]
fn report_content(request: ReportContentRequest) -> SquareResult<()> {
    services::interaction::report_content(request, caller())
}

// Discovery API
#[query]
fn discover_content(request: DiscoverContentRequest) -> SquareResult<FeedResponse> {
    services::discovery::discover_content(request)
}

#[query]
fn search_content(request: SearchRequest) -> SquareResult<Vec<SearchResultResponse>> {
    services::discovery::search_content(request)
}

#[query]
fn get_trending_topics(request: GetTrendingTopicsRequest) -> SquareResult<Vec<TrendingTopicResponse>> {
    services::discovery::get_trending_topics(request)
}

#[query]
fn get_hot_tags(request: DiscoveryGetHotTagsRequest) -> SquareResult<DiscoveryHotTagsResponse> {
    services::discovery::get_hot_tags(request)
}

#[query]
fn get_personalized_recommendations(request: PersonalizedRecommendationsRequest) -> SquareResult<FeedResponse> {
    services::discovery::get_personalized_recommendations(request)
}

// Rewards API
// Note: claim_daily_check_in has been moved to the daily_checkin_task canister
// Users should call that canister directly for check-ins

#[update(name = "complete_task")]
fn complete_task_async(request: CompleteTaskRequest) -> SquareResult<TaskCompletionResponse> {
    services::reward::complete_task(request, caller())
}

#[query]
fn get_user_rewards() -> SquareResult<UserRewardsResponse> {
    services::reward::get_user_rewards(caller())
}

#[query]
fn get_available_tasks() -> SquareResult<Vec<TaskResponse>> {
    services::reward::get_available_tasks(caller())
}

#[update]
fn moderate_content(request: ModerateContentRequest) -> SquareResult<()> {
    auth::is_manager_or_admin()?;
    services::content::moderate_content(request)
}

#[update]
fn award_points(request: AwardPointsRequest) -> SquareResult<()> {
    services::reward::award_points(request)
}

#[update(name = "create_task")]
fn create_task(request: CreateTaskRequest) -> SquareResult<String> {
    services::reward::create_task(request)
}

#[update]
fn update_task(request: UpdateTaskRequest) -> SquareResult<()> {
    services::reward::update_task(request)
}

#[update]
fn delete_task(task_id: String) -> SquareResult<()> {
    services::reward::delete_task(task_id)
}

// System functions
#[heartbeat]
fn heartbeat() {
    // Use time-based throttling to reduce execution frequency
    let current_time = ic_cdk::api::time() / 1_000_000;
    
    // Store last execution time in thread-local storage
    thread_local! {
        static LAST_HEARTBEAT_TIME: std::cell::RefCell<u64> = std::cell::RefCell::new(0);
    }
    
    // Check if enough time has passed since last full execution (every 6 hours)
    let should_run_full = LAST_HEARTBEAT_TIME.with(|last_time| {
        let last = *last_time.borrow();
        let six_hours = 6 * 60 * 60; // 6 hours in seconds
        
        if current_time - last >= six_hours {
            *last_time.borrow_mut() = current_time;
            true
        } else {
            false
        }
    });
    
    if should_run_full {
        // Only update trending content on full runs (expensive operation)
        let _ = services::discovery::update_trending_content();
        
        // Initialize default tasks if they don't exist
        services::reward::init_default_tasks_all_enabled();
        
    }
    
    // Always record cycles consumption (lightweight operation)
    services::cycles::record_cycles_consumption();
}

// Cycles Monitoring API
#[query]
fn get_cycles_balance() -> SquareResult<CyclesBalanceResponse> {
    services::cycles::get_cycles_balance()
}

// Storage Management API
#[update]
fn migrate_to_sharded_storage() -> SquareResult<String> {
    // Check if caller is admin
    auth::is_admin()?;
    
    // Trigger migration
    storage::migration::migrate_all();
    
    Ok("Migration to sharded storage completed successfully".to_string())
}

#[query]
fn get_cycles_consumption_history() -> SquareResult<CyclesConsumptionResponse> {
    // Only authenticated users can view consumption history
    auth::get_authenticated_caller()?;
    services::cycles::get_cycles_consumption_history()
}

#[update]
fn update_cycles_threshold(request: UpdateCyclesThresholdRequest) -> SquareResult<()> {
    // Only admin can update threshold configuration
    services::cycles::update_cycles_threshold(request, caller())
}

#[query]
fn get_cycles_threshold() -> SquareResult<CyclesThresholdConfig> {
    // Only admin can view threshold configuration
    let _caller_principal = auth::get_authenticated_caller()?;
    match auth::is_admin() {
        Err(msg) => return Err(SquareError::Unauthorized(msg)),
        Ok(_) => {}
    }
    services::cycles::get_cycles_threshold()
}

#[query]
fn get_cycles_notifications() -> SquareResult<CyclesNotificationsResponse> {
    // Only admin can view notifications
    let _caller_principal = auth::get_authenticated_caller()?;
    match auth::is_admin() {
        Err(msg) => return Err(SquareError::Unauthorized(msg)),
        Ok(_) => {}
    }
    services::cycles::get_cycles_notifications()
}

#[update]
fn acknowledge_notification(timestamp: u64) -> SquareResult<()> {
    // Only admin can acknowledge notifications
    services::cycles::acknowledge_notification(timestamp, caller())
}

#[update]
fn update_notification_settings(email_enabled: Option<bool>, email_address: Option<String>, notification_frequency_hours: Option<u64>) -> SquareResult<()> {
    // Only admin can update notification settings
    services::cycles::update_notification_settings(email_enabled, email_address, notification_frequency_hours, caller())
}

#[query]
fn get_notification_settings() -> SquareResult<NotificationSettings> {
    // Only admin can view notification settings
    services::cycles::get_notification_settings(caller())
}

// Error Monitoring API
#[query]
fn get_error_history() -> ApiResponse<Vec<String>> {
    with_error_handling(|| {
        Ok(utils::error_monitor::get_error_history())
    })()
}

#[query]
fn get_error_stats() -> ApiResponse<Vec<(ErrorCode, u64, u64, u64)>> {
    with_error_handling(|| {
        Ok(utils::error_monitor::get_error_stats())
    })()
}

#[query]
fn get_most_common_errors(limit: usize) -> ApiResponse<Vec<(ErrorCode, u64)>> {
    with_error_handling(|| {
        Ok(utils::error_monitor::get_most_common_errors(limit))
    })()
}

// State management for canister upgrades
#[pre_upgrade]
fn pre_upgrade() {
    ic_cdk::println!("Starting pre-upgrade hook");
    storage::migration::save_state_for_upgrade();
    ic_cdk::println!("Pre-upgrade hook completed");
}

#[post_upgrade]
fn post_upgrade() {
    ic_cdk::println!("Starting post-upgrade hook");
    utils::error_interceptor::register_global_error_handler();
    
    // Restore state
    storage::migration::restore_state_after_upgrade();
    
    // Initialize default tasks if needed
    services::reward::init_default_tasks_if_empty();
    
    // Initialize admin if needed
    auth::init_admin_if_empty();
    
    // Initialize cycles monitoring
    services::cycles::init_cycles_monitoring();
    
    ic_cdk::println!("Post-upgrade hook completed");
}

ic_cdk::export_candid!();
