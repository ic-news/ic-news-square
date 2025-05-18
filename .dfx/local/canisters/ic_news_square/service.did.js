export const idlFactory = ({ IDL }) => {
  const CommentResponse = IDL.Rec();
  const Value = IDL.Rec();
  const ErrorSeverity = IDL.Variant({
    'Error' : IDL.Null,
    'Info' : IDL.Null,
    'Critical' : IDL.Null,
    'Warning' : IDL.Null,
  });
  const ErrorContext = IDL.Record({
    'function' : IDL.Text,
    'timestamp' : IDL.Nat64,
    'details' : IDL.Opt(IDL.Text),
    'entity_id' : IDL.Opt(IDL.Text),
    'severity' : ErrorSeverity,
    'module' : IDL.Text,
  });
  const ErrorCode = IDL.Variant({
    'MissingRequiredField' : IDL.Null,
    'ValidationFailed' : IDL.Null,
    'ResourceAlreadyExists' : IDL.Null,
    'DataCorruption' : IDL.Null,
    'ResourceNotAvailable' : IDL.Null,
    'AuthForbidden' : IDL.Null,
    'InvalidInput' : IDL.Null,
    'OperationFailed' : IDL.Null,
    'InvalidFormat' : IDL.Null,
    'DataInconsistency' : IDL.Null,
    'DependencyFailed' : IDL.Null,
    'SystemError' : IDL.Null,
    'DataLoss' : IDL.Null,
    'OperationTimeout' : IDL.Null,
    'ContentTooLong' : IDL.Null,
    'NotFound' : IDL.Null,
    'PermissionDenied' : IDL.Null,
    'OperationCancelled' : IDL.Null,
    'InvalidData' : IDL.Null,
    'InvalidCredentials' : IDL.Null,
    'Unauthorized' : IDL.Null,
    'AlreadyExists' : IDL.Null,
    'UnexpectedError' : IDL.Null,
    'RateLimitExceeded' : IDL.Null,
    'ServiceUnavailable' : IDL.Null,
    'ResourceUnavailable' : IDL.Null,
    'InsufficientPermissions' : IDL.Null,
    'ResourceNotFound' : IDL.Null,
    'ResourceExhausted' : IDL.Null,
    'ValidationInvalidInput' : IDL.Null,
    'AuthUnauthorized' : IDL.Null,
    'Forbidden' : IDL.Null,
    'SessionExpired' : IDL.Null,
    'InvalidOperation' : IDL.Null,
    'QuotaExceeded' : IDL.Null,
    'ServiceError' : IDL.Null,
    'ServiceTimeout' : IDL.Null,
  });
  const SquareErrorEnhanced = IDL.Record({
    'recoverable' : IDL.Bool,
    'context' : ErrorContext,
    'code' : ErrorCode,
    'message' : IDL.Text,
    'recovery_hint' : IDL.Opt(IDL.Text),
  });
  const SquareError = IDL.Variant({
    'Enhanced' : SquareErrorEnhanced,
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
    'principal' : IDL.Principal,
    'reference_id' : IDL.Opt(IDL.Text),
    'points' : IDL.Nat64,
    'reason' : IDL.Text,
  });
  const CompleteTaskRequest = IDL.Record({
    'task_id' : IDL.Text,
    'proof' : IDL.Opt(IDL.Text),
  });
  const TaskCompletionResponse = IDL.Record({
    'total_points' : IDL.Nat64,
    'message' : IDL.Text,
    'success' : IDL.Bool,
    'points_earned' : IDL.Nat64,
  });
  const Result_2 = IDL.Variant({
    'Ok' : TaskCompletionResponse,
    'Err' : SquareError,
  });
  const ParentType = IDL.Variant({ 'Post' : IDL.Null, 'Comment' : IDL.Null });
  const CreateCommentRequest = IDL.Record({
    'id' : IDL.Opt(IDL.Text),
    'content' : IDL.Text,
    'parent_id' : IDL.Text,
    'parent_type' : ParentType,
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
    'is_following' : IDL.Bool,
    'principal' : IDL.Principal,
    'username' : IDL.Text,
    'interests' : IDL.Vec(IDL.Text),
    'following_count' : IDL.Nat64,
    'is_followed_by_caller' : IDL.Bool,
    'handle' : IDL.Text,
    'followers_count' : IDL.Nat64,
    'avatar' : IDL.Text,
  });
  const ContentVisibility_1 = IDL.Variant({
    'Private' : IDL.Null,
    'FollowersOnly' : IDL.Null,
    'Public' : IDL.Null,
  });
  CommentResponse.fill(
    IDL.Record({
      'id' : IDL.Text,
      'status' : ContentStatus,
      'updated_at' : IDL.Nat64,
      'content' : IDL.Text,
      'child_comments' : IDL.Vec(CommentResponse),
      'author_info' : UserSocialResponse,
      'created_at' : IDL.Nat64,
      'author' : IDL.Principal,
      'parent_id' : IDL.Text,
      'is_liked' : IDL.Bool,
      'comments_count' : IDL.Nat64,
      'visibility' : ContentVisibility_1,
      'likes_count' : IDL.Nat64,
      'parent_type' : ParentType,
    })
  );
  const Result_3 = IDL.Variant({ 'Ok' : CommentResponse, 'Err' : SquareError });
  const NewsReference = IDL.Record({
    'metadata' : IDL.Vec(IDL.Tuple(IDL.Text, IDL.Text)),
    'canister_id' : IDL.Principal,
  });
  const ContentVisibility = IDL.Variant({
    'Private' : IDL.Null,
    'FollowersOnly' : IDL.Null,
    'Public' : IDL.Null,
  });
  const CreatePostRequest = IDL.Record({
    'id' : IDL.Opt(IDL.Text),
    'is_nsfw' : IDL.Opt(IDL.Bool),
    'content' : IDL.Text,
    'hashtags' : IDL.Vec(IDL.Text),
    'media_urls' : IDL.Vec(IDL.Text),
    'tags' : IDL.Opt(IDL.Vec(IDL.Text)),
    'news_reference' : IDL.Opt(NewsReference),
    'token_mentions' : IDL.Opt(IDL.Vec(IDL.Text)),
    'mentions' : IDL.Opt(IDL.Vec(IDL.Text)),
    'visibility' : IDL.Opt(ContentVisibility),
  });
  const PostResponse = IDL.Record({
    'id' : IDL.Text,
    'status' : ContentStatus,
    'updated_at' : IDL.Nat64,
    'content' : IDL.Text,
    'author_info' : UserSocialResponse,
    'hashtags' : IDL.Vec(IDL.Text),
    'media_urls' : IDL.Vec(IDL.Text),
    'tags' : IDL.Vec(IDL.Text),
    'news_reference' : IDL.Opt(NewsReference),
    'created_at' : IDL.Nat64,
    'author' : IDL.Principal,
    'token_mentions' : IDL.Vec(IDL.Text),
    'comments_count' : IDL.Nat64,
    'visibility' : ContentVisibility_1,
    'likes_count' : IDL.Nat64,
  });
  const ApiError = IDL.Record({
    'recoverable' : IDL.Bool,
    'code' : IDL.Nat32,
    'message' : IDL.Text,
    'details' : IDL.Opt(IDL.Text),
    'recovery_hint' : IDL.Opt(IDL.Text),
  });
  const ApiResponse = IDL.Record({
    'data' : IDL.Opt(PostResponse),
    'error' : IDL.Opt(ApiError),
    'success' : IDL.Bool,
  });
  const TaskType = IDL.Variant({
    'OneTime' : IDL.Null,
    'Weekly' : IDL.Null,
    'Daily' : IDL.Null,
    'Monthly' : IDL.Null,
    'Special' : IDL.Null,
  });
  const SocialInteractionRequirement = IDL.Record({
    'like_count' : IDL.Opt(IDL.Nat64),
    'follow_count' : IDL.Opt(IDL.Nat64),
  });
  const LoginStreakRequirement = IDL.Record({ 'days_required' : IDL.Nat64 });
  const ContentCreationRequirement = IDL.Record({
    'comment_count' : IDL.Opt(IDL.Nat64),
    'post_count' : IDL.Opt(IDL.Nat64),
    'required_hashtags' : IDL.Opt(IDL.Vec(IDL.Text)),
  });
  const TaskRequirements = IDL.Record({
    'social_interaction' : IDL.Opt(SocialInteractionRequirement),
    'required_tokens' : IDL.Opt(IDL.Vec(IDL.Text)),
    'required_nfts' : IDL.Opt(IDL.Vec(IDL.Text)),
    'login_streak' : IDL.Opt(LoginStreakRequirement),
    'custom_requirements' : IDL.Opt(IDL.Vec(IDL.Text)),
    'content_creation' : IDL.Opt(ContentCreationRequirement),
  });
  const CreateTaskRequest = IDL.Record({
    'id' : IDL.Text,
    'title' : IDL.Text,
    'points_reward' : IDL.Nat64,
    'canister_id' : IDL.Principal,
    'description' : IDL.Text,
    'end_time' : IDL.Opt(IDL.Nat64),
    'completion_criteria' : IDL.Text,
    'start_time' : IDL.Opt(IDL.Nat64),
    'task_type' : TaskType,
    'requirements' : IDL.Opt(TaskRequirements),
  });
  const Result_4 = IDL.Variant({ 'Ok' : IDL.Text, 'Err' : SquareError });
  const ApiResponse_1 = IDL.Record({
    'data' : IDL.Opt(IDL.Bool),
    'error' : IDL.Opt(ApiError),
    'success' : IDL.Bool,
  });
  const ApiResponse_2 = IDL.Record({
    'data' : IDL.Opt(IDL.Text),
    'error' : IDL.Opt(ApiError),
    'success' : IDL.Bool,
  });
  const ApiResponse_3 = IDL.Record({
    'data' : IDL.Opt(IDL.Vec(IDL.Tuple(IDL.Text, IDL.Text))),
    'error' : IDL.Opt(ApiError),
    'success' : IDL.Bool,
  });
  const SortOption = IDL.Variant({
    'MostCommented' : IDL.Null,
    'Trending' : IDL.Null,
    'MostLiked' : IDL.Null,
    'Latest' : IDL.Null,
  });
  const PaginationParams = IDL.Record({
    'offset' : IDL.Opt(IDL.Nat64),
    'limit' : IDL.Opt(IDL.Nat64),
  });
  const ContentFilter = IDL.Record({
    'hashtag' : IDL.Opt(IDL.Text),
    'token_mention' : IDL.Opt(IDL.Text),
    'created_after' : IDL.Opt(IDL.Nat64),
    'author' : IDL.Opt(IDL.Principal),
    'created_before' : IDL.Opt(IDL.Nat64),
  });
  const DiscoverContentRequest = IDL.Record({
    'sort_by' : IDL.Opt(SortOption),
    'pagination' : PaginationParams,
    'tags' : IDL.Opt(IDL.Vec(IDL.Text)),
    'filter' : IDL.Opt(ContentFilter),
    'content_types' : IDL.Opt(IDL.Vec(ParentType)),
  });
  const FeedResponse = IDL.Record({
    'total' : IDL.Nat64,
    'comments' : IDL.Vec(CommentResponse),
    'posts' : IDL.Vec(PostResponse),
    'next_offset' : IDL.Nat64,
    'has_more' : IDL.Bool,
  });
  const Result_5 = IDL.Variant({ 'Ok' : FeedResponse, 'Err' : SquareError });
  const ApiResponse_4 = IDL.Record({
    'data' : IDL.Opt(IDL.Null),
    'error' : IDL.Opt(ApiError),
    'success' : IDL.Bool,
  });
  const TaskResponse = IDL.Record({
    'id' : IDL.Text,
    'title' : IDL.Text,
    'description' : IDL.Text,
    'created_at' : IDL.Nat64,
    'completion_criteria' : IDL.Text,
    'is_completed' : IDL.Bool,
    'task_type' : TaskType,
    'expiration_time' : IDL.Opt(IDL.Nat64),
    'points' : IDL.Nat64,
  });
  const Result_6 = IDL.Variant({
    'Ok' : IDL.Vec(TaskResponse),
    'Err' : SquareError,
  });
  const CommentsResponse = IDL.Record({
    'total' : IDL.Nat64,
    'comments' : IDL.Vec(CommentResponse),
    'next_offset' : IDL.Nat64,
    'has_more' : IDL.Bool,
  });
  const Result_7 = IDL.Variant({
    'Ok' : CommentsResponse,
    'Err' : SquareError,
  });
  const CyclesBalanceResponse = IDL.Record({
    'estimated_days_remaining' : IDL.Nat64,
    'threshold_warning' : IDL.Bool,
    'balance' : IDL.Nat64,
    'balance_in_trillion' : IDL.Float64,
  });
  const Result_8 = IDL.Variant({
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
  const Result_9 = IDL.Variant({
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
  const ApiResponse_5 = IDL.Record({
    'data' : IDL.Opt(CyclesNotificationsResponse),
    'error' : IDL.Opt(ApiError),
    'success' : IDL.Bool,
  });
  const CyclesThresholdConfig = IDL.Record({
    'critical_threshold' : IDL.Nat64,
    'warning_threshold' : IDL.Nat64,
    'notification_enabled' : IDL.Bool,
  });
  const ApiResponse_6 = IDL.Record({
    'data' : IDL.Opt(CyclesThresholdConfig),
    'error' : IDL.Opt(ApiError),
    'success' : IDL.Bool,
  });
  const ApiResponse_7 = IDL.Record({
    'data' : IDL.Opt(IDL.Vec(IDL.Text)),
    'error' : IDL.Opt(ApiError),
    'success' : IDL.Bool,
  });
  const ApiResponse_8 = IDL.Record({
    'data' : IDL.Opt(
      IDL.Vec(IDL.Tuple(ErrorCode, IDL.Nat64, IDL.Nat64, IDL.Nat64))
    ),
    'error' : IDL.Opt(ApiError),
    'success' : IDL.Bool,
  });
  const ApiResponse_9 = IDL.Record({
    'data' : IDL.Opt(IDL.Vec(UserSocialResponse)),
    'error' : IDL.Opt(ApiError),
    'success' : IDL.Bool,
  });
  const HeartbeatIntervalResponse = IDL.Record({
    'interval_hours' : IDL.Nat64,
  });
  const ApiResponse_10 = IDL.Record({
    'data' : IDL.Opt(HeartbeatIntervalResponse),
    'error' : IDL.Opt(ApiError),
    'success' : IDL.Bool,
  });
  const TagType = IDL.Variant({
    'Custom' : IDL.Null,
    'Category' : IDL.Null,
    'Topic' : IDL.Null,
    'Location' : IDL.Null,
  });
  const GetHotTagsRequest = IDL.Record({
    'limit' : IDL.Opt(IDL.Nat32),
    'tag_type' : IDL.Opt(TagType),
  });
  const HotTagInfo = IDL.Record({
    'name' : IDL.Text,
    'count' : IDL.Nat64,
    'tag_type' : TagType,
  });
  const HotTagsResponse = IDL.Record({
    'updated_at' : IDL.Nat64,
    'tags' : IDL.Vec(HotTagInfo),
  });
  const Result_10 = IDL.Variant({
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
  const Result_11 = IDL.Variant({ 'Ok' : LikesResponse, 'Err' : SquareError });
  const LogEntry = IDL.Record({
    'message' : IDL.Text,
    'timestamp' : IDL.Nat64,
  });
  const ApiResponse_11 = IDL.Record({
    'data' : IDL.Opt(IDL.Vec(IDL.Tuple(ErrorCode, IDL.Nat64))),
    'error' : IDL.Opt(ApiError),
    'success' : IDL.Bool,
  });
  const Result_12 = IDL.Variant({ 'Ok' : IDL.Bool, 'Err' : SquareError });
  const PersonalizedRecommendationsRequest = IDL.Record({
    'diversity_factor' : IDL.Opt(IDL.Float64),
    'recency_weight' : IDL.Opt(IDL.Float64),
    'include_followed_topics' : IDL.Opt(IDL.Bool),
    'pagination' : PaginationParams,
    'include_followed_users' : IDL.Opt(IDL.Bool),
    'include_trending' : IDL.Opt(IDL.Bool),
    'include_similar_to_liked' : IDL.Opt(IDL.Bool),
    'content_types' : IDL.Opt(IDL.Vec(ParentType)),
  });
  const Result_13 = IDL.Variant({ 'Ok' : PostResponse, 'Err' : SquareError });
  const PostsResponse = IDL.Record({
    'total' : IDL.Nat64,
    'posts' : IDL.Vec(PostResponse),
    'next_offset' : IDL.Nat64,
  });
  const Result_14 = IDL.Variant({ 'Ok' : PostsResponse, 'Err' : SquareError });
  const GetTrendingTopicsRequest = IDL.Record({
    'limit' : IDL.Opt(IDL.Nat32),
    'time_range_hours' : IDL.Opt(IDL.Nat32),
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
  const Result_15 = IDL.Variant({
    'Ok' : IDL.Vec(TrendingTopicResponse),
    'Err' : SquareError,
  });
  const UserLeaderboardItem = IDL.Record({
    'principal' : IDL.Principal,
    'username' : IDL.Text,
    'comment_count' : IDL.Nat64,
    'last_claim_date' : IDL.Nat64,
    'consecutive_daily_logins' : IDL.Nat64,
    'like_count' : IDL.Nat64,
    'rank' : IDL.Nat64,
    'post_count' : IDL.Nat64,
    'reputation' : IDL.Nat64,
    'handle' : IDL.Text,
    'followers_count' : IDL.Nat64,
    'avatar' : IDL.Text,
  });
  const UserLeaderboardResponse = IDL.Record({
    'total_users' : IDL.Nat64,
    'users' : IDL.Vec(UserLeaderboardItem),
    'next_offset' : IDL.Nat64,
    'has_more' : IDL.Bool,
  });
  const ApiResponse_12 = IDL.Record({
    'data' : IDL.Opt(UserLeaderboardResponse),
    'error' : IDL.Opt(ApiError),
    'success' : IDL.Bool,
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
  const NotificationPreferences = IDL.Record({
    'follows' : IDL.Bool,
    'likes' : IDL.Bool,
    'comments' : IDL.Bool,
    'mentions' : IDL.Bool,
    'system' : IDL.Bool,
  });
  const InteractionPreferences = IDL.Record({
    'allow_comments' : IDL.Bool,
    'allow_mentions' : IDL.Bool,
    'allow_follows' : IDL.Bool,
    'show_likes' : IDL.Bool,
  });
  const UserPrivacySettings = IDL.Record({
    'notification_preferences' : NotificationPreferences,
    'content_visibility' : ContentVisibility_1,
    'interaction_preferences' : InteractionPreferences,
    'profile_visibility' : ContentVisibility_1,
  });
  const UserProfileResponse = IDL.Record({
    'bio' : IDL.Text,
    'is_following' : IDL.Bool,
    'status' : UserStatus,
    'last_login' : IDL.Nat64,
    'updated_at' : IDL.Nat64,
    'principal' : IDL.Principal,
    'username' : IDL.Text,
    'interests' : IDL.Vec(IDL.Text),
    'role' : UserRole,
    'following_count' : IDL.Nat64,
    'created_at' : IDL.Nat64,
    'handle' : IDL.Text,
    'registered_at' : IDL.Nat64,
    'followers_count' : IDL.Nat64,
    'privacy_settings' : IDL.Opt(UserPrivacySettings),
    'social_links' : IDL.Vec(IDL.Tuple(IDL.Text, IDL.Text)),
    'avatar' : IDL.Text,
  });
  const ApiResponse_13 = IDL.Record({
    'data' : IDL.Opt(UserProfileResponse),
    'error' : IDL.Opt(ApiError),
    'success' : IDL.Bool,
  });
  Value.fill(
    IDL.Variant({
      'Int' : IDL.Int64,
      'Map' : IDL.Vec(IDL.Tuple(IDL.Text, Value)),
      'Nat' : IDL.Nat64,
      'Blob' : IDL.Vec(IDL.Nat8),
      'Bool' : IDL.Bool,
      'Null' : IDL.Null,
      'Text' : IDL.Text,
      'Float' : IDL.Float64,
      'Principal' : IDL.Principal,
      'Array' : IDL.Vec(Value),
    })
  );
  const Result_16 = IDL.Variant({
    'Ok' : IDL.Vec(IDL.Tuple(IDL.Text, Value)),
    'Err' : SquareError,
  });
  const LikeContentRequest = IDL.Record({
    'content_id' : IDL.Text,
    'content_type' : ParentType,
  });
  const Result_17 = IDL.Variant({
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
    'interests' : IDL.Opt(IDL.Vec(IDL.Text)),
    'handle' : IDL.Text,
    'social_links' : IDL.Opt(IDL.Vec(IDL.Tuple(IDL.Text, IDL.Text))),
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
    'content_types' : IDL.Opt(IDL.Vec(ParentType)),
  });
  const SearchResultResponse = IDL.Record({
    'id' : IDL.Text,
    'title' : IDL.Opt(IDL.Text),
    'content_type' : ParentType,
    'relevance_score' : IDL.Float64,
    'snippet' : IDL.Text,
    'created_at' : IDL.Nat64,
    'author' : UserSocialResponse,
  });
  const Result_18 = IDL.Variant({
    'Ok' : IDL.Vec(SearchResultResponse),
    'Err' : SquareError,
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
    'visibility' : IDL.Opt(ContentVisibility_1),
  });
  const UpdateProfileRequest = IDL.Record({
    'bio' : IDL.Opt(IDL.Text),
    'username' : IDL.Opt(IDL.Text),
    'interests' : IDL.Opt(IDL.Vec(IDL.Text)),
    'handle' : IDL.Opt(IDL.Text),
    'privacy_settings' : IDL.Opt(UserPrivacySettings),
    'social_links' : IDL.Opt(IDL.Vec(IDL.Tuple(IDL.Text, IDL.Text))),
    'avatar' : IDL.Opt(IDL.Text),
  });
  return IDL.Service({
    'acknowledge_notification' : IDL.Func([IDL.Nat64], [Result], []),
    'add_manager' : IDL.Func([IDL.Principal], [Result_1], []),
    'award_points' : IDL.Func([AwardPointsRequest], [Result], []),
    'clear_logs' : IDL.Func([], [IDL.Bool], []),
    'complete_task' : IDL.Func([CompleteTaskRequest], [Result_2], []),
    'create_comment' : IDL.Func([CreateCommentRequest], [Result_3], []),
    'create_post' : IDL.Func([CreatePostRequest], [ApiResponse], []),
    'create_task' : IDL.Func([CreateTaskRequest], [Result_4], []),
    'debug_fix_user_data' : IDL.Func([IDL.Text], [ApiResponse_1], []),
    'debug_fix_user_profile' : IDL.Func([IDL.Text], [ApiResponse_2], []),
    'debug_list_all_users' : IDL.Func([], [ApiResponse_3], ['query']),
    'delete_comment' : IDL.Func([IDL.Text], [Result], []),
    'delete_post' : IDL.Func([IDL.Text], [Result], []),
    'delete_task' : IDL.Func([IDL.Text], [Result], []),
    'discover_content' : IDL.Func(
        [DiscoverContentRequest],
        [Result_5],
        ['query'],
      ),
    'follow_user' : IDL.Func([IDL.Principal], [ApiResponse_4], []),
    'get_available_tasks' : IDL.Func([], [Result_6], ['query']),
    'get_comment' : IDL.Func([IDL.Text], [Result_3], ['query']),
    'get_comments' : IDL.Func(
        [IDL.Text, IDL.Text, PaginationParams],
        [Result_7],
        ['query'],
      ),
    'get_cycles_balance' : IDL.Func([], [Result_8], ['query']),
    'get_cycles_consumption_history' : IDL.Func([], [Result_9], ['query']),
    'get_cycles_notifications' : IDL.Func([], [ApiResponse_5], ['query']),
    'get_cycles_threshold' : IDL.Func([], [ApiResponse_6], ['query']),
    'get_error_history' : IDL.Func([], [ApiResponse_7], ['query']),
    'get_error_stats' : IDL.Func([], [ApiResponse_8], ['query']),
    'get_followers' : IDL.Func([IDL.Opt(IDL.Text)], [ApiResponse_9], ['query']),
    'get_following' : IDL.Func([IDL.Opt(IDL.Text)], [ApiResponse_9], ['query']),
    'get_heartbeat_interval' : IDL.Func([], [ApiResponse_10], ['query']),
    'get_hot_tags' : IDL.Func([GetHotTagsRequest], [Result_10], ['query']),
    'get_likes' : IDL.Func([IDL.Text, ParentType], [Result_11], ['query']),
    'get_logs' : IDL.Func([], [IDL.Vec(LogEntry)], ['query']),
    'get_most_common_errors' : IDL.Func(
        [IDL.Nat64],
        [ApiResponse_11],
        ['query'],
      ),
    'get_notification_settings' : IDL.Func([], [Result_12], ['query']),
    'get_personalized_recommendations' : IDL.Func(
        [PersonalizedRecommendationsRequest],
        [Result_5],
        ['query'],
      ),
    'get_post' : IDL.Func([IDL.Text], [Result_13], ['query']),
    'get_posts' : IDL.Func([PaginationParams], [Result_14], ['query']),
    'get_recent_logs' : IDL.Func([IDL.Nat64], [IDL.Vec(LogEntry)], ['query']),
    'get_trending_topics' : IDL.Func(
        [GetTrendingTopicsRequest],
        [Result_15],
        ['query'],
      ),
    'get_user_content' : IDL.Func(
        [IDL.Opt(IDL.Text), IDL.Opt(ParentType), PaginationParams],
        [Result_5],
        ['query'],
      ),
    'get_user_leaderboard' : IDL.Func(
        [PaginationParams],
        [ApiResponse_12],
        ['query'],
      ),
    'get_user_profile' : IDL.Func(
        [IDL.Opt(IDL.Text)],
        [ApiResponse_13],
        ['query'],
      ),
    'get_user_rewards' : IDL.Func([], [Result_16], ['query']),
    'like_content' : IDL.Func([LikeContentRequest], [Result], []),
    'list_managers' : IDL.Func([], [Result_17], ['query']),
    'migrate_storage' : IDL.Func([], [ApiResponse_2], []),
    'moderate_content' : IDL.Func([ContentModerationRequest], [Result], []),
    'register_user' : IDL.Func([RegisterUserRequest], [ApiResponse_4], []),
    'remove_manager' : IDL.Func([IDL.Principal], [Result_1], []),
    'report_content' : IDL.Func([ReportContentRequest], [Result], []),
    'search_content' : IDL.Func([SearchRequest], [Result_18], ['query']),
    'unfollow_user' : IDL.Func([IDL.Principal], [ApiResponse_4], []),
    'unlike_content' : IDL.Func([LikeContentRequest], [Result], []),
    'update_comment' : IDL.Func([UpdateCommentRequest], [Result_3], []),
    'update_cycles_threshold' : IDL.Func(
        [UpdateCyclesThresholdRequest],
        [ApiResponse_6],
        [],
      ),
    'update_heartbeat_interval' : IDL.Func(
        [HeartbeatIntervalResponse],
        [ApiResponse_10],
        [],
      ),
    'update_notification_settings' : IDL.Func(
        [IDL.Opt(IDL.Bool)],
        [Result],
        [],
      ),
    'update_post' : IDL.Func([UpdatePostRequest], [ApiResponse], []),
    'update_task' : IDL.Func([CreateTaskRequest], [Result], []),
    'update_user_profile' : IDL.Func(
        [UpdateProfileRequest],
        [ApiResponse_2],
        [],
      ),
  });
};
export const init = ({ IDL }) => { return []; };
