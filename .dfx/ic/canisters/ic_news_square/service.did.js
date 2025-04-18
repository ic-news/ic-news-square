export const idlFactory = ({ IDL }) => {
  const CommentResponse = IDL.Rec();
  const SquareError = IDL.Variant({
    'ValidationFailed' : IDL.Text,
    'SystemError' : IDL.Text,
    'ContentTooLong' : IDL.Text,
    'NotFound' : IDL.Text,
    'Unauthorized' : IDL.Text,
    'AlreadyExists' : IDL.Text,
    'InvalidOperation' : IDL.Text,
  });
  const Result = IDL.Variant({ 'Ok' : IDL.Null, 'Err' : SquareError });
  const Result_1 = IDL.Variant({ 'Ok' : IDL.Null, 'Err' : IDL.Text });
  const AwardPointsRequest = IDL.Record({
    'reference_id' : IDL.Opt(IDL.Text),
    'user' : IDL.Principal,
    'points' : IDL.Nat64,
    'reason' : IDL.Text,
  });
  const CompleteTaskRequest = IDL.Record({
    'task_id' : IDL.Text,
    'proof' : IDL.Opt(IDL.Text),
  });
  const TaskCompletionResponse = IDL.Record({
    'task_id' : IDL.Text,
    'total_points' : IDL.Nat64,
    'success' : IDL.Bool,
    'points_earned' : IDL.Nat64,
  });
  const Result_2 = IDL.Variant({
    'Ok' : TaskCompletionResponse,
    'Err' : SquareError,
  });
  const ContentVisibility = IDL.Variant({
    'Private' : IDL.Null,
    'FollowersOnly' : IDL.Null,
    'Public' : IDL.Null,
  });
  const CreateArticleRequest = IDL.Record({
    'content' : IDL.Text,
    'hashtags' : IDL.Vec(IDL.Text),
    'media_urls' : IDL.Vec(IDL.Text),
    'token_mentions' : IDL.Vec(IDL.Text),
    'visibility' : ContentVisibility,
  });
  const ContentStatus = IDL.Variant({
    'UnderReview' : IDL.Null,
    'Active' : IDL.Null,
    'Hidden' : IDL.Null,
    'Removed' : IDL.Null,
    'Deleted' : IDL.Null,
  });
  const UserSocialResponse = IDL.Record({
    'bio' : IDL.Text,
    'principal' : IDL.Principal,
    'username' : IDL.Text,
    'following_count' : IDL.Nat64,
    'is_followed_by_caller' : IDL.Bool,
    'handle' : IDL.Text,
    'followers_count' : IDL.Nat64,
    'avatar' : IDL.Text,
  });
  const ArticleResponse = IDL.Record({
    'id' : IDL.Text,
    'status' : ContentStatus,
    'updated_at' : IDL.Nat64,
    'content' : IDL.Text,
    'author_info' : UserSocialResponse,
    'hashtags' : IDL.Vec(IDL.Text),
    'shares_count' : IDL.Nat64,
    'media_urls' : IDL.Vec(IDL.Text),
    'created_at' : IDL.Nat64,
    'author' : IDL.Principal,
    'token_mentions' : IDL.Vec(IDL.Text),
    'comments_count' : IDL.Nat64,
    'visibility' : ContentVisibility,
    'likes_count' : IDL.Nat64,
  });
  const Result_3 = IDL.Variant({ 'Ok' : ArticleResponse, 'Err' : SquareError });
  const ParentType = IDL.Variant({
    'Article' : IDL.Null,
    'Post' : IDL.Null,
    'Comment' : IDL.Null,
  });
  const CreateCommentRequest = IDL.Record({
    'content' : IDL.Text,
    'parent_id' : IDL.Text,
    'parent_type' : ParentType,
  });
  CommentResponse.fill(
    IDL.Record({
      'id' : IDL.Text,
      'status' : ContentStatus,
      'updated_at' : IDL.Nat64,
      'content' : IDL.Text,
      'child_comments' : IDL.Vec(CommentResponse),
      'author_info' : UserSocialResponse,
      'shares_count' : IDL.Nat64,
      'created_at' : IDL.Nat64,
      'author' : IDL.Principal,
      'next_child_comment_offset' : IDL.Nat64,
      'parent_id' : IDL.Text,
      'is_liked' : IDL.Bool,
      'has_more_child_comments' : IDL.Bool,
      'comments_count' : IDL.Nat64,
      'visibility' : ContentVisibility,
      'likes_count' : IDL.Nat64,
      'parent_type' : ParentType,
    })
  );
  const Result_4 = IDL.Variant({ 'Ok' : CommentResponse, 'Err' : SquareError });
  const NewsReferenceRequest = IDL.Record({
    'metadata' : IDL.Vec(IDL.Tuple(IDL.Text, IDL.Text)),
    'canister_id' : IDL.Principal,
  });
  const CreatePostRequest = IDL.Record({
    'content' : IDL.Text,
    'hashtags' : IDL.Vec(IDL.Text),
    'media_urls' : IDL.Vec(IDL.Text),
    'tags' : IDL.Vec(IDL.Text),
    'news_reference' : IDL.Opt(NewsReferenceRequest),
    'token_mentions' : IDL.Vec(IDL.Text),
    'visibility' : ContentVisibility,
  });
  const NewsReference = IDL.Record({
    'metadata' : IDL.Vec(IDL.Tuple(IDL.Text, IDL.Text)),
    'canister_id' : IDL.Principal,
  });
  const PostResponse = IDL.Record({
    'id' : IDL.Text,
    'status' : ContentStatus,
    'updated_at' : IDL.Nat64,
    'content' : IDL.Text,
    'author_info' : UserSocialResponse,
    'hashtags' : IDL.Vec(IDL.Text),
    'shares_count' : IDL.Nat64,
    'media_urls' : IDL.Vec(IDL.Text),
    'tags' : IDL.Vec(IDL.Text),
    'news_reference' : IDL.Opt(NewsReference),
    'created_at' : IDL.Nat64,
    'author' : IDL.Principal,
    'token_mentions' : IDL.Vec(IDL.Text),
    'comments_count' : IDL.Nat64,
    'visibility' : ContentVisibility,
    'likes_count' : IDL.Nat64,
  });
  const Result_5 = IDL.Variant({ 'Ok' : PostResponse, 'Err' : SquareError });
  const TaskType = IDL.Variant({
    'Event' : IDL.Null,
    'OneTime' : IDL.Null,
    'Weekly' : IDL.Null,
    'Daily' : IDL.Null,
  });
  const SocialInteractionRequirement = IDL.Record({
    'share_count' : IDL.Opt(IDL.Nat64),
    'like_count' : IDL.Opt(IDL.Nat64),
    'follow_count' : IDL.Opt(IDL.Nat64),
  });
  const LoginStreakRequirement = IDL.Record({ 'days_required' : IDL.Nat64 });
  const ContentCreationRequirement = IDL.Record({
    'comment_count' : IDL.Opt(IDL.Nat64),
    'article_count' : IDL.Opt(IDL.Nat64),
    'post_count' : IDL.Opt(IDL.Nat64),
    'required_hashtags' : IDL.Opt(IDL.Vec(IDL.Text)),
  });
  const TaskRequirements = IDL.Record({
    'social_interaction' : IDL.Opt(SocialInteractionRequirement),
    'custom' : IDL.Opt(IDL.Text),
    'login_streak' : IDL.Opt(LoginStreakRequirement),
    'content_creation' : IDL.Opt(ContentCreationRequirement),
  });
  const CreateTaskRequest = IDL.Record({
    'title' : IDL.Text,
    'points_reward' : IDL.Nat64,
    'canister_id' : IDL.Principal,
    'description' : IDL.Text,
    'end_time' : IDL.Opt(IDL.Nat64),
    'start_time' : IDL.Opt(IDL.Nat64),
    'task_type' : TaskType,
    'requirements' : TaskRequirements,
  });
  const Result_6 = IDL.Variant({ 'Ok' : IDL.Text, 'Err' : SquareError });
  const SortOption = IDL.Variant({
    'MostShared' : IDL.Null,
    'MostCommented' : IDL.Null,
    'Trending' : IDL.Null,
    'MostLiked' : IDL.Null,
    'Latest' : IDL.Null,
  });
  const PaginationParams = IDL.Record({
    'offset' : IDL.Nat64,
    'limit' : IDL.Nat64,
  });
  const ContentFilter = IDL.Record({
    'hashtag' : IDL.Opt(IDL.Text),
    'token_mention' : IDL.Opt(IDL.Text),
    'created_after' : IDL.Opt(IDL.Nat64),
    'author' : IDL.Opt(IDL.Principal),
    'created_before' : IDL.Opt(IDL.Nat64),
  });
  const DiscoverContentRequest = IDL.Record({
    'sort_by' : SortOption,
    'pagination' : PaginationParams,
    'filter' : IDL.Opt(ContentFilter),
  });
  const FeedResponse = IDL.Record({
    'articles' : IDL.Vec(ArticleResponse),
    'comments' : IDL.Vec(CommentResponse),
    'posts' : IDL.Vec(PostResponse),
    'next_offset' : IDL.Nat64,
    'has_more' : IDL.Bool,
  });
  const Result_7 = IDL.Variant({ 'Ok' : FeedResponse, 'Err' : SquareError });
  const TaskResponse = IDL.Record({
    'id' : IDL.Text,
    'title' : IDL.Text,
    'points_reward' : IDL.Nat64,
    'description' : IDL.Text,
    'is_completed' : IDL.Bool,
    'task_type' : TaskType,
    'completion_time' : IDL.Opt(IDL.Nat64),
    'expiration_time' : IDL.Opt(IDL.Nat64),
  });
  const Result_8 = IDL.Variant({
    'Ok' : IDL.Vec(TaskResponse),
    'Err' : SquareError,
  });
  const CommentsResponse = IDL.Record({
    'total' : IDL.Nat64,
    'comments' : IDL.Vec(CommentResponse),
    'next_offset' : IDL.Nat64,
    'has_more' : IDL.Bool,
  });
  const Result_9 = IDL.Variant({
    'Ok' : CommentsResponse,
    'Err' : SquareError,
  });
  const CyclesBalanceResponse = IDL.Record({
    'estimated_days_remaining' : IDL.Nat64,
    'threshold_warning' : IDL.Bool,
    'balance' : IDL.Nat64,
    'balance_in_trillion' : IDL.Float64,
  });
  const Result_10 = IDL.Variant({
    'Ok' : CyclesBalanceResponse,
    'Err' : SquareError,
  });
  const DailyConsumption = IDL.Record({
    'date' : IDL.Nat64,
    'operations' : IDL.Nat64,
    'consumption' : IDL.Nat64,
  });
  const CyclesConsumptionResponse = IDL.Record({
    'daily_consumption' : IDL.Vec(DailyConsumption),
    'average_daily_consumption' : IDL.Nat64,
    'total_consumed_last_week' : IDL.Nat64,
  });
  const Result_11 = IDL.Variant({
    'Ok' : CyclesConsumptionResponse,
    'Err' : SquareError,
  });
  const CyclesWarningSeverity = IDL.Variant({
    'Critical' : IDL.Null,
    'Warning' : IDL.Null,
  });
  const CyclesWarningNotification = IDL.Record({
    'balance' : IDL.Nat64,
    'threshold' : IDL.Nat64,
    'message' : IDL.Text,
    'timestamp' : IDL.Nat64,
    'severity' : CyclesWarningSeverity,
    'is_acknowledged' : IDL.Bool,
  });
  const CyclesNotificationsResponse = IDL.Record({
    'notifications' : IDL.Vec(CyclesWarningNotification),
    'unacknowledged_count' : IDL.Nat64,
  });
  const Result_12 = IDL.Variant({
    'Ok' : CyclesNotificationsResponse,
    'Err' : SquareError,
  });
  const CyclesThresholdConfig = IDL.Record({
    'critical_threshold' : IDL.Nat64,
    'warning_threshold' : IDL.Nat64,
    'notification_enabled' : IDL.Bool,
  });
  const Result_13 = IDL.Variant({
    'Ok' : CyclesThresholdConfig,
    'Err' : SquareError,
  });
  const Result_14 = IDL.Variant({
    'Ok' : IDL.Vec(UserSocialResponse),
    'Err' : SquareError,
  });
  const GetHotTagsRequest = IDL.Record({ 'limit' : IDL.Opt(IDL.Nat64) });
  const TagStats = IDL.Record({
    'tag' : IDL.Text,
    'post_count' : IDL.Nat64,
    'last_used' : IDL.Nat64,
  });
  const HotTagsResponse = IDL.Record({ 'tags' : IDL.Vec(TagStats) });
  const Result_15 = IDL.Variant({
    'Ok' : HotTagsResponse,
    'Err' : SquareError,
  });
  const UserLikeInfo = IDL.Record({
    'principal' : IDL.Principal,
    'username' : IDL.Text,
    'timestamp' : IDL.Nat64,
  });
  const LikesResponse = IDL.Record({
    'total' : IDL.Nat64,
    'content_id' : IDL.Text,
    'content_type' : ParentType,
    'likes' : IDL.Vec(UserLikeInfo),
  });
  const Result_16 = IDL.Variant({ 'Ok' : LikesResponse, 'Err' : SquareError });
  const NotificationSettings = IDL.Record({
    'email_address' : IDL.Opt(IDL.Text),
    'notification_frequency_hours' : IDL.Nat64,
    'email_enabled' : IDL.Bool,
  });
  const Result_17 = IDL.Variant({
    'Ok' : NotificationSettings,
    'Err' : SquareError,
  });
  const PersonalizedRecommendationsRequest = IDL.Record({
    'limit' : IDL.Nat64,
    'content_types' : IDL.Vec(ParentType),
  });
  const PostsResponse = IDL.Record({
    'total' : IDL.Nat64,
    'posts' : IDL.Vec(PostResponse),
    'next_offset' : IDL.Nat64,
  });
  const Result_18 = IDL.Variant({ 'Ok' : PostsResponse, 'Err' : SquareError });
  const SharesResponse = IDL.Record({
    'content_id' : IDL.Text,
    'count' : IDL.Nat64,
    'content_type' : ParentType,
  });
  const Result_19 = IDL.Variant({ 'Ok' : SharesResponse, 'Err' : SquareError });
  const TimePeriod = IDL.Variant({
    'Day' : IDL.Null,
    'AllTime' : IDL.Null,
    'Week' : IDL.Null,
    'Month' : IDL.Null,
  });
  const GetTrendingTopicsRequest = IDL.Record({
    'time_period' : TimePeriod,
    'limit' : IDL.Nat64,
  });
  const TrendDirection = IDL.Variant({
    'New' : IDL.Null,
    'Stable' : IDL.Null,
    'Rising' : IDL.Null,
    'Falling' : IDL.Null,
  });
  const TrendingTopicResponse = IDL.Record({
    'topic' : IDL.Text,
    'count' : IDL.Nat64,
    'trend_direction' : TrendDirection,
  });
  const Result_20 = IDL.Variant({
    'Ok' : IDL.Vec(TrendingTopicResponse),
    'Err' : SquareError,
  });
  const UserLeaderboardItem = IDL.Record({
    'principal' : IDL.Principal,
    'username' : IDL.Text,
    'last_claim_date' : IDL.Opt(IDL.Nat64),
    'consecutive_daily_logins' : IDL.Nat64,
    'rank' : IDL.Nat64,
    'article_count' : IDL.Nat64,
    'post_count' : IDL.Nat64,
    'followers_count' : IDL.Nat64,
    'points' : IDL.Nat64,
    'avatar' : IDL.Text,
  });
  const UserLeaderboardResponse = IDL.Record({
    'total' : IDL.Nat64,
    'users' : IDL.Vec(UserLeaderboardItem),
    'next_offset' : IDL.Nat64,
  });
  const Result_21 = IDL.Variant({
    'Ok' : UserLeaderboardResponse,
    'Err' : SquareError,
  });
  const UserStatus = IDL.Variant({
    'Active' : IDL.Null,
    'Suspended' : IDL.Null,
    'Banned' : IDL.Null,
    'Restricted' : IDL.Null,
  });
  const UserRole = IDL.Variant({
    'User' : IDL.Null,
    'Admin' : IDL.Null,
    'Moderator' : IDL.Null,
    'Creator' : IDL.Null,
  });
  const UserProfileResponse = IDL.Record({
    'bio' : IDL.Text,
    'status' : UserStatus,
    'last_login' : IDL.Nat64,
    'principal' : IDL.Principal,
    'username' : IDL.Text,
    'role' : UserRole,
    'following_count' : IDL.Nat64,
    'handle' : IDL.Text,
    'registered_at' : IDL.Nat64,
    'followers_count' : IDL.Nat64,
    'social_links' : IDL.Vec(IDL.Tuple(IDL.Text, IDL.Text)),
    'avatar' : IDL.Text,
  });
  const Result_22 = IDL.Variant({
    'Ok' : UserProfileResponse,
    'Err' : SquareError,
  });
  const PointsTransaction = IDL.Record({
    'reference_id' : IDL.Opt(IDL.Text),
    'timestamp' : IDL.Nat64,
    'amount' : IDL.Int64,
    'reason' : IDL.Text,
  });
  const UserRewardsResponse = IDL.Record({
    'principal' : IDL.Principal,
    'last_claim_date' : IDL.Opt(IDL.Nat64),
    'consecutive_daily_logins' : IDL.Nat64,
    'points_history' : IDL.Vec(PointsTransaction),
    'points' : IDL.Nat64,
  });
  const Result_23 = IDL.Variant({
    'Ok' : UserRewardsResponse,
    'Err' : SquareError,
  });
  const Result_24 = IDL.Variant({
    'Ok' : IDL.Vec(IDL.Principal),
    'Err' : IDL.Text,
  });
  const ContentModerationRequest = IDL.Record({
    'status' : ContentStatus,
    'content_id' : IDL.Text,
    'content_type' : ParentType,
    'reason' : IDL.Text,
  });
  const RegisterUserRequest = IDL.Record({
    'bio' : IDL.Text,
    'username' : IDL.Text,
    'handle' : IDL.Text,
    'social_links' : IDL.Vec(IDL.Tuple(IDL.Text, IDL.Text)),
    'avatar' : IDL.Text,
  });
  const ReportReason = IDL.Variant({
    'Violence' : IDL.Null,
    'Scam' : IDL.Null,
    'Spam' : IDL.Null,
    'Harassment' : IDL.Null,
    'Other' : IDL.Null,
    'FalseInformation' : IDL.Null,
    'IllegalContent' : IDL.Null,
  });
  const ReportContentRequest = IDL.Record({
    'content_id' : IDL.Text,
    'content_type' : ParentType,
    'description' : IDL.Opt(IDL.Text),
    'reason' : ReportReason,
  });
  const SearchRequest = IDL.Record({
    'pagination' : PaginationParams,
    'query' : IDL.Text,
    'content_types' : IDL.Vec(ParentType),
  });
  const SearchResultResponse = IDL.Record({
    'content_id' : IDL.Text,
    'content_type' : ParentType,
    'relevance_score' : IDL.Float64,
    'snippet' : IDL.Text,
    'created_at' : IDL.Nat64,
    'author' : IDL.Principal,
    'author_username' : IDL.Text,
    'likes_count' : IDL.Nat64,
  });
  const Result_25 = IDL.Variant({
    'Ok' : IDL.Vec(SearchResultResponse),
    'Err' : SquareError,
  });
  const UpdateArticleRequest = IDL.Record({
    'id' : IDL.Text,
    'content' : IDL.Text,
    'hashtags' : IDL.Opt(IDL.Vec(IDL.Text)),
    'media_urls' : IDL.Opt(IDL.Vec(IDL.Text)),
    'token_mentions' : IDL.Opt(IDL.Vec(IDL.Text)),
    'visibility' : IDL.Opt(ContentVisibility),
  });
  const UpdateCommentRequest = IDL.Record({
    'id' : IDL.Text,
    'content' : IDL.Text,
  });
  const UpdateCyclesThresholdRequest = IDL.Record({
    'critical_threshold' : IDL.Opt(IDL.Nat64),
    'warning_threshold' : IDL.Opt(IDL.Nat64),
    'notification_enabled' : IDL.Opt(IDL.Bool),
  });
  const UpdatePostRequest = IDL.Record({
    'id' : IDL.Text,
    'content' : IDL.Text,
    'hashtags' : IDL.Opt(IDL.Vec(IDL.Text)),
    'media_urls' : IDL.Opt(IDL.Vec(IDL.Text)),
    'tags' : IDL.Opt(IDL.Vec(IDL.Text)),
    'news_reference' : IDL.Opt(NewsReference),
    'token_mentions' : IDL.Opt(IDL.Vec(IDL.Text)),
    'visibility' : IDL.Opt(ContentVisibility),
  });
  const UpdateTaskRequest = IDL.Record({
    'id' : IDL.Text,
    'title' : IDL.Opt(IDL.Text),
    'points_reward' : IDL.Opt(IDL.Nat64),
    'description' : IDL.Opt(IDL.Text),
    'end_time' : IDL.Opt(IDL.Nat64),
    'start_time' : IDL.Opt(IDL.Nat64),
    'requirements' : IDL.Opt(TaskRequirements),
  });
  const UpdateProfileRequest = IDL.Record({
    'bio' : IDL.Text,
    'username' : IDL.Text,
    'handle' : IDL.Opt(IDL.Text),
    'social_links' : IDL.Opt(IDL.Vec(IDL.Tuple(IDL.Text, IDL.Text))),
    'avatar' : IDL.Text,
  });
  return IDL.Service({
    'acknowledge_notification' : IDL.Func([IDL.Nat64], [Result], []),
    'add_manager' : IDL.Func([IDL.Principal], [Result_1], []),
    'award_points' : IDL.Func([AwardPointsRequest], [Result], []),
    'complete_task' : IDL.Func([CompleteTaskRequest], [Result_2], []),
    'create_article' : IDL.Func([CreateArticleRequest], [Result_3], []),
    'create_comment' : IDL.Func([CreateCommentRequest], [Result_4], []),
    'create_post' : IDL.Func([CreatePostRequest], [Result_5], []),
    'create_task' : IDL.Func([CreateTaskRequest], [Result_6], []),
    'delete_article' : IDL.Func([IDL.Text], [Result], []),
    'delete_comment' : IDL.Func([IDL.Text], [Result], []),
    'delete_post' : IDL.Func([IDL.Text], [Result], []),
    'delete_task' : IDL.Func([IDL.Text], [Result], []),
    'discover_content' : IDL.Func(
        [DiscoverContentRequest],
        [Result_7],
        ['query'],
      ),
    'follow_user' : IDL.Func([IDL.Principal], [Result], []),
    'get_article' : IDL.Func([IDL.Text], [Result_3], ['query']),
    'get_available_tasks' : IDL.Func([], [Result_8], ['query']),
    'get_comment' : IDL.Func([IDL.Text], [Result_4], ['query']),
    'get_comments' : IDL.Func(
        [IDL.Text, IDL.Text, PaginationParams],
        [Result_9],
        ['query'],
      ),
    'get_cycles_balance' : IDL.Func([], [Result_10], ['query']),
    'get_cycles_consumption_history' : IDL.Func([], [Result_11], ['query']),
    'get_cycles_notifications' : IDL.Func([], [Result_12], ['query']),
    'get_cycles_threshold' : IDL.Func([], [Result_13], ['query']),
    'get_followers' : IDL.Func(
        [IDL.Opt(IDL.Principal)],
        [Result_14],
        ['query'],
      ),
    'get_following' : IDL.Func(
        [IDL.Opt(IDL.Principal)],
        [Result_14],
        ['query'],
      ),
    'get_hot_tags' : IDL.Func([GetHotTagsRequest], [Result_15], ['query']),
    'get_likes' : IDL.Func([IDL.Text, ParentType], [Result_16], ['query']),
    'get_notification_settings' : IDL.Func([], [Result_17], ['query']),
    'get_personalized_recommendations' : IDL.Func(
        [PersonalizedRecommendationsRequest],
        [Result_7],
        ['query'],
      ),
    'get_post' : IDL.Func([IDL.Text], [Result_5], ['query']),
    'get_posts' : IDL.Func([PaginationParams], [Result_18], ['query']),
    'get_shares' : IDL.Func([IDL.Text, ParentType], [Result_19], ['query']),
    'get_trending_topics' : IDL.Func(
        [GetTrendingTopicsRequest],
        [Result_20],
        ['query'],
      ),
    'get_user_content' : IDL.Func(
        [IDL.Opt(IDL.Principal), IDL.Opt(ParentType), PaginationParams],
        [Result_7],
        ['query'],
      ),
    'get_user_leaderboard' : IDL.Func(
        [PaginationParams],
        [Result_21],
        ['query'],
      ),
    'get_user_profile' : IDL.Func(
        [IDL.Opt(IDL.Principal)],
        [Result_22],
        ['query'],
      ),
    'get_user_rewards' : IDL.Func([], [Result_23], ['query']),
    'like_content' : IDL.Func([IDL.Text, ParentType], [Result], []),
    'list_managers' : IDL.Func([], [Result_24], ['query']),
    'migrate_to_sharded_storage' : IDL.Func([], [Result_6], []),
    'moderate_content' : IDL.Func([ContentModerationRequest], [Result], []),
    'register_user' : IDL.Func([RegisterUserRequest], [Result], []),
    'remove_manager' : IDL.Func([IDL.Principal], [Result_1], []),
    'report_content' : IDL.Func([ReportContentRequest], [Result], []),
    'search_content' : IDL.Func([SearchRequest], [Result_25], ['query']),
    'share_content' : IDL.Func([IDL.Text, ParentType], [Result], []),
    'unfollow_user' : IDL.Func([IDL.Principal], [Result], []),
    'unlike_content' : IDL.Func([IDL.Text, ParentType], [Result], []),
    'update_article' : IDL.Func([UpdateArticleRequest], [Result_3], []),
    'update_comment' : IDL.Func([UpdateCommentRequest], [Result_4], []),
    'update_cycles_threshold' : IDL.Func(
        [UpdateCyclesThresholdRequest],
        [Result],
        [],
      ),
    'update_notification_settings' : IDL.Func(
        [IDL.Opt(IDL.Bool), IDL.Opt(IDL.Text), IDL.Opt(IDL.Nat64)],
        [Result],
        [],
      ),
    'update_post' : IDL.Func([UpdatePostRequest], [Result_5], []),
    'update_task' : IDL.Func([UpdateTaskRequest], [Result], []),
    'update_user_profile' : IDL.Func([UpdateProfileRequest], [Result], []),
  });
};
export const init = ({ IDL }) => { return []; };
