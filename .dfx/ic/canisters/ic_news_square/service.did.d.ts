import type { Principal } from '@dfinity/principal';
import type { ActorMethod } from '@dfinity/agent';
import type { IDL } from '@dfinity/candid';

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
  'reference_id' : [] | [string],
  'user' : Principal,
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
  'shares_count' : bigint,
  'created_at' : bigint,
  'author' : Principal,
  'next_child_comment_offset' : bigint,
  'parent_id' : string,
  'is_liked' : boolean,
  'has_more_child_comments' : boolean,
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
export interface ContentCreationRequirement {
  'comment_count' : [] | [bigint],
  'article_count' : [] | [bigint],
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
export interface CreateArticleRequest {
  'content' : string,
  'hashtags' : Array<string>,
  'media_urls' : Array<string>,
  'token_mentions' : Array<string>,
  'visibility' : ContentVisibility,
}
export interface CreateCommentRequest {
  'content' : string,
  'parent_id' : string,
  'parent_type' : ParentType,
}
export interface CreatePostRequest {
  'content' : string,
  'hashtags' : Array<string>,
  'media_urls' : Array<string>,
  'tags' : Array<string>,
  'news_reference' : [] | [NewsReferenceRequest],
  'token_mentions' : Array<string>,
  'visibility' : ContentVisibility,
}
export interface CreateTaskRequest {
  'title' : string,
  'points_reward' : bigint,
  'canister_id' : Principal,
  'description' : string,
  'end_time' : [] | [bigint],
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
  'sort_by' : SortOption,
  'pagination' : PaginationParams,
  'filter' : [] | [ContentFilter],
}
export interface FeedResponse {
  'articles' : Array<ArticleResponse>,
  'comments' : Array<CommentResponse>,
  'posts' : Array<PostResponse>,
  'next_offset' : bigint,
  'has_more' : boolean,
}
export interface GetHotTagsRequest { 'limit' : [] | [bigint] }
export interface GetTrendingTopicsRequest {
  'time_period' : TimePeriod,
  'limit' : bigint,
}
export interface HotTagsResponse { 'tags' : Array<TagStats> }
export interface LikesResponse {
  'total' : bigint,
  'content_id' : string,
  'content_type' : ParentType,
  'likes' : Array<UserLikeInfo>,
}
export interface LoginStreakRequirement { 'days_required' : bigint }
export interface NewsReference {
  'metadata' : Array<[string, string]>,
  'canister_id' : Principal,
}
export interface NewsReferenceRequest {
  'metadata' : Array<[string, string]>,
  'canister_id' : Principal,
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
  'limit' : bigint,
  'content_types' : Array<ParentType>,
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
  'news_reference' : [] | [NewsReference],
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
  'bio' : string,
  'username' : string,
  'handle' : string,
  'social_links' : Array<[string, string]>,
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
export type Result_10 = { 'Ok' : CyclesBalanceResponse } |
  { 'Err' : SquareError };
export type Result_11 = { 'Ok' : CyclesConsumptionResponse } |
  { 'Err' : SquareError };
export type Result_12 = { 'Ok' : CyclesNotificationsResponse } |
  { 'Err' : SquareError };
export type Result_13 = { 'Ok' : CyclesThresholdConfig } |
  { 'Err' : SquareError };
export type Result_14 = { 'Ok' : Array<UserSocialResponse> } |
  { 'Err' : SquareError };
export type Result_15 = { 'Ok' : HotTagsResponse } |
  { 'Err' : SquareError };
export type Result_16 = { 'Ok' : LikesResponse } |
  { 'Err' : SquareError };
export type Result_17 = { 'Ok' : NotificationSettings } |
  { 'Err' : SquareError };
export type Result_18 = { 'Ok' : PostsResponse } |
  { 'Err' : SquareError };
export type Result_19 = { 'Ok' : SharesResponse } |
  { 'Err' : SquareError };
export type Result_2 = { 'Ok' : TaskCompletionResponse } |
  { 'Err' : SquareError };
export type Result_20 = { 'Ok' : Array<TrendingTopicResponse> } |
  { 'Err' : SquareError };
export type Result_21 = { 'Ok' : UserLeaderboardResponse } |
  { 'Err' : SquareError };
export type Result_22 = { 'Ok' : UserProfileResponse } |
  { 'Err' : SquareError };
export type Result_23 = { 'Ok' : UserRewardsResponse } |
  { 'Err' : SquareError };
export type Result_24 = { 'Ok' : Array<Principal> } |
  { 'Err' : string };
export type Result_25 = { 'Ok' : Array<SearchResultResponse> } |
  { 'Err' : SquareError };
export type Result_3 = { 'Ok' : ArticleResponse } |
  { 'Err' : SquareError };
export type Result_4 = { 'Ok' : CommentResponse } |
  { 'Err' : SquareError };
export type Result_5 = { 'Ok' : PostResponse } |
  { 'Err' : SquareError };
export type Result_6 = { 'Ok' : string } |
  { 'Err' : SquareError };
export type Result_7 = { 'Ok' : FeedResponse } |
  { 'Err' : SquareError };
export type Result_8 = { 'Ok' : Array<TaskResponse> } |
  { 'Err' : SquareError };
export type Result_9 = { 'Ok' : CommentsResponse } |
  { 'Err' : SquareError };
export interface SearchRequest {
  'pagination' : PaginationParams,
  'query' : string,
  'content_types' : Array<ParentType>,
}
export interface SearchResultResponse {
  'content_id' : string,
  'content_type' : ParentType,
  'relevance_score' : number,
  'snippet' : string,
  'created_at' : bigint,
  'author' : Principal,
  'author_username' : string,
  'likes_count' : bigint,
}
export interface SharesResponse {
  'content_id' : string,
  'count' : bigint,
  'content_type' : ParentType,
}
export interface SocialInteractionRequirement {
  'share_count' : [] | [bigint],
  'like_count' : [] | [bigint],
  'follow_count' : [] | [bigint],
}
export type SortOption = { 'MostShared' : null } |
  { 'MostCommented' : null } |
  { 'Trending' : null } |
  { 'MostLiked' : null } |
  { 'Latest' : null };
export type SquareError = { 'ValidationFailed' : string } |
  { 'SystemError' : string } |
  { 'ContentTooLong' : string } |
  { 'NotFound' : string } |
  { 'Unauthorized' : string } |
  { 'AlreadyExists' : string } |
  { 'InvalidOperation' : string };
export interface TagStats {
  'tag' : string,
  'post_count' : bigint,
  'last_used' : bigint,
}
export interface TaskCompletionResponse {
  'task_id' : string,
  'total_points' : bigint,
  'success' : boolean,
  'points_earned' : bigint,
}
export interface TaskRequirements {
  'social_interaction' : [] | [SocialInteractionRequirement],
  'custom' : [] | [string],
  'login_streak' : [] | [LoginStreakRequirement],
  'content_creation' : [] | [ContentCreationRequirement],
}
export interface TaskResponse {
  'id' : string,
  'title' : string,
  'points_reward' : bigint,
  'description' : string,
  'is_completed' : boolean,
  'task_type' : TaskType,
  'completion_time' : [] | [bigint],
  'expiration_time' : [] | [bigint],
}
export type TaskType = { 'Event' : null } |
  { 'OneTime' : null } |
  { 'Weekly' : null } |
  { 'Daily' : null };
export type TimePeriod = { 'Day' : null } |
  { 'AllTime' : null } |
  { 'Week' : null } |
  { 'Month' : null };
export type TrendDirection = { 'New' : null } |
  { 'Stable' : null } |
  { 'Rising' : null } |
  { 'Falling' : null };
export interface TrendingTopicResponse {
  'topic' : string,
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
  'news_reference' : [] | [NewsReference],
  'token_mentions' : [] | [Array<string>],
  'visibility' : [] | [ContentVisibility],
}
export interface UpdateProfileRequest {
  'bio' : string,
  'username' : string,
  'handle' : [] | [string],
  'social_links' : [] | [Array<[string, string]>],
  'avatar' : string,
}
export interface UpdateTaskRequest {
  'id' : string,
  'title' : [] | [string],
  'points_reward' : [] | [bigint],
  'description' : [] | [string],
  'end_time' : [] | [bigint],
  'start_time' : [] | [bigint],
  'requirements' : [] | [TaskRequirements],
}
export interface UserLeaderboardItem {
  'principal' : Principal,
  'username' : string,
  'last_claim_date' : [] | [bigint],
  'consecutive_daily_logins' : bigint,
  'rank' : bigint,
  'article_count' : bigint,
  'post_count' : bigint,
  'followers_count' : bigint,
  'points' : bigint,
  'avatar' : string,
}
export interface UserLeaderboardResponse {
  'total' : bigint,
  'users' : Array<UserLeaderboardItem>,
  'next_offset' : bigint,
}
export interface UserLikeInfo {
  'principal' : Principal,
  'username' : string,
  'timestamp' : bigint,
}
export interface UserProfileResponse {
  'bio' : string,
  'status' : UserStatus,
  'last_login' : bigint,
  'principal' : Principal,
  'username' : string,
  'role' : UserRole,
  'following_count' : bigint,
  'handle' : string,
  'registered_at' : bigint,
  'followers_count' : bigint,
  'social_links' : Array<[string, string]>,
  'avatar' : string,
}
export interface UserRewardsResponse {
  'principal' : Principal,
  'last_claim_date' : [] | [bigint],
  'consecutive_daily_logins' : bigint,
  'points_history' : Array<PointsTransaction>,
  'points' : bigint,
}
export type UserRole = { 'User' : null } |
  { 'Admin' : null } |
  { 'Moderator' : null } |
  { 'Creator' : null };
export interface UserSocialResponse {
  'bio' : string,
  'principal' : Principal,
  'username' : string,
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
export interface _SERVICE {
  'acknowledge_notification' : ActorMethod<[bigint], Result>,
  'add_manager' : ActorMethod<[Principal], Result_1>,
  'award_points' : ActorMethod<[AwardPointsRequest], Result>,
  'complete_task' : ActorMethod<[CompleteTaskRequest], Result_2>,
  'create_article' : ActorMethod<[CreateArticleRequest], Result_3>,
  'create_comment' : ActorMethod<[CreateCommentRequest], Result_4>,
  'create_post' : ActorMethod<[CreatePostRequest], Result_5>,
  'create_task' : ActorMethod<[CreateTaskRequest], Result_6>,
  'delete_article' : ActorMethod<[string], Result>,
  'delete_comment' : ActorMethod<[string], Result>,
  'delete_post' : ActorMethod<[string], Result>,
  'delete_task' : ActorMethod<[string], Result>,
  'discover_content' : ActorMethod<[DiscoverContentRequest], Result_7>,
  'follow_user' : ActorMethod<[Principal], Result>,
  'get_article' : ActorMethod<[string], Result_3>,
  'get_available_tasks' : ActorMethod<[], Result_8>,
  'get_comment' : ActorMethod<[string], Result_4>,
  'get_comments' : ActorMethod<[string, string, PaginationParams], Result_9>,
  'get_cycles_balance' : ActorMethod<[], Result_10>,
  'get_cycles_consumption_history' : ActorMethod<[], Result_11>,
  'get_cycles_notifications' : ActorMethod<[], Result_12>,
  'get_cycles_threshold' : ActorMethod<[], Result_13>,
  'get_followers' : ActorMethod<[[] | [Principal]], Result_14>,
  'get_following' : ActorMethod<[[] | [Principal]], Result_14>,
  'get_hot_tags' : ActorMethod<[GetHotTagsRequest], Result_15>,
  'get_likes' : ActorMethod<[string, ParentType], Result_16>,
  'get_notification_settings' : ActorMethod<[], Result_17>,
  'get_personalized_recommendations' : ActorMethod<
    [PersonalizedRecommendationsRequest],
    Result_7
  >,
  'get_post' : ActorMethod<[string], Result_5>,
  'get_posts' : ActorMethod<[PaginationParams], Result_18>,
  'get_shares' : ActorMethod<[string, ParentType], Result_19>,
  'get_trending_topics' : ActorMethod<[GetTrendingTopicsRequest], Result_20>,
  'get_user_content' : ActorMethod<
    [[] | [Principal], [] | [ParentType], PaginationParams],
    Result_7
  >,
  'get_user_leaderboard' : ActorMethod<[PaginationParams], Result_21>,
  'get_user_profile' : ActorMethod<[[] | [Principal]], Result_22>,
  'get_user_rewards' : ActorMethod<[], Result_23>,
  'like_content' : ActorMethod<[string, ParentType], Result>,
  'list_managers' : ActorMethod<[], Result_24>,
  'migrate_to_sharded_storage' : ActorMethod<[], Result_6>,
  'moderate_content' : ActorMethod<[ContentModerationRequest], Result>,
  'register_user' : ActorMethod<[RegisterUserRequest], Result>,
  'remove_manager' : ActorMethod<[Principal], Result_1>,
  'report_content' : ActorMethod<[ReportContentRequest], Result>,
  'search_content' : ActorMethod<[SearchRequest], Result_25>,
  'share_content' : ActorMethod<[string, ParentType], Result>,
  'unfollow_user' : ActorMethod<[Principal], Result>,
  'unlike_content' : ActorMethod<[string, ParentType], Result>,
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
  'update_post' : ActorMethod<[UpdatePostRequest], Result_5>,
  'update_task' : ActorMethod<[UpdateTaskRequest], Result>,
  'update_user_profile' : ActorMethod<[UpdateProfileRequest], Result>,
}
export declare const idlFactory: IDL.InterfaceFactory;
export declare const init: (args: { IDL: typeof IDL }) => IDL.Type[];
