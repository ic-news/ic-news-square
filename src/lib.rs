use candid::Principal;
use ic_cdk::api::{caller, time, performance_counter, canister_balance};
use ic_cdk_macros::*;
use ic_cdk::api::stable::{stable_size, stable_grow, stable_write, stable_read};

// Import modules
mod auth;
mod models;
mod services;
mod storage;
mod utils;

// Import specific types
use models::content::{ContentType, PaginationParams, CreatePostRequest, UpdatePostRequest, PostResponse, PostsResponse, CreateCommentRequest, UpdateCommentRequest, CommentResponse, CommentsResponse, ModerateContentRequest};
use models::user::{RegisterUserRequest, UpdateProfileRequest, UserProfileResponse, UserSocialResponse, UserLeaderboardResponse, FollowUserRequest};
use models::interaction::{LikeContentRequest, LikesResponse, ReportContentRequest};
use models::reward::{CompleteTaskRequest, TaskCompletionResponse, TaskResponse, AwardPointsRequest, CreateTaskRequest, UpdateTaskRequest, UserRewardsResponse, Value};
use models::discovery::{DiscoverContentRequest, SearchRequest, SearchResultResponse, GetTrendingTopicsRequest, TrendingTopicResponse, PersonalizedRecommendationsRequest};
use models::display::{FeedResponse};
use models::error::{SquareError, SquareResult, ErrorCode};
use models::tag::{};
use models::cycles::{CyclesBalanceResponse, CyclesConsumptionResponse, UpdateCyclesThresholdRequest, CyclesThresholdConfig, CyclesNotificationsResponse, NotificationSettings, UpdateHeartbeatIntervalRequest, HeartbeatIntervalResponse};
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

    
    // Initialize default tasks
    services::reward::init_default_tasks_all_enabled();
}

// User API
#[update]
fn register_user(request: RegisterUserRequest) -> ApiResponse<()> {
    with_error_handling(|| services::user::register_user(request, caller()))()
}

#[update]
fn update_user_profile(request: UpdateProfileRequest) -> ApiResponse<String> {
    with_error_handling(|| services::user::update_user_profile(request, caller()))()
}

#[query]
fn get_user_profile(user_identifier: Option<String>) -> ApiResponse<UserProfileResponse> {
    with_error_handling(|| {
        let identifier = match user_identifier {
            Some(id) => id,
            None => caller().to_text()
        };
        services::user::get_user_profile(identifier)
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
fn get_followers(user_identifier: Option<String>) -> ApiResponse<Vec<UserSocialResponse>> {
    with_error_handling(|| {
        let current_caller = auth::get_authenticated_caller()?;
        let identifier = match user_identifier {
            Some(id) => id,
            None => caller().to_text()
        };
        services::user::get_followers(identifier, Some(current_caller))
    })()
}

#[query]
fn get_following(user_identifier: Option<String>) -> ApiResponse<Vec<UserSocialResponse>> {
    with_error_handling(|| {
        let current_caller = auth::get_authenticated_caller()?;
        let identifier = match user_identifier {
            Some(id) => id,
            None => caller().to_text()
        };
        services::user::get_following(identifier, Some(current_caller))
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
fn get_user_content(user_identifier: Option<String>, content_type: Option<ContentType>, pagination: PaginationParams) -> SquareResult<FeedResponse> {
    let identifier = match user_identifier {
        Some(id) => id,
        None => caller().to_text()
    };
    services::content::get_user_content(identifier, content_type, crate::models::content::PaginationParams { offset: pagination.offset, limit: pagination.limit })
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
    let current_time = time() / 1_000_000;
    
    // Store last execution time in thread-local storage
    thread_local! {
        static LAST_HEARTBEAT_TIME: std::cell::RefCell<u64> = std::cell::RefCell::new(0);
    }
    
    // Check if enough time has passed since last full execution
    let should_run_full = LAST_HEARTBEAT_TIME.with(|last_time| {
        let last = *last_time.borrow();
        
        // Get configured heartbeat interval from main storage
        let interval_hours = storage::STORAGE.with(|storage| {
            let store = storage.borrow();
            store.heartbeat_interval_hours
        });
        
        let interval_seconds = interval_hours * 60 * 60; // Convert hours to seconds
        
        if current_time - last >= interval_seconds {
            *last_time.borrow_mut() = current_time;
            true
        } else {
            false
        }
    });
    
    if should_run_full {
        // Only update trending content on full runs (expensive operation)
        let _ = services::discovery::update_trending_content(Vec::new());
        
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

// Admin API - Heartbeat Configuration
#[update]
fn update_heartbeat_interval(request: UpdateHeartbeatIntervalRequest) -> ApiResponse<HeartbeatIntervalResponse> {
    with_error_handling(|| {
        services::admin::update_heartbeat_interval(request)
    })()
}

#[query]
fn get_heartbeat_interval() -> ApiResponse<HeartbeatIntervalResponse> {
    with_error_handling(|| {
        services::admin::get_heartbeat_interval()
    })()
}

// Admin API - Storage Management
#[update]
fn migrate_storage() -> ApiResponse<String> {
    with_error_handling(|| {
        auth::require_admin()?;
        services::admin::migrate_storage()
    })()
}

#[query]
fn get_cycles_consumption_history() -> SquareResult<CyclesConsumptionResponse> {
    // Only authenticated users can view consumption history
    auth::get_authenticated_caller()?;
    services::cycles::get_cycles_consumption_history()
}

#[update]
fn update_cycles_threshold(request: UpdateCyclesThresholdRequest) -> ApiResponse<CyclesThresholdConfig> {
    with_error_handling(|| {
        services::admin::update_cycles_threshold(request)
    })()
}

#[query]
fn get_cycles_threshold() -> ApiResponse<CyclesThresholdConfig> {
    with_error_handling(|| {
        services::admin::get_cycles_threshold()
    })()
}

#[query]
fn get_cycles_notifications() -> ApiResponse<CyclesNotificationsResponse> {
    with_error_handling(|| {
        services::admin::get_cycles_notifications()
    })()
}

#[update]
fn acknowledge_notification(timestamp: u64) -> SquareResult<()> {
    // Only admin can acknowledge notifications
    services::cycles::acknowledge_notification(timestamp, caller())
}

#[update]
fn update_notification_settings(enabled: Option<bool>) -> SquareResult<()> {
    // Only admin can update notification settings
    services::cycles::update_notification_settings(enabled, caller())
}

#[query]
fn get_notification_settings() -> SquareResult<bool> {
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
    utils::logger::log("========== STARTING PRE-UPGRADE HOOK ==========");
    
    let memory_usage = performance_counter(0);
    utils::logger::log(&format!("Memory usage before upgrade: {} bytes", memory_usage));
    
    let stable_size_before = stable_size();
    utils::logger::log(&format!("Current stable storage size: {} pages ({} bytes)", 
        stable_size_before, 
        stable_size_before * 65536));
    
    let heap_size = canister_balance();
    utils::logger::log(&format!("Current heap memory usage: {} bytes", heap_size));
    
    utils::logger::log("Preparing main storage for upgrade...");
    storage::migration::synchronize_storage_before_upgrade();
    
    storage::STORAGE.with(|storage| {
        let store = storage.borrow();
        utils::logger::log(&format!("Main storage users count: {}", store.users.len()));
        utils::logger::log(&format!("Main storage posts count: {}", store.posts.len()));
        utils::logger::log(&format!("Main storage comments count: {}", store.comments.len()));
        utils::logger::log(&format!("Main storage user rewards count: {}", store.user_rewards.len()));
    });
    
    utils::logger::log("Saving main storage data...");
    storage::migration::save_state_for_upgrade();
    
    let stable_size_after_data = stable_size();
    utils::logger::log(&format!("Stable storage size after saving data: {} pages ({} bytes)", 
        stable_size_after_data, 
        stable_size_after_data * 65536));
    utils::logger::log(&format!("Data size: {} bytes", 
        (stable_size_after_data - stable_size_before) * 65536));
    
    // Main storage is the only storage mechanism used
    utils::logger::log("Using main storage for all data");
    
    // Log main storage statistics
    utils::logger::log("Main storage statistics:");
    storage::STORAGE.with(|storage| {
        let store = storage.borrow();
        utils::logger::log(&format!("- Users: {}", store.users.len()));
    });
    
    storage::STORAGE.with(|storage| {
        let store = storage.borrow();
        
        // Log counts from main storage
        utils::logger::log(&format!("- User profiles: {}", store.user_profiles.as_ref().map_or(0, |profiles| profiles.len())));
        utils::logger::log(&format!("- User stats: {}", store.user_stats.as_ref().map_or(0, |stats| stats.len())));
        utils::logger::log(&format!("- Posts: {}", store.posts.len()));
        utils::logger::log(&format!("- Comments: {}", store.comments.len()));
        utils::logger::log(&format!("- Likes: {}", store.likes.len()));
    });
    
    // Continue logging main storage statistics
    storage::STORAGE.with(|storage| {
        let store = storage.borrow();
        utils::logger::log(&format!("- User rewards: {}", store.user_rewards.len()));
        utils::logger::log(&format!("- User tasks: {}", store.user_tasks.len()));
    });
    
    // Main storage is now the primary storage
    utils::logger::log("✅ Successfully saved main storage to stable storage");
    
    /*
    storage::STORAGE.with(|storage| {
        let storage_ref = storage.borrow();
        match ic_cdk::storage::stable_save((storage_ref.clone(),)) {
            Ok(_) => utils::logger::log("✅ Successfully saved main storage to stable storage"),
            Err(e) => utils::logger::log(&format!("❌ ERROR saving main storage: {:?}", e)),
        }
    });
    */
    
    let stable_size_after_main = stable_size();
    utils::logger::log(&format!("Stable storage size after saving main storage: {} pages ({} bytes)", 
        stable_size_after_main, 
        stable_size_after_main * 65536));
    utils::logger::log(&format!("Main storage data size: {} bytes", 
        (stable_size_after_main - stable_size_after_data) * 65536));
    
    utils::logger::log("Setting backup flag...");
    
    let current_pages = stable_size();
    let max_pages = 4294967295;
    
    utils::logger::log(&format!("Current stable pages: {}, Max pages: {}", current_pages, max_pages));
    
    if current_pages < max_pages {
        match stable_grow(1) {
            Ok(new_pages) => {
                if new_pages > 0 {
                    let backup_flag: u64 = 0x1234567890ABCDEF;
                    let offset = (stable_size() - 1) * 65536;
                    stable_write(offset, &backup_flag.to_le_bytes());
                    utils::logger::log(&format!("✅ Set backup flag at offset {}", offset));
                } else {
                    utils::logger::log("⚠️ WARNING: Could not grow stable memory, but no error returned");
                    
                    if current_pages > 0 {
                        let backup_flag: u64 = 0x1234567890ABCDEF;
                        let offset = (current_pages * 65536) - 8; 
                        stable_write(offset, &backup_flag.to_le_bytes());
                        utils::logger::log(&format!("Wrote backup flag to existing page at offset {}", offset));
                    }
                }
            },
            Err(e) => {
                utils::logger::log(&format!("ERROR: Failed to grow stable memory: {:?}", e));
                
                if current_pages > 0 {
                    let backup_flag: u64 = 0x1234567890ABCDEF;
                    let offset = (current_pages * 65536) - 8; 
                    stable_write(offset, &backup_flag.to_le_bytes());
                    utils::logger::log(&format!("Wrote backup flag to existing page at offset {}", offset));
                }
            },
        }
    } else {
        utils::logger::log("WARNING: Stable storage at maximum capacity");
        
        if current_pages > 0 {
            let backup_flag: u64 = 0x1234567890ABCDEF;
            let offset = (current_pages * 65536) - 8; 
            stable_write(offset, &backup_flag.to_le_bytes());
            utils::logger::log(&format!("Wrote backup flag to existing page at offset {}", offset));
        }
    }
    
    let final_stable_size = stable_size();
    utils::logger::log(&format!("Final stable storage size: {} pages ({} bytes)", 
        final_stable_size, 
        final_stable_size * 65536));
    
    utils::logger::log("========== PRE-UPGRADE HOOK COMPLETED ==========");
}

#[post_upgrade]
fn post_upgrade() {
    utils::logger::log("========== STARTING POST-UPGRADE HOOK ==========");
    utils::error_interceptor::register_global_error_handler();
    
    let stable_size = stable_size();
    utils::logger::log(&format!("Current stable storage size: {} pages ({} bytes)", 
        stable_size, 
        stable_size * 65536));
    
    let mut has_valid_backup = false;
    let mut backup_flag_value = 0u64;
    
    if stable_size > 0 {
        let offset = (stable_size - 1) * 65536;
        let mut flag_bytes = [0u8; 8];
        stable_read(offset, &mut flag_bytes);
        backup_flag_value = u64::from_le_bytes(flag_bytes);
        has_valid_backup = backup_flag_value == 0x1234567890ABCDEF;
        utils::logger::log_fmt("Backup flag check at standard position: {}, value: {:?}", format!("{}, {:X}", has_valid_backup, backup_flag_value));
    }
    
    if !has_valid_backup && stable_size > 0 {
        let offset = (stable_size * 65536) - 8; 
        let mut flag_bytes = [0u8; 8];
        stable_read(offset, &mut flag_bytes);
        backup_flag_value = u64::from_le_bytes(flag_bytes);
        has_valid_backup = backup_flag_value == 0x1234567890ABCDEF;
        utils::logger::log_fmt("Backup flag check at alternate position: {}, value: {:?}", format!("{}, {:X}", has_valid_backup, backup_flag_value));
    }
    
    // Main storage is the only storage mechanism used
    
    let mut main_storage_restored = false;
    if has_valid_backup {
        utils::logger::log("✅ Valid backup flag found, attempting to restore main storage...");
        
        let storage_result: Result<(crate::models::storage::Storage,), String> = ic_cdk::storage::stable_restore();
        
        match storage_result {
            Ok((restored_storage,)) => {
                utils::logger::log("✅ Successfully restored main storage with:");
                utils::logger::log(&format!("- Users: {}", restored_storage.users.len()));
                utils::logger::log(&format!("- User profiles: {}", restored_storage.user_profiles.as_ref().map_or(0, |profiles| profiles.len())));
                utils::logger::log(&format!("- User stats: {}", restored_storage.user_stats.as_ref().map_or(0, |stats| stats.len())));
                utils::logger::log(&format!("- Posts: {}", restored_storage.posts.len()));
                utils::logger::log(&format!("- Comments: {}", restored_storage.comments.len()));
                utils::logger::log(&format!("- Likes: {}", restored_storage.likes.len()));
                utils::logger::log(&format!("- User rewards: {}", restored_storage.user_rewards.len()));
                utils::logger::log(&format!("- User tasks: {}", restored_storage.user_tasks.len()));
                
                // We still restore the main storage for now, but we'll eventually remove this
                storage::STORAGE.with(|storage| {
                    *storage.borrow_mut() = restored_storage;
                });
                main_storage_restored = true;
                
                utils::logger::log("Main storage restored successfully");
            },
            Err(e) => {
                utils::logger::log(&format!("ERROR restoring main storage: {:?}", e));
                utils::logger::log("This could be due to data structure changes or corruption");
                utils::logger::log("This is critical as main storage is the only storage mechanism");
            }
        }
    } else {
        utils::logger::log(&format!("⚠️ WARNING: No valid backup flag found. Expected: 0x1234567890ABCDEF, Found: {:X}", backup_flag_value));
        utils::logger::log("This could indicate that pre_upgrade did not complete successfully");
    }
    
    if !main_storage_restored {
        utils::logger::log("Restoring main storage...");
        
        storage::STORAGE.with(|storage| {
            let store = storage.borrow();
            utils::logger::log(&format!("Main storage users count before restoration: {}", store.users.len()));
            utils::logger::log(&format!("Main storage posts count before restoration: {}", store.posts.len()));
        });
        
        storage::migration::restore_state_after_upgrade();
        
        storage::STORAGE.with(|storage| {
            let store = storage.borrow();
            utils::logger::log(&format!("After restoration, main storage has:"));
            utils::logger::log(&format!("- Users: {}", store.users.len()));
            utils::logger::log(&format!("- User profiles: {}", store.user_profiles.as_ref().map_or(0, |profiles| profiles.len())));
            utils::logger::log(&format!("- User stats: {}", store.user_stats.as_ref().map_or(0, |stats| stats.len())));
            utils::logger::log(&format!("- Posts: {}", store.posts.len()));
            utils::logger::log(&format!("- Comments: {}", store.comments.len()));
            utils::logger::log(&format!("- Likes: {}", store.likes.len()));
            utils::logger::log(&format!("- User rewards: {}", store.user_rewards.len()));
            utils::logger::log(&format!("- User tasks: {}", store.user_tasks.len()));
        });
    }
    
    // Only main storage is used.
    
    let main_storage_empty = storage::STORAGE.with(|storage| {
        let store = storage.borrow();
        store.users.is_empty() && store.posts.is_empty()
    });
    
    utils::logger::log(&format!("Main storage state: empty={}", main_storage_empty));
    
    if main_storage_empty {
        utils::logger::log("⚠️ WARNING: Main storage is empty! This may indicate a problem with data loading.");
    } else {
        utils::logger::log("Main storage contains data, proceeding with normal operation.");
    }
    
    // Perform any necessary post-upgrade synchronization
    storage::migration::synchronize_storage_after_upgrade();
    
    storage::STORAGE.with(|storage| {
        let store = storage.borrow();
        utils::logger::log(&format!("After synchronization, main storage has:"));
        utils::logger::log(&format!("- Users: {}", store.users.len()));
        utils::logger::log(&format!("- Posts: {}", store.posts.len()));
        utils::logger::log(&format!("- Comments: {}", store.comments.len()));
        utils::logger::log(&format!("- User rewards: {}", store.user_rewards.len()));
        utils::logger::log(&format!("- User tasks: {}", store.user_tasks.len()));
    });
    
    // Synchronize user data to ensure consistency across all storage locations
    utils::logger::log("Synchronizing user data to ensure consistency...");
    match services::user::synchronize_all_user_data() {
        Ok(_) => utils::logger::log("User data synchronization completed successfully"),
        Err(e) => utils::logger::log(&format!("Error during user data synchronization: {:?}", e))
    }
    
    // Initialize default tasks if needed
    utils::logger::log("Initializing default tasks if needed...");
    services::reward::init_default_tasks_if_empty();
    
    // Initialize admin if needed
    utils::logger::log("Initializing admin if needed...");
    auth::init_admin_if_empty();
    
    // Initialize cycles monitoring
    utils::logger::log("Initializing cycles monitoring...");
    services::cycles::init_cycles_monitoring();
    
    utils::logger::log("========== POST-UPGRADE HOOK COMPLETED ==========");
}

// Logging API

#[query]
fn get_logs() -> Vec<utils::logger::LogEntry> {
    utils::logger::get_all_logs()
}

#[query]
fn get_recent_logs(count: usize) -> Vec<utils::logger::LogEntry> {
    utils::logger::get_recent_logs(count)
}

#[update]
fn clear_logs() -> bool {
    match auth::is_admin() {
        Ok(_) => {
            utils::logger::clear_logs();
            true
        },
        Err(_) => false
    }
}

// Debug API

#[query]
fn debug_list_all_users() -> ApiResponse<Vec<(String, String)>> {
    with_error_handling(|| services::user::debug_list_all_users())()
}

#[update]
fn debug_fix_user_data(principal_str: String) -> ApiResponse<bool> {
    with_error_handling(|| services::user::debug_fix_user_data(principal_str))()
}

#[update]
fn debug_fix_user_profile(principal_str: String) -> ApiResponse<String> {
    with_error_handling(|| services::user::debug_fix_user_profile(principal_str))()
}

ic_cdk::export_candid!();
