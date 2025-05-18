import type { Principal } from '@dfinity/principal';
import type { ActorMethod } from '@dfinity/agent';
import type { IDL } from '@dfinity/candid';

export interface ApiError {
  'recoverable' : boolean,
  'code' : number,
  'message' : string,
  'details' : [] | [string],
  'recovery_hint' : [] | [string],
}
export interface ApiResponse {
  'data' : [] | [PostResponse],
  'error' : [] | [ApiError],
  'success' : boolean,
}
export interface ApiResponse_1 {
  'data' : [] | [boolean],
  'error' : [] | [ApiError],
  'success' : boolean,
}
export interface ApiResponse_10 {
  'data' : [] | [HeartbeatIntervalResponse],
  'error' : [] | [ApiError],
  'success' : boolean,
}
export interface ApiResponse_11 {
  'data' : [] | [Array<[ErrorCode, bigint]>],
  'error' : [] | [ApiError],
  'success' : boolean,
}
export interface ApiResponse_12 {
  'data' : [] | [UserLeaderboardResponse],
  'error' : [] | [ApiError],
  'success' : boolean,
}
export interface ApiResponse_13 {
  'data' : [] | [UserProfileResponse],
  'error' : [] | [ApiError],
  'success' : boolean,
}
export interface ApiResponse_2 {
  'data' : [] | [string],
  'error' : [] | [ApiError],
  'success' : boolean,
}
export interface ApiResponse_3 {
  'data' : [] | [Array<[string, string]>],
  'error' : [] | [ApiError],
  'success' : boolean,
}
export interface ApiResponse_4 {
  'data' : [] | [null],
  'error' : [] | [ApiError],
  'success' : boolean,
}
export interface ApiResponse_5 {
  'data' : [] | [CyclesNotificationsResponse],
  'error' : [] | [ApiError],
  'success' : boolean,
}
export interface ApiResponse_6 {
  'data' : [] | [CyclesThresholdConfig],
  'error' : [] | [ApiError],
  'success' : boolean,
}
export interface ApiResponse_7 {
  'data' : [] | [Array<string>],
  'error' : [] | [ApiError],
  'success' : boolean,
}
export interface ApiResponse_8 {
  'data' : [] | [Array<[ErrorCode, bigint, bigint, bigint]>],
  'error' : [] | [ApiError],
  'success' : boolean,
}
export interface ApiResponse_9 {
  'data' : [] | [Array<UserSocialResponse>],
  'error' : [] | [ApiError],
  'success' : boolean,
}
export interface AwardPointsRequest {
  'principal' : Principal,
  'reference_id' : [] | [string],
  'points' : bigint,
  'reason' : string,
}
export interface CommentResponse {
  'id' : string,
  'status' : ContentStatus,
  'updated_at' : bigint,
  'content' : string,
  'child_comments' : Array<CommentResponse>,
  'author_info' : UserSocialResponse,
  'created_at' : bigint,
  'author' : Principal,
  'parent_id' : string,
  'is_liked' : boolean,
  'comments_count' : bigint,
  'visibility' : ContentVisibility_1,
  'likes_count' : bigint,
  'parent_type' : ParentType,
}
export interface CommentsResponse {
  'total' : bigint,
  'comments' : Array<CommentResponse>,
  'next_offset' : bigint,
  'has_more' : boolean,
}
export interface CompleteTaskRequest {
  'task_id' : string,
  'proof' : [] | [string],
}
export interface ContentCreationRequirement {
  'comment_count' : [] | [bigint],
  'post_count' : [] | [bigint],
  'required_hashtags' : [] | [Array<string>],
}
export interface ContentFilter {
  'hashtag' : [] | [string],
  'token_mention' : [] | [string],
  'created_after' : [] | [bigint],
  'author' : [] | [Principal],
  'created_before' : [] | [bigint],
}
export interface ContentModerationRequest {
  'status' : ContentStatus,
  'content_id' : string,
  'content_type' : ParentType,
  'reason' : string,
}
export type ContentStatus = { 'UnderReview' : null } |
  { 'Active' : null } |
  { 'Hidden' : null } |
  { 'Removed' : null } |
  { 'Deleted' : null };
export type ContentVisibility = { 'Private' : null } |
  { 'FollowersOnly' : null } |
  { 'Public' : null };
export type ContentVisibility_1 = { 'Private' : null } |
  { 'FollowersOnly' : null } |
  { 'Public' : null };
export interface CreateCommentRequest {
  'id' : [] | [string],
  'content' : string,
  'parent_id' : string,
  'parent_type' : ParentType,
}
export interface CreatePostRequest {
  'id' : [] | [string],
  'is_nsfw' : [] | [boolean],
  'content' : string,
  'hashtags' : Array<string>,
  'media_urls' : Array<string>,
  'tags' : [] | [Array<string>],
  'news_reference' : [] | [NewsReference],
  'token_mentions' : [] | [Array<string>],
  'mentions' : [] | [Array<string>],
  'visibility' : [] | [ContentVisibility],
}
export interface CreateTaskRequest {
  'id' : string,
  'title' : string,
  'points_reward' : bigint,
  'canister_id' : Principal,
  'description' : string,
  'end_time' : [] | [bigint],
  'completion_criteria' : string,
  'start_time' : [] | [bigint],
  'task_type' : TaskType,
  'requirements' : [] | [TaskRequirements],
}
export interface CyclesBalanceResponse {
  'estimated_days_remaining' : bigint,
  'threshold_warning' : boolean,
  'balance' : bigint,
  'balance_in_trillion' : number,
}
export interface CyclesConsumptionResponse {
  'daily_consumption' : Array<DailyConsumption>,
  'average_daily_consumption' : bigint,
  'total_consumed_last_week' : bigint,
}
export interface CyclesNotificationsResponse {
  'notifications' : Array<CyclesWarningNotification>,
  'unacknowledged_count' : bigint,
}
export interface CyclesThresholdConfig {
  'critical_threshold' : bigint,
  'warning_threshold' : bigint,
  'notification_enabled' : boolean,
}
export interface CyclesWarningNotification {
  'balance' : bigint,
  'threshold' : bigint,
  'message' : string,
  'timestamp' : bigint,
  'severity' : CyclesWarningSeverity,
  'is_acknowledged' : boolean,
}
export type CyclesWarningSeverity = { 'Critical' : null } |
  { 'Warning' : null };
export interface DailyConsumption {
  'date' : bigint,
  'operations' : bigint,
  'consumption' : bigint,
}
export interface DiscoverContentRequest {
  'sort_by' : [] | [SortOption],
  'pagination' : PaginationParams,
  'tags' : [] | [Array<string>],
  'filter' : [] | [ContentFilter],
  'content_types' : [] | [Array<ParentType>],
}
export type ErrorCode = { 'MissingRequiredField' : null } |
  { 'ValidationFailed' : null } |
  { 'ResourceAlreadyExists' : null } |
  { 'DataCorruption' : null } |
  { 'ResourceNotAvailable' : null } |
  { 'AuthForbidden' : null } |
  { 'InvalidInput' : null } |
  { 'OperationFailed' : null } |
  { 'InvalidFormat' : null } |
  { 'DataInconsistency' : null } |
  { 'DependencyFailed' : null } |
  { 'SystemError' : null } |
  { 'DataLoss' : null } |
  { 'OperationTimeout' : null } |
  { 'ContentTooLong' : null } |
  { 'NotFound' : null } |
  { 'PermissionDenied' : null } |
  { 'OperationCancelled' : null } |
  { 'InvalidData' : null } |
  { 'InvalidCredentials' : null } |
  { 'Unauthorized' : null } |
  { 'AlreadyExists' : null } |
  { 'UnexpectedError' : null } |
  { 'RateLimitExceeded' : null } |
  { 'ServiceUnavailable' : null } |
  { 'ResourceUnavailable' : null } |
  { 'InsufficientPermissions' : null } |
  { 'ResourceNotFound' : null } |
  { 'ResourceExhausted' : null } |
  { 'ValidationInvalidInput' : null } |
  { 'AuthUnauthorized' : null } |
  { 'Forbidden' : null } |
  { 'SessionExpired' : null } |
  { 'InvalidOperation' : null } |
  { 'QuotaExceeded' : null } |
  { 'ServiceError' : null } |
  { 'ServiceTimeout' : null };
export interface ErrorContext {
  'function' : string,
  'timestamp' : bigint,
  'details' : [] | [string],
  'entity_id' : [] | [string],
  'severity' : ErrorSeverity,
  'module' : string,
}
export type ErrorSeverity = { 'Error' : null } |
  { 'Info' : null } |
  { 'Critical' : null } |
  { 'Warning' : null };
export interface FeedResponse {
  'total' : bigint,
  'comments' : Array<CommentResponse>,
  'posts' : Array<PostResponse>,
  'next_offset' : bigint,
  'has_more' : boolean,
}
export interface GetHotTagsRequest {
  'limit' : [] | [number],
  'tag_type' : [] | [TagType],
}
export interface GetTrendingTopicsRequest {
  'limit' : [] | [number],
  'time_range_hours' : [] | [number],
}
export interface HeartbeatIntervalResponse { 'interval_hours' : bigint }
export interface HotTagInfo {
  'name' : string,
  'count' : bigint,
  'tag_type' : TagType,
}
export interface HotTagsResponse {
  'updated_at' : bigint,
  'tags' : Array<HotTagInfo>,
}
export interface InteractionPreferences {
  'allow_comments' : boolean,
  'allow_mentions' : boolean,
  'allow_follows' : boolean,
  'show_likes' : boolean,
}
export interface LikeContentRequest {
  'content_id' : string,
  'content_type' : ParentType,
}
export interface LikesResponse {
  'total' : bigint,
  'content_id' : string,
  'content_type' : ParentType,
  'likes' : Array<UserLikeInfo>,
}
export interface LogEntry { 'message' : string, 'timestamp' : bigint }
export interface LoginStreakRequirement { 'days_required' : bigint }
export interface NewsReference {
  'metadata' : Array<[string, string]>,
  'canister_id' : Principal,
}
export interface NotificationPreferences {
  'follows' : boolean,
  'likes' : boolean,
  'comments' : boolean,
  'mentions' : boolean,
  'system' : boolean,
}
export interface PaginationParams {
  'offset' : [] | [bigint],
  'limit' : [] | [bigint],
}
export type ParentType = { 'Post' : null } |
  { 'Comment' : null };
export interface PersonalizedRecommendationsRequest {
  'diversity_factor' : [] | [number],
  'recency_weight' : [] | [number],
  'include_followed_topics' : [] | [boolean],
  'pagination' : PaginationParams,
  'include_followed_users' : [] | [boolean],
  'include_trending' : [] | [boolean],
  'include_similar_to_liked' : [] | [boolean],
  'content_types' : [] | [Array<ParentType>],
}
export interface PostResponse {
  'id' : string,
  'status' : ContentStatus,
  'updated_at' : bigint,
  'content' : string,
  'author_info' : UserSocialResponse,
  'hashtags' : Array<string>,
  'media_urls' : Array<string>,
  'tags' : Array<string>,
  'news_reference' : [] | [NewsReference],
  'created_at' : bigint,
  'author' : Principal,
  'token_mentions' : Array<string>,
  'comments_count' : bigint,
  'visibility' : ContentVisibility_1,
  'likes_count' : bigint,
}
export interface PostsResponse {
  'total' : bigint,
  'posts' : Array<PostResponse>,
  'next_offset' : bigint,
}
export interface RegisterUserRequest {
  'bio' : string,
  'username' : string,
  'interests' : [] | [Array<string>],
  'handle' : string,
  'social_links' : [] | [Array<[string, string]>],
  'avatar' : string,
}
export interface ReportContentRequest {
  'content_id' : string,
  'content_type' : ParentType,
  'description' : [] | [string],
  'reason' : ReportReason,
}
export type ReportReason = { 'Violence' : null } |
  { 'Scam' : null } |
  { 'Spam' : null } |
  { 'Harassment' : null } |
  { 'Other' : null } |
  { 'FalseInformation' : null } |
  { 'IllegalContent' : null };
export type Result = { 'Ok' : null } |
  { 'Err' : SquareError };
export type Result_1 = { 'Ok' : null } |
  { 'Err' : string };
export type Result_10 = { 'Ok' : HotTagsResponse } |
  { 'Err' : SquareError };
export type Result_11 = { 'Ok' : LikesResponse } |
  { 'Err' : SquareError };
export type Result_12 = { 'Ok' : boolean } |
  { 'Err' : SquareError };
export type Result_13 = { 'Ok' : PostResponse } |
  { 'Err' : SquareError };
export type Result_14 = { 'Ok' : PostsResponse } |
  { 'Err' : SquareError };
export type Result_15 = { 'Ok' : Array<TrendingTopicResponse> } |
  { 'Err' : SquareError };
export type Result_16 = { 'Ok' : Array<[string, Value]> } |
  { 'Err' : SquareError };
export type Result_17 = { 'Ok' : Array<Principal> } |
  { 'Err' : string };
export type Result_18 = { 'Ok' : Array<SearchResultResponse> } |
  { 'Err' : SquareError };
export type Result_2 = { 'Ok' : TaskCompletionResponse } |
  { 'Err' : SquareError };
export type Result_3 = { 'Ok' : CommentResponse } |
  { 'Err' : SquareError };
export type Result_4 = { 'Ok' : string } |
  { 'Err' : SquareError };
export type Result_5 = { 'Ok' : FeedResponse } |
  { 'Err' : SquareError };
export type Result_6 = { 'Ok' : Array<TaskResponse> } |
  { 'Err' : SquareError };
export type Result_7 = { 'Ok' : CommentsResponse } |
  { 'Err' : SquareError };
export type Result_8 = { 'Ok' : CyclesBalanceResponse } |
  { 'Err' : SquareError };
export type Result_9 = { 'Ok' : CyclesConsumptionResponse } |
  { 'Err' : SquareError };
export interface SearchRequest {
  'pagination' : PaginationParams,
  'query' : string,
  'content_types' : [] | [Array<ParentType>],
}
export interface SearchResultResponse {
  'id' : string,
  'title' : [] | [string],
  'content_type' : ParentType,
  'relevance_score' : number,
  'snippet' : string,
  'created_at' : bigint,
  'author' : UserSocialResponse,
}
export interface SocialInteractionRequirement {
  'like_count' : [] | [bigint],
  'follow_count' : [] | [bigint],
}
export type SortOption = { 'MostCommented' : null } |
  { 'Trending' : null } |
  { 'MostLiked' : null } |
  { 'Latest' : null };
export type SquareError = { 'Enhanced' : SquareErrorEnhanced } |
  { 'ValidationFailed' : string } |
  { 'SystemError' : string } |
  { 'ContentTooLong' : string } |
  { 'NotFound' : string } |
  { 'Unauthorized' : string } |
  { 'AlreadyExists' : string } |
  { 'InvalidOperation' : string };
export interface SquareErrorEnhanced {
  'recoverable' : boolean,
  'context' : ErrorContext,
  'code' : ErrorCode,
  'message' : string,
  'recovery_hint' : [] | [string],
}
export type TagType = { 'Custom' : null } |
  { 'Category' : null } |
  { 'Topic' : null } |
  { 'Location' : null };
export interface TaskCompletionResponse {
  'total_points' : bigint,
  'message' : string,
  'success' : boolean,
  'points_earned' : bigint,
}
export interface TaskRequirements {
  'social_interaction' : [] | [SocialInteractionRequirement],
  'required_tokens' : [] | [Array<string>],
  'required_nfts' : [] | [Array<string>],
  'login_streak' : [] | [LoginStreakRequirement],
  'custom_requirements' : [] | [Array<string>],
  'content_creation' : [] | [ContentCreationRequirement],
}
export interface TaskResponse {
  'id' : string,
  'title' : string,
  'description' : string,
  'created_at' : bigint,
  'completion_criteria' : string,
  'is_completed' : boolean,
  'task_type' : TaskType,
  'expiration_time' : [] | [bigint],
  'points' : bigint,
}
export type TaskType = { 'OneTime' : null } |
  { 'Weekly' : null } |
  { 'Daily' : null } |
  { 'Monthly' : null } |
  { 'Special' : null };
export type TrendDirection = { 'New' : null } |
  { 'Stable' : null } |
  { 'Rising' : null } |
  { 'Falling' : null };
export interface TrendingTopicResponse {
  'topic' : string,
  'count' : bigint,
  'trend_direction' : TrendDirection,
}
export interface UpdateCommentRequest { 'id' : string, 'content' : string }
export interface UpdateCyclesThresholdRequest {
  'critical_threshold' : [] | [bigint],
  'warning_threshold' : [] | [bigint],
  'notification_enabled' : [] | [boolean],
}
export interface UpdatePostRequest {
  'id' : string,
  'content' : string,
  'hashtags' : [] | [Array<string>],
  'media_urls' : [] | [Array<string>],
  'tags' : [] | [Array<string>],
  'news_reference' : [] | [NewsReference],
  'token_mentions' : [] | [Array<string>],
  'visibility' : [] | [ContentVisibility_1],
}
export interface UpdateProfileRequest {
  'bio' : [] | [string],
  'username' : [] | [string],
  'interests' : [] | [Array<string>],
  'handle' : [] | [string],
  'privacy_settings' : [] | [UserPrivacySettings],
  'social_links' : [] | [Array<[string, string]>],
  'avatar' : [] | [string],
}
export interface UserLeaderboardItem {
  'principal' : Principal,
  'username' : string,
  'comment_count' : bigint,
  'last_claim_date' : bigint,
  'consecutive_daily_logins' : bigint,
  'like_count' : bigint,
  'rank' : bigint,
  'post_count' : bigint,
  'reputation' : bigint,
  'handle' : string,
  'followers_count' : bigint,
  'avatar' : string,
}
export interface UserLeaderboardResponse {
  'total_users' : bigint,
  'users' : Array<UserLeaderboardItem>,
  'next_offset' : bigint,
  'has_more' : boolean,
}
export interface UserLikeInfo {
  'principal' : Principal,
  'username' : string,
  'timestamp' : bigint,
}
export interface UserPrivacySettings {
  'notification_preferences' : NotificationPreferences,
  'content_visibility' : ContentVisibility_1,
  'interaction_preferences' : InteractionPreferences,
  'profile_visibility' : ContentVisibility_1,
}
export interface UserProfileResponse {
  'bio' : string,
  'is_following' : boolean,
  'status' : UserStatus,
  'last_login' : bigint,
  'updated_at' : bigint,
  'principal' : Principal,
  'username' : string,
  'interests' : Array<string>,
  'role' : UserRole,
  'following_count' : bigint,
  'created_at' : bigint,
  'handle' : string,
  'registered_at' : bigint,
  'followers_count' : bigint,
  'privacy_settings' : [] | [UserPrivacySettings],
  'social_links' : Array<[string, string]>,
  'avatar' : string,
}
export type UserRole = { 'User' : null } |
  { 'Admin' : null } |
  { 'Moderator' : null } |
  { 'Creator' : null };
export interface UserSocialResponse {
  'bio' : string,
  'is_following' : boolean,
  'principal' : Principal,
  'username' : string,
  'interests' : Array<string>,
  'following_count' : bigint,
  'is_followed_by_caller' : boolean,
  'handle' : string,
  'followers_count' : bigint,
  'avatar' : string,
}
export type UserStatus = { 'Active' : null } |
  { 'Suspended' : null } |
  { 'Banned' : null } |
  { 'Restricted' : null };
export type Value = { 'Int' : bigint } |
  { 'Map' : Array<[string, Value]> } |
  { 'Nat' : bigint } |
  { 'Blob' : Uint8Array | number[] } |
  { 'Bool' : boolean } |
  { 'Null' : null } |
  { 'Text' : string } |
  { 'Float' : number } |
  { 'Principal' : Principal } |
  { 'Array' : Array<Value> };
export interface _SERVICE {
  'acknowledge_notification' : ActorMethod<[bigint], Result>,
  'add_manager' : ActorMethod<[Principal], Result_1>,
  'award_points' : ActorMethod<[AwardPointsRequest], Result>,
  'clear_logs' : ActorMethod<[], boolean>,
  'complete_task' : ActorMethod<[CompleteTaskRequest], Result_2>,
  'create_comment' : ActorMethod<[CreateCommentRequest], Result_3>,
  'create_post' : ActorMethod<[CreatePostRequest], ApiResponse>,
  'create_task' : ActorMethod<[CreateTaskRequest], Result_4>,
  'debug_fix_user_data' : ActorMethod<[string], ApiResponse_1>,
  'debug_fix_user_profile' : ActorMethod<[string], ApiResponse_2>,
  'debug_list_all_users' : ActorMethod<[], ApiResponse_3>,
  'delete_comment' : ActorMethod<[string], Result>,
  'delete_post' : ActorMethod<[string], Result>,
  'delete_task' : ActorMethod<[string], Result>,
  'discover_content' : ActorMethod<[DiscoverContentRequest], Result_5>,
  'follow_user' : ActorMethod<[Principal], ApiResponse_4>,
  'get_available_tasks' : ActorMethod<[], Result_6>,
  'get_comment' : ActorMethod<[string], Result_3>,
  'get_comments' : ActorMethod<[string, string, PaginationParams], Result_7>,
  'get_cycles_balance' : ActorMethod<[], Result_8>,
  'get_cycles_consumption_history' : ActorMethod<[], Result_9>,
  'get_cycles_notifications' : ActorMethod<[], ApiResponse_5>,
  'get_cycles_threshold' : ActorMethod<[], ApiResponse_6>,
  'get_error_history' : ActorMethod<[], ApiResponse_7>,
  'get_error_stats' : ActorMethod<[], ApiResponse_8>,
  'get_followers' : ActorMethod<[[] | [string]], ApiResponse_9>,
  'get_following' : ActorMethod<[[] | [string]], ApiResponse_9>,
  'get_heartbeat_interval' : ActorMethod<[], ApiResponse_10>,
  'get_hot_tags' : ActorMethod<[GetHotTagsRequest], Result_10>,
  'get_likes' : ActorMethod<[string, ParentType], Result_11>,
  'get_logs' : ActorMethod<[], Array<LogEntry>>,
  'get_most_common_errors' : ActorMethod<[bigint], ApiResponse_11>,
  'get_notification_settings' : ActorMethod<[], Result_12>,
  'get_personalized_recommendations' : ActorMethod<
    [PersonalizedRecommendationsRequest],
    Result_5
  >,
  'get_post' : ActorMethod<[string], Result_13>,
  'get_posts' : ActorMethod<[PaginationParams], Result_14>,
  'get_recent_logs' : ActorMethod<[bigint], Array<LogEntry>>,
  'get_trending_topics' : ActorMethod<[GetTrendingTopicsRequest], Result_15>,
  'get_user_content' : ActorMethod<
    [[] | [string], [] | [ParentType], PaginationParams],
    Result_5
  >,
  'get_user_leaderboard' : ActorMethod<[PaginationParams], ApiResponse_12>,
  'get_user_profile' : ActorMethod<[[] | [string]], ApiResponse_13>,
  'get_user_rewards' : ActorMethod<[], Result_16>,
  'like_content' : ActorMethod<[LikeContentRequest], Result>,
  'list_managers' : ActorMethod<[], Result_17>,
  'migrate_storage' : ActorMethod<[], ApiResponse_2>,
  'moderate_content' : ActorMethod<[ContentModerationRequest], Result>,
  'register_user' : ActorMethod<[RegisterUserRequest], ApiResponse_4>,
  'remove_manager' : ActorMethod<[Principal], Result_1>,
  'report_content' : ActorMethod<[ReportContentRequest], Result>,
  'search_content' : ActorMethod<[SearchRequest], Result_18>,
  'unfollow_user' : ActorMethod<[Principal], ApiResponse_4>,
  'unlike_content' : ActorMethod<[LikeContentRequest], Result>,
  'update_comment' : ActorMethod<[UpdateCommentRequest], Result_3>,
  'update_cycles_threshold' : ActorMethod<
    [UpdateCyclesThresholdRequest],
    ApiResponse_6
  >,
  'update_heartbeat_interval' : ActorMethod<
    [HeartbeatIntervalResponse],
    ApiResponse_10
  >,
  'update_notification_settings' : ActorMethod<[[] | [boolean]], Result>,
  'update_post' : ActorMethod<[UpdatePostRequest], ApiResponse>,
  'update_task' : ActorMethod<[CreateTaskRequest], Result>,
  'update_user_profile' : ActorMethod<[UpdateProfileRequest], ApiResponse_2>,
}
export declare const idlFactory: IDL.InterfaceFactory;
export declare const init: (args: { IDL: typeof IDL }) => IDL.Type[];
