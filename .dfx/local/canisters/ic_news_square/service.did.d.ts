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
  'data' : [] | [null],
  'error' : [] | [ApiError],
  'success' : boolean,
}
export interface ApiResponse_2 {
  'data' : [] | [Array<string>],
  'error' : [] | [ApiError],
  'success' : boolean,
}
export interface ApiResponse_3 {
  'data' : [] | [Array<[ErrorCode, bigint, bigint, bigint]>],
  'error' : [] | [ApiError],
  'success' : boolean,
}
export interface ApiResponse_4 {
  'data' : [] | [Array<UserSocialResponse>],
  'error' : [] | [ApiError],
  'success' : boolean,
}
export interface ApiResponse_5 {
  'data' : [] | [Array<[ErrorCode, bigint]>],
  'error' : [] | [ApiError],
  'success' : boolean,
}
export interface ApiResponse_6 {
  'data' : [] | [UserLeaderboardResponse],
  'error' : [] | [ApiError],
  'success' : boolean,
}
export interface ApiResponse_7 {
  'data' : [] | [UserProfileResponse],
  'error' : [] | [ApiError],
  'success' : boolean,
}
export interface ArticleResponse {
  'id' : string,
  'status' : ContentStatus,
  'updated_at' : bigint,
  'content' : string,
  'author_info' : UserSocialResponse,
  'hashtags' : Array<string>,
  'shares_count' : bigint,
  'media_urls' : Array<string>,
  'created_at' : bigint,
  'author' : Principal,
  'token_mentions' : Array<string>,
  'comments_count' : bigint,
  'visibility' : ContentVisibility,
  'likes_count' : bigint,
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
  'child_comments' : Array<string>,
  'author_info' : UserSocialResponse,
  'shares_count' : bigint,
  'created_at' : bigint,
  'author' : Principal,
  'parent_id' : string,
  'is_liked' : boolean,
  'comments_count' : bigint,
  'visibility' : ContentVisibility,
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
export interface CreateArticleRequest {
  'id' : [] | [string],
  'is_nsfw' : [] | [boolean],
  'content' : string,
  'hashtags' : Array<string>,
  'media_urls' : Array<string>,
  'token_mentions' : [] | [Array<string>],
  'visibility' : [] | [ContentVisibility],
}
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
  'requirements' : TaskRequirements,
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
  'pagination' : PaginationParams,
  'tags' : [] | [Array<string>],
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
  'articles' : Array<ArticleResponse>,
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
export interface NotificationPreferences {
  'shares' : boolean,
  'follows' : boolean,
  'likes' : boolean,
  'comments' : boolean,
  'mentions' : boolean,
  'system' : boolean,
}
export interface NotificationSettings {
  'email_address' : [] | [string],
  'notification_frequency_hours' : bigint,
  'email_enabled' : boolean,
}
export interface PaginationParams { 'offset' : bigint, 'limit' : bigint }
export type ParentType = { 'Article' : null } |
  { 'Post' : null } |
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
export interface PointsTransaction {
  'reference_id' : [] | [string],
  'timestamp' : bigint,
  'amount' : bigint,
  'reason' : string,
}
export interface PostResponse {
  'id' : string,
  'status' : ContentStatus,
  'updated_at' : bigint,
  'content' : string,
  'author_info' : UserSocialResponse,
  'hashtags' : Array<string>,
  'shares_count' : bigint,
  'media_urls' : Array<string>,
  'tags' : Array<string>,
  'created_at' : bigint,
  'author' : Principal,
  'token_mentions' : Array<string>,
  'comments_count' : bigint,
  'visibility' : ContentVisibility,
  'likes_count' : bigint,
}
export interface PostsResponse {
  'total' : bigint,
  'posts' : Array<PostResponse>,
  'next_offset' : bigint,
}
export interface RegisterUserRequest {
  'bio' : [] | [string],
  'username' : string,
  'interests' : [] | [Array<string>],
  'handle' : string,
  'social_links' : [] | [Array<[string, string]>],
  'avatar' : [] | [string],
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
export type Result_10 = { 'Ok' : CyclesConsumptionResponse } |
  { 'Err' : SquareError };
export type Result_11 = { 'Ok' : CyclesNotificationsResponse } |
  { 'Err' : SquareError };
export type Result_12 = { 'Ok' : CyclesThresholdConfig } |
  { 'Err' : SquareError };
export type Result_13 = { 'Ok' : HotTagsResponse } |
  { 'Err' : SquareError };
export type Result_14 = { 'Ok' : LikesResponse } |
  { 'Err' : SquareError };
export type Result_15 = { 'Ok' : NotificationSettings } |
  { 'Err' : SquareError };
export type Result_16 = { 'Ok' : PostResponse } |
  { 'Err' : SquareError };
export type Result_17 = { 'Ok' : PostsResponse } |
  { 'Err' : SquareError };
export type Result_18 = { 'Ok' : SharesResponse } |
  { 'Err' : SquareError };
export type Result_19 = { 'Ok' : Array<TrendingTopicResponse> } |
  { 'Err' : SquareError };
export type Result_2 = { 'Ok' : TaskCompletionResponse } |
  { 'Err' : SquareError };
export type Result_20 = { 'Ok' : UserRewardsResponse } |
  { 'Err' : SquareError };
export type Result_21 = { 'Ok' : Array<Principal> } |
  { 'Err' : string };
export type Result_22 = { 'Ok' : Array<SearchResultResponse> } |
  { 'Err' : SquareError };
export type Result_3 = { 'Ok' : ArticleResponse } |
  { 'Err' : SquareError };
export type Result_4 = { 'Ok' : CommentResponse } |
  { 'Err' : SquareError };
export type Result_5 = { 'Ok' : string } |
  { 'Err' : SquareError };
export type Result_6 = { 'Ok' : FeedResponse } |
  { 'Err' : SquareError };
export type Result_7 = { 'Ok' : Array<TaskResponse> } |
  { 'Err' : SquareError };
export type Result_8 = { 'Ok' : CommentsResponse } |
  { 'Err' : SquareError };
export type Result_9 = { 'Ok' : CyclesBalanceResponse } |
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
  'author' : Principal,
}
export interface SharesResponse {
  'content_id' : string,
  'count' : bigint,
  'content_type' : ParentType,
}
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
  'level_up' : boolean,
  'total_points' : bigint,
  'level' : bigint,
  'message' : string,
  'success' : boolean,
  'points_earned' : bigint,
}
export interface TaskRequirements {
  'required_tokens' : Array<string>,
  'required_nfts' : Array<string>,
  'min_level' : bigint,
  'custom_requirements' : Array<string>,
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
export type TrendDirection = { 'Stable' : null } |
  { 'Rising' : null } |
  { 'Falling' : null };
export interface TrendingTopicResponse {
  'name' : string,
  'count' : bigint,
  'trend_direction' : TrendDirection,
}
export interface UpdateArticleRequest {
  'id' : string,
  'content' : string,
  'hashtags' : [] | [Array<string>],
  'media_urls' : [] | [Array<string>],
  'token_mentions' : [] | [Array<string>],
  'visibility' : [] | [ContentVisibility],
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
  'token_mentions' : [] | [Array<string>],
  'visibility' : [] | [ContentVisibility],
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
  'rank' : bigint,
  'level' : bigint,
  'handle' : string,
  'points' : bigint,
  'avatar' : [] | [string],
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
  'content_visibility' : ContentVisibility,
  'interaction_preferences' : InteractionPreferences,
  'profile_visibility' : ContentVisibility,
}
export interface UserProfileResponse {
  'bio' : [] | [string],
  'is_following' : boolean,
  'updated_at' : bigint,
  'principal' : Principal,
  'username' : string,
  'interests' : Array<string>,
  'following_count' : bigint,
  'created_at' : bigint,
  'handle' : string,
  'followers_count' : bigint,
  'social_links' : Array<[string, string]>,
  'avatar' : [] | [string],
}
export interface UserRewardsResponse {
  'completed_tasks' : Array<string>,
  'points_history' : Array<PointsTransaction>,
  'level' : bigint,
  'points' : bigint,
}
export interface UserSocialResponse {
  'bio' : [] | [string],
  'is_following' : boolean,
  'principal' : Principal,
  'username' : string,
  'following_count' : bigint,
  'is_followed_by_caller' : boolean,
  'handle' : string,
  'followers_count' : bigint,
  'avatar' : [] | [string],
}
export interface _SERVICE {
  'acknowledge_notification' : ActorMethod<[bigint], Result>,
  'add_manager' : ActorMethod<[Principal], Result_1>,
  'award_points' : ActorMethod<[AwardPointsRequest], Result>,
  'complete_task' : ActorMethod<[CompleteTaskRequest], Result_2>,
  'create_article' : ActorMethod<[CreateArticleRequest], Result_3>,
  'create_comment' : ActorMethod<[CreateCommentRequest], Result_4>,
  'create_post' : ActorMethod<[CreatePostRequest], ApiResponse>,
  'create_task' : ActorMethod<[CreateTaskRequest], Result_5>,
  'delete_article' : ActorMethod<[string], Result>,
  'delete_comment' : ActorMethod<[string], Result>,
  'delete_post' : ActorMethod<[string], Result>,
  'delete_task' : ActorMethod<[string], Result>,
  'discover_content' : ActorMethod<[DiscoverContentRequest], Result_6>,
  'follow_user' : ActorMethod<[Principal], ApiResponse_1>,
  'get_article' : ActorMethod<[string], Result_3>,
  'get_available_tasks' : ActorMethod<[], Result_7>,
  'get_comment' : ActorMethod<[string], Result_4>,
  'get_comments' : ActorMethod<[string, string, PaginationParams], Result_8>,
  'get_cycles_balance' : ActorMethod<[], Result_9>,
  'get_cycles_consumption_history' : ActorMethod<[], Result_10>,
  'get_cycles_notifications' : ActorMethod<[], Result_11>,
  'get_cycles_threshold' : ActorMethod<[], Result_12>,
  'get_error_history' : ActorMethod<[], ApiResponse_2>,
  'get_error_stats' : ActorMethod<[], ApiResponse_3>,
  'get_followers' : ActorMethod<[[] | [Principal]], ApiResponse_4>,
  'get_following' : ActorMethod<[[] | [Principal]], ApiResponse_4>,
  'get_hot_tags' : ActorMethod<[GetHotTagsRequest], Result_13>,
  'get_likes' : ActorMethod<[string, ParentType], Result_14>,
  'get_most_common_errors' : ActorMethod<[bigint], ApiResponse_5>,
  'get_notification_settings' : ActorMethod<[], Result_15>,
  'get_personalized_recommendations' : ActorMethod<
    [PersonalizedRecommendationsRequest],
    Result_6
  >,
  'get_post' : ActorMethod<[string], Result_16>,
  'get_posts' : ActorMethod<[PaginationParams], Result_17>,
  'get_shares' : ActorMethod<[string, ParentType], Result_18>,
  'get_trending_topics' : ActorMethod<[GetTrendingTopicsRequest], Result_19>,
  'get_user_content' : ActorMethod<
    [[] | [Principal], [] | [ParentType], PaginationParams],
    Result_6
  >,
  'get_user_leaderboard' : ActorMethod<[PaginationParams], ApiResponse_6>,
  'get_user_profile' : ActorMethod<[[] | [Principal]], ApiResponse_7>,
  'get_user_rewards' : ActorMethod<[], Result_20>,
  'like_content' : ActorMethod<[LikeContentRequest], Result>,
  'list_managers' : ActorMethod<[], Result_21>,
  'migrate_to_sharded_storage' : ActorMethod<[], Result_5>,
  'moderate_content' : ActorMethod<[ContentModerationRequest], Result>,
  'register_user' : ActorMethod<[RegisterUserRequest], ApiResponse_1>,
  'remove_manager' : ActorMethod<[Principal], Result_1>,
  'report_content' : ActorMethod<[ReportContentRequest], Result>,
  'search_content' : ActorMethod<[SearchRequest], Result_22>,
  'share_content' : ActorMethod<[string, ParentType], Result>,
  'test_error_context' : ActorMethod<[], ApiResponse_2>,
  'test_error_handling' : ActorMethod<[], ApiResponse_1>,
  'test_error_recovery_hints' : ActorMethod<[], ApiResponse_2>,
  'unfollow_user' : ActorMethod<[Principal], ApiResponse_1>,
  'unlike_content' : ActorMethod<[LikeContentRequest], Result>,
  'update_article' : ActorMethod<[UpdateArticleRequest], Result_3>,
  'update_comment' : ActorMethod<[UpdateCommentRequest], Result_4>,
  'update_cycles_threshold' : ActorMethod<
    [UpdateCyclesThresholdRequest],
    Result
  >,
  'update_notification_settings' : ActorMethod<
    [[] | [boolean], [] | [string], [] | [bigint]],
    Result
  >,
  'update_post' : ActorMethod<[UpdatePostRequest], ApiResponse>,
  'update_task' : ActorMethod<[CreateTaskRequest], Result>,
  'update_user_profile' : ActorMethod<[UpdateProfileRequest], ApiResponse_1>,
}
export declare const idlFactory: IDL.InterfaceFactory;
export declare const init: (args: { IDL: typeof IDL }) => IDL.Type[];
