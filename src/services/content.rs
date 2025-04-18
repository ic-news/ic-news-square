use candid::Principal;
use ic_cdk::api::time;

use crate::auth::is_admin;
use crate::models::content::*;
use crate::models::display::{FeedResponse, ContentDetailResponse};
use crate::models::error::{SquareError, SquareResult};
use crate::storage::{STORAGE, Post, Article, Comment, ContentStatus, ParentType, ContentVisibility};
// use crate::storage::sharded_ops::{insert_post_sharded};
use crate::utils::error_handler::{content_too_long_error, not_found_error, unauthorized_error, log_and_return, validation_error};

// Post management functions
pub fn create_post(request: CreatePostRequest, caller: Principal) -> SquareResult<PostResponse> {
    const MODULE: &str = "services::content";
    const FUNCTION: &str = "create_post";
    
    // Validate content length
    if request.content.len() > MAX_POST_LENGTH {
        return log_and_return(content_too_long_error(
            "Post", 
            MAX_POST_LENGTH, 
            request.content.len(), 
            MODULE, 
            FUNCTION
        ));
    }
    
    // Validate hashtags count
    if request.hashtags.len() > MAX_HASHTAGS {
        return log_and_return(validation_error(
            &format!("Too many hashtags. Maximum allowed is {}", MAX_HASHTAGS),
            MODULE,
            FUNCTION
        ));
    }
    
    // Validate token mentions count
    if let Some(token_mentions) = &request.token_mentions {
        if token_mentions.len() > MAX_TOKEN_MENTIONS {
            return log_and_return(validation_error(
                &format!("Too many token mentions. Maximum allowed is {}", MAX_TOKEN_MENTIONS),
                MODULE,
                FUNCTION
            ));
        }
    }
    
    // Validate media URLs count
    if request.media_urls.len() > MAX_MEDIA_URLS {
        return log_and_return(validation_error(
            &format!("Too many media URLs. Maximum allowed is {}", MAX_MEDIA_URLS),
            MODULE,
            FUNCTION
        ));
    }
    
    // Validate tags count
    if let Some(tags) = &request.tags {
        if tags.len() > 5 {
            return log_and_return(validation_error(
                "Too many tags. Maximum allowed is 5",
                MODULE,
                FUNCTION
            ));
        }
    }
    
    // Create post
    // First create the post and get the necessary information, but not the user information
    let post = STORAGE.with(|storage| {
        let mut storage = storage.borrow_mut();
        
        // Use provided ID or generate a new one
        let id = if let Some(custom_id) = &request.id {
            custom_id.clone()
        } else {
            // Generate unique ID
            let content_counter = storage.content_counter + 1;
            storage.content_counter = content_counter;
            format!("post_{}", content_counter)
        };
        
        // Create post object
        let post = Post {
            id: id.clone(),
            author: caller,
            content: request.content.clone(),
            media_urls: request.media_urls.clone(),
            hashtags: request.hashtags.clone(),
            token_mentions: request.token_mentions.clone().unwrap_or_default(),
            tags: request.tags.clone().unwrap_or_default(),
            created_at: time() / 1_000_000,
            updated_at: time() / 1_000_000,
            status: ContentStatus::Active,
            visibility: request.visibility.unwrap_or(ContentVisibility::Public),
        };
        
        // Store post in both traditional and sharded storage
        storage.posts.insert(id.clone(), post.clone());
        
        // Also store in sharded storage
        // TODO: Implement sharded storage functionality
        // insert_post_sharded(id.clone(), post.clone());
        
        // Update user's posts list
        let user_posts = storage.user_posts.entry(caller).or_insert_with(Vec::new);
        user_posts.push(id.clone());
        
        // Update user stats
        if let Some(user_stats) = storage.user_stats.get_mut(&caller) {
            user_stats.post_count += 1;
        }
        
        // Update trending topics
        for hashtag in &request.hashtags {
            let count = storage.trending_topics.entry(hashtag.clone()).or_insert(0);
            *count += 1;
        }
        
        post
    });
    
    // After STORAGE borrowing ends, get user information
    let author_info = crate::services::user::get_user_social_info(post.author, None)?;
    
    // Build response
    Ok(PostResponse {
        id: post.id,
        author: post.author,
        content: post.content,
        media_urls: post.media_urls,
        hashtags: post.hashtags,
        token_mentions: post.token_mentions,
        tags: post.tags,
        created_at: post.created_at,
        updated_at: post.updated_at,
        status: post.status,
        visibility: post.visibility,
        likes_count: 0,
        comments_count: 0,
        shares_count: 0,
        author_info,
    })
    .map_err(|e: std::cell::BorrowMutError| SquareError::SystemError(format!("Failed to create post: {}", e)))
}

pub fn get_post(id: String) -> SquareResult<PostResponse> {
    const MODULE: &str = "services::content";
    const FUNCTION: &str = "get_post";
    
    // Get post data and statistics, but don't get user information yet
    let post_data = STORAGE.with(|storage| {
        let storage = storage.borrow();
        
        let post = storage.posts.get(&id)
            .ok_or_else(|| not_found_error("Post", &id, MODULE, FUNCTION))?;
            
        // Check if post is active
        if post.status != ContentStatus::Active {
            return log_and_return(not_found_error("Post", &id, MODULE, FUNCTION)
                .with_details("Post is not active"));
        }
        
        // Count likes
        let likes_count = storage.likes.get(&id)
            .map(|likes| likes.len() as u64)
            .unwrap_or(0);
            
        // Count comments
        let comments_count = storage.comments.values()
            .filter(|comment| 
                comment.parent_id == id && 
                comment.parent_type == ParentType::Post &&
                comment.status == ContentStatus::Active
            )
            .count() as u64;
            
        // Get shares count
        let shares_count = storage.shares.get(&id).copied().unwrap_or(0);
        
        // Clone post data for use after STORAGE borrowing ends
        let post_id = post.id.clone();
        let post_author = post.author;
        let post_content = post.content.clone();
        let post_media_urls = post.media_urls.clone();
        let post_hashtags = post.hashtags.clone();
        let post_token_mentions = post.token_mentions.clone();
        let post_tags = post.tags.clone();
        let post_created_at = post.created_at;
        let post_updated_at = post.updated_at;
        let post_status = post.status.clone();
        let post_visibility = post.visibility.clone();
        
        Ok((post_id, post_author, post_content, post_media_urls, post_hashtags, 
            post_token_mentions, post_tags, post_created_at, post_updated_at, 
            post_status, post_visibility, likes_count, comments_count, shares_count))
    })?;
    
    // Destructure the retrieved data
    let (post_id, post_author, post_content, post_media_urls, post_hashtags, 
         post_token_mentions, post_tags, post_created_at, post_updated_at, 
         post_status, post_visibility, likes_count, comments_count, shares_count) = post_data;
    
    // After STORAGE borrowing ends, get user information
    let author_info = crate::services::user::get_user_social_info(post_author, None)?;
    
    // Build response
    Ok(PostResponse {
        id: post_id,
        author: post_author,
        content: post_content,
        media_urls: post_media_urls,
        hashtags: post_hashtags,
        token_mentions: post_token_mentions,
        tags: post_tags,
        created_at: post_created_at,
        updated_at: post_updated_at,
        status: post_status,
        visibility: post_visibility,
        likes_count,
        comments_count,
        shares_count,
        author_info,
    })
}

pub fn get_posts(pagination: PaginationParams) -> SquareResult<PostsResponse> {
    const MODULE: &str = "services::content";
    const FUNCTION: &str = "get_posts";
    
    // First collect all necessary post data, but don't get user information yet
    let post_data = STORAGE.with(|storage| {
        let storage = storage.borrow();
        
        // Collect post data and statistics
        let posts_data: Vec<(Post, u64, u64, u64)> = storage.posts.values()
            .filter(|post| post.status == ContentStatus::Active)
            .map(|post| {
                // Calculate likes, comments and shares count for each post
                let likes_count = storage.likes.get(&post.id)
                    .map(|likes| likes.len() as u64)
                    .unwrap_or(0);
                
                let comments_count = storage.comments.values()
                    .filter(|comment| 
                        comment.parent_id == post.id && 
                        comment.parent_type == ParentType::Post &&
                        comment.status == ContentStatus::Active
                    )
                    .count() as u64;
                
                let shares_count = storage.shares.get(&post.id)
                    .copied()
                    .unwrap_or(0);
                
                // Return post and statistics data
                (post.clone(), likes_count, comments_count, shares_count)
            })
            .collect();
        
        posts_data
    });
    
    // After STORAGE borrowing ends, get user info for each post and build PostResponse
    let mut posts: Vec<PostResponse> = Vec::new();
    for (post, likes_count, comments_count, shares_count) in post_data {
        // Get user information
        if let Ok(author_info) = crate::services::user::get_user_social_info(post.author, None) {
            // Build response
            posts.push(PostResponse {
                id: post.id,
                author: post.author,
                content: post.content,
                media_urls: post.media_urls,
                hashtags: post.hashtags,
                token_mentions: post.token_mentions,
                created_at: post.created_at,
                updated_at: post.updated_at,
                status: post.status,
                visibility: post.visibility,
                likes_count,
                tags: post.tags,
                comments_count,
                shares_count,
                author_info,
            });
        }
    }
    
    // Sort by creation time (newest first)
    posts.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    
    // Apply pagination
    let total = posts.len();
    let start = std::cmp::min(pagination.offset, total);
    let end = std::cmp::min(start + pagination.limit, total);
    let posts = posts[start..end].to_vec();
    
    Ok(PostsResponse {
        posts,
        total: total as u64,
        next_offset: if end < total { end } else { total },
    })
}

pub fn update_post(request: UpdatePostRequest, caller: Principal) -> SquareResult<PostResponse> {
    const MODULE: &str = "services::content";
    const FUNCTION: &str = "update_post";
    
    // Get all necessary data, but don't get user information
    let post_data = STORAGE.with(|storage| {
        let mut storage = storage.borrow_mut();
        
        // First check if post exists
        let post_exists = storage.posts.contains_key(&request.id);
        if !post_exists {
            return log_and_return(not_found_error("Post", &request.id, MODULE, FUNCTION));
        }
        
        // Get post info before mutable borrow
        let post_author;
        let is_admin = storage.admin == Some(caller);
        let is_manager = storage.managers.contains(&caller);
        
        {
            let post = storage.posts.get(&request.id).unwrap();
            post_author = post.author;
        }
        
        // Now get mutable reference
        let post = storage.posts.get_mut(&request.id).unwrap();
        
        if post_author != caller && !is_admin && !is_manager {
            return log_and_return(unauthorized_error(
                "Only the author or admin can update this post", 
                MODULE, 
                FUNCTION
            ));
        }
        
        // Update fields if provided
        // Validate content length
        if request.content.len() > MAX_POST_LENGTH {
            return log_and_return(content_too_long_error(
                "Post", 
                MAX_POST_LENGTH, 
                request.content.len(), 
                MODULE, 
                FUNCTION
            ));
        }
        post.content = request.content;
        
        if let Some(media_urls) = request.media_urls {
            if media_urls.len() > MAX_MEDIA_URLS {
                return log_and_return(validation_error(
                    &format!("Too many media URLs. Maximum allowed is {}", MAX_MEDIA_URLS),
                    MODULE,
                    FUNCTION
                ));
            }
            post.media_urls = media_urls;
        }
        
        if let Some(hashtags) = request.hashtags {
            if hashtags.len() > MAX_HASHTAGS {
                return log_and_return(validation_error(
                    &format!("Too many hashtags. Maximum allowed is {}", MAX_HASHTAGS),
                    MODULE,
                    FUNCTION
                ));
            }
            post.hashtags = hashtags;
        }
        
        if let Some(token_mentions) = request.token_mentions {
            if token_mentions.len() > MAX_TOKEN_MENTIONS {
                return log_and_return(validation_error(
                    &format!("Too many token mentions. Maximum allowed is {}", MAX_TOKEN_MENTIONS),
                    MODULE,
                    FUNCTION
                ));
            }
            post.token_mentions = token_mentions;
        }

        if let Some(tags) = request.tags {
            if tags.len() > 5 {
                return log_and_return(validation_error(
                    "Too many tags. Maximum allowed is 5",
                    MODULE,
                    FUNCTION
                ));
            }
            post.tags = tags;
        }
        
        if let Some(visibility) = request.visibility {
            post.visibility = visibility;
        }
        
        // Update timestamp
        post.updated_at = time() / 1_000_000;
        
        // Create response data
        let post_id = post.id.clone();
        let post_author = post.author;
        let post_content = post.content.clone();
        let post_media_urls = post.media_urls.clone();
        let post_hashtags = post.hashtags.clone();
        let post_token_mentions = post.token_mentions.clone();
        let post_created_at = post.created_at;
        let post_updated_at = post.updated_at;
        let post_status = post.status.clone();
        let post_visibility = post.visibility.clone();
        let post_tags = post.tags.clone();
        
        // Count likes
        let likes_count = storage.likes.get(&post_id)
            .map(|likes| likes.len() as u64)
            .unwrap_or(0);
            
        // Count shares
        let shares_count = storage.shares.get(&post_id)
            .map(|shares| *shares)
            .unwrap_or(0);
        
        // Return all necessary data
        Ok((post_id, post_author, post_content, post_media_urls, post_hashtags, 
            post_token_mentions, post_created_at, post_updated_at, post_status, 
            post_tags, post_visibility, likes_count, shares_count))
    })?;
    
    // Destructure the retrieved data
    let (post_id, post_author, post_content, post_media_urls, post_hashtags, 
         post_token_mentions, post_created_at, post_updated_at, post_status, 
         post_tags, post_visibility, likes_count, shares_count) = post_data;
    
    // After STORAGE's mutable borrow ends, get user information
    let author_info = crate::services::user::get_user_social_info(post_author, None)?;
    
    // Return the post response
    Ok(PostResponse {
        id: post_id,
        author: post_author,
        content: post_content,
        media_urls: post_media_urls,
        hashtags: post_hashtags,
        token_mentions: post_token_mentions,
        created_at: post_created_at,
        updated_at: post_updated_at,
        status: post_status,
        visibility: post_visibility,
        likes_count,
        shares_count,
        tags: post_tags,
        comments_count: 0, // We'll need to count comments in a separate function
        author_info,
    })
}

pub fn delete_post(id: String, caller: Principal) -> SquareResult<()> {
    const MODULE: &str = "services::content";
    const FUNCTION: &str = "delete_post";
    
    STORAGE.with(|storage| {
        let mut storage = storage.borrow_mut();
        
        let post = storage.posts.get(&id)
            .ok_or_else(|| not_found_error("Post", &id, MODULE, FUNCTION))?;
            
        // Get author and check ownership
        let post_author = post.author;
        let is_admin = storage.admin == Some(caller);
        let is_manager = storage.managers.contains(&caller);
        
        if post_author != caller && !is_admin && !is_manager {
            return log_and_return(unauthorized_error(
                "Only the author or admin can delete this post",
                MODULE,
                FUNCTION
            ));
        }
        
        // Mark as removed instead of actually deleting
        if let Some(post) = storage.posts.get_mut(&id) {
            post.status = ContentStatus::Removed;
            post.updated_at = time() / 1_000_000;
        }
        
        Ok(())
    })
}

// Article management functions
pub fn create_article(request: CreateArticleRequest, caller: Principal) -> SquareResult<ArticleResponse> {
    const MODULE: &str = "services::content";
    const FUNCTION: &str = "create_article";
    
    // Validate content length
    if request.content.len() > MAX_ARTICLE_LENGTH {
        return log_and_return(content_too_long_error(
            "Article", 
            MAX_ARTICLE_LENGTH, 
            request.content.len(), 
            MODULE, 
            FUNCTION
        ));
    }
    
    // Validate hashtags count
    if request.hashtags.len() > MAX_HASHTAGS {
        return log_and_return(validation_error(
            &format!("Too many hashtags. Maximum allowed is {}", MAX_HASHTAGS),
            MODULE,
            FUNCTION
        ));
    }
    
    // Validate token mentions count
    if let Some(token_mentions) = &request.token_mentions {
        if token_mentions.len() > MAX_TOKEN_MENTIONS {
            return log_and_return(validation_error(
                &format!("Too many token mentions. Maximum allowed is {}", MAX_TOKEN_MENTIONS),
                MODULE,
                FUNCTION
            ));
        }
    }
    
    // Validate media URLs count
    if request.media_urls.len() > MAX_MEDIA_URLS {
        return log_and_return(validation_error(
            &format!("Too many media URLs. Maximum allowed is {}", MAX_MEDIA_URLS),
            MODULE,
            FUNCTION
        ));
    }
    
    // Create article and get article ID
    let article_id = STORAGE.with(|storage| {
        let mut storage = storage.borrow_mut();
        
        // Use provided ID or generate a new one
        let id = if let Some(custom_id) = &request.id {
            custom_id.clone()
        } else {
            // Generate unique ID
            let content_counter = storage.content_counter + 1;
            storage.content_counter = content_counter;
            format!("article_{}", content_counter)
        };
        
        // Create article object
        let article = Article {
            id: id.clone(),
            author: caller,
            content: request.content,
            media_urls: request.media_urls.clone(),
            hashtags: request.hashtags.clone(),
            token_mentions: request.token_mentions.clone().unwrap_or_default(),
            created_at: time() / 1_000_000,
            updated_at: time() / 1_000_000,
            status: ContentStatus::Active,
            visibility: request.visibility.unwrap_or(ContentVisibility::Public),
        };
        
        // Store article
        storage.articles.insert(id.clone(), article);
        
        // Update user's articles list
        let user_articles = storage.user_articles.entry(caller).or_insert_with(Vec::new);
        user_articles.push(id.clone());
        
        // Update user stats
        if let Some(user_stats) = storage.user_stats.get_mut(&caller) {
            user_stats.article_count += 1;
        }
        
        // Update trending topics
        for hashtag in request.hashtags {
            let count = storage.trending_topics.entry(hashtag).or_insert(0);
            *count += 1;
        }
        
        // Return article ID
        id
    });
    
    // After mutable borrowing of STORAGE ends, get article details
    get_article(article_id)
        .map_err(|e| SquareError::SystemError(format!("Failed to create article: {}", e)))
}

// Comment management functions
pub fn create_comment(request: CreateCommentRequest, caller: Principal) -> SquareResult<CommentResponse> {
    const MODULE: &str = "services::content";
    const FUNCTION: &str = "create_comment";
    
    // Validate content length
    if request.content.len() > MAX_COMMENT_LENGTH {
        return log_and_return(validation_error(
            &format!("Comment content exceeds maximum length of {} characters", MAX_COMMENT_LENGTH),
            MODULE,
            FUNCTION
        ));
    }
    
    // Validate parent exists
    STORAGE.with(|storage| {
        let mut storage = storage.borrow_mut();
        
        // Check if parent exists based on parent type
        match request.parent_type {
            ParentType::Post => {
                if !storage.posts.contains_key(&request.parent_id) {
                    return log_and_return(not_found_error(
                        "Post", 
                        &request.parent_id, 
                        MODULE, 
                        FUNCTION
                    ).with_details("Parent post not found"));
                }
            },
            ParentType::Article => {
                if !storage.articles.contains_key(&request.parent_id) {
                    return log_and_return(not_found_error(
                        "Article", 
                        &request.parent_id, 
                        MODULE, 
                        FUNCTION
                    ).with_details("Parent article not found"));
                }
            },
            ParentType::Comment => {
                if !storage.comments.contains_key(&request.parent_id) {
                    return log_and_return(not_found_error(
                        "Comment", 
                        &request.parent_id, 
                        MODULE, 
                        FUNCTION
                    ).with_details("Parent comment not found"));
                }
            },
        }
        
        // Use provided ID or generate a new one
        let id = if let Some(custom_id) = &request.id {
            custom_id.clone()
        } else {
            // Generate unique ID
            let content_counter = storage.content_counter + 1;
            storage.content_counter = content_counter;
            format!("comment_{}", content_counter)
        };
        
        // Create comment object
        let comment = Comment {
            id: id.clone(),
            author: caller,
            content: request.content,
            parent_id: request.parent_id.clone(),
            parent_type: request.parent_type,
            created_at: time() / 1_000_000,
            updated_at: time() / 1_000_000,
            status: ContentStatus::Active,
            child_comments: Vec::new(),
            likes_count: 0,
        };
        
        // Update parent comment if this is a reply to another comment
        if request.parent_type == ParentType::Comment {
            if let Some(parent_comment) = storage.comments.get_mut(&request.parent_id) {
                parent_comment.child_comments.push(id.clone());
            }
        }

        // Store comment
        storage.comments.insert(id.clone(), comment.clone());
        
        // Update user's comments list
        let user_comments = storage.user_comments.entry(caller).or_insert_with(Vec::new);
        user_comments.push(id.clone());
        
        // Update user stats
        if let Some(user_stats) = storage.user_stats.get_mut(&caller) {
            user_stats.comment_count += 1;
        }
        
        // Drop the mutable borrow before getting user info
        drop(storage);
        
        // Get author info
        let author_info = crate::services::user::get_user_social_info(caller, None)?;
        
        // Create comment response
        Ok(CommentResponse {
            id: id.clone(),
            author: caller,
            content: comment.content.clone(),
            parent_id: comment.parent_id.clone(),
            parent_type: comment.parent_type.clone(),
            created_at: comment.created_at,
            updated_at: comment.updated_at,
            status: comment.status.clone(),
            likes_count: 0,
            comments_count: 0,
            shares_count: 0,
            visibility: ContentVisibility::Public,
            child_comments: Vec::new(),
            author_info,
            is_liked: false, // New comment is not liked by default
        })
    })
}

// Content moderation functions
pub fn moderate_content(request: ContentModerationRequest) -> SquareResult<()> {
    STORAGE.with(|storage| {
        let mut storage = storage.borrow_mut();
        
        match request.content_type {
            ContentType::Post => {
                let post = storage.posts.get_mut(&request.content_id)
                    .ok_or_else(|| SquareError::NotFound(format!("Post not found: {}", request.content_id)))?;
                post.status = request.status;
                post.updated_at = time() / 1_000_000;
            },
            ContentType::Article => {
                let article = storage.articles.get_mut(&request.content_id)
                    .ok_or_else(|| SquareError::NotFound(format!("Article not found: {}", request.content_id)))?;
                article.status = request.status;
                article.updated_at = time() / 1_000_000;
            },
            ContentType::Comment => {
                let comment = storage.comments.get_mut(&request.content_id)
                    .ok_or_else(|| SquareError::NotFound(format!("Comment not found: {}", request.content_id)))?;
                comment.status = request.status;
                comment.updated_at = time() / 1_000_000;
            },
        }
        
        Ok(())
    })
}

// Article management functions
pub fn get_article(id: String) -> SquareResult<ArticleResponse> {
    // Get article data and statistics, but don't get user information yet
    let article_data = STORAGE.with(|storage| {
        let storage = storage.borrow();
        
        let article = storage.articles.get(&id)
            .ok_or_else(|| SquareError::NotFound(format!("Article not found: {}", id)))?;
            
        // Check if article is active
        if article.status != ContentStatus::Active {
            return Err(SquareError::NotFound(format!("Article not available: {}", id)));
        }
        
        // Count likes
        let likes_count = storage.likes.get(&id)
            .map(|likes| likes.len() as u64)
            .unwrap_or(0);
            
        // Count comments
        let comments_count = storage.comments.values()
            .filter(|comment| 
                comment.parent_id == id && 
                comment.parent_type == ParentType::Article &&
                comment.status == ContentStatus::Active
            )
            .count() as u64;
            
        // Count shares
        let shares_count = storage.shares.get(&id)
            .map(|shares| *shares)
            .unwrap_or(0);
        
        // Clone article data for use after STORAGE borrowing ends
        let article_id = article.id.clone();
        let article_author = article.author;
        let article_content = article.content.clone();
        let article_media_urls = article.media_urls.clone();
        let article_hashtags = article.hashtags.clone();
        let article_token_mentions = article.token_mentions.clone();
        let article_created_at = article.created_at;
        let article_updated_at = article.updated_at;
        let article_status = article.status.clone();
        let article_visibility = article.visibility.clone();
        
        Ok((article_id, article_author, article_content, article_media_urls,
            article_hashtags, article_token_mentions, article_created_at,
            article_updated_at, article_status, article_visibility, likes_count, comments_count, shares_count))
    })?;
    
    // Destructure the retrieved data
    let (article_id, article_author, article_content, article_media_urls,
        article_hashtags, article_token_mentions, article_created_at,
        article_updated_at, article_status, article_visibility, likes_count, comments_count, shares_count) = article_data;
    
    // After STORAGE borrowing ends, get user information
    let author_info = crate::services::user::get_user_social_info(article_author, None)?;
    
    // Build response
    Ok(ArticleResponse {
        id: article_id,
        author: article_author,

        content: article_content,
        media_urls: article_media_urls,
        hashtags: article_hashtags,
        token_mentions: article_token_mentions,
        created_at: article_created_at,
        updated_at: article_updated_at,
        status: article_status,
        visibility: article_visibility,
        likes_count,
        comments_count,
        shares_count,
        author_info,
    })
}

pub fn update_article(request: UpdateArticleRequest, caller: Principal) -> SquareResult<ArticleResponse> {

    
    // Validate content length if provided
    // Validate content length
    if request.content.len() > MAX_ARTICLE_LENGTH {
        return Err(SquareError::ContentTooLong(format!(
            "Article content exceeds maximum length of {} characters", MAX_ARTICLE_LENGTH
        )));
    }
    
    // Validate hashtags count if provided
    if let Some(hashtags) = &request.hashtags {
        if hashtags.len() > MAX_HASHTAGS {
            return Err(SquareError::ValidationFailed(format!(
                "Too many hashtags. Maximum allowed is {}", MAX_HASHTAGS
            )));
        }
    }
    
    // Validate token mentions count if provided
    if let Some(token_mentions) = &request.token_mentions {
        if token_mentions.len() > MAX_TOKEN_MENTIONS {
            return Err(SquareError::ValidationFailed(format!(
                "Too many token mentions. Maximum allowed is {}", MAX_TOKEN_MENTIONS
            )));
        }
    }
    
    // Validate media URLs count if provided
    if let Some(media_urls) = &request.media_urls {
        if media_urls.len() > MAX_MEDIA_URLS {
            return Err(SquareError::ValidationFailed(format!(
                "Too many media URLs. Maximum allowed is {}", MAX_MEDIA_URLS
            )));
        }
    }
    
    // Update article
    STORAGE.with(|storage| {
        let mut storage = storage.borrow_mut();
        
        // First check if article exists
        let article_exists = storage.articles.contains_key(&request.id);
        if !article_exists {
            return Err(SquareError::NotFound(format!("Article not found: {}", request.id)));
        }
        
        // Get article info before mutable borrow
        let article_author;
        {
            let article = storage.articles.get(&request.id).unwrap();
            article_author = article.author;
        }
        
        // Check ownership
        if article_author != caller && is_admin().is_err() {
            return Err(SquareError::Unauthorized("You can only update your own articles".to_string()));
        }
        
        // Now get mutable reference
        let article = storage.articles.get_mut(&request.id).unwrap();
        
        // Update required fields
        article.content = request.content;
        
        if let Some(media_urls) = request.media_urls {
            article.media_urls = media_urls;
        }
        
        // Store old and new hashtags for later processing
        let mut old_hashtags = Vec::new();
        let mut new_hashtags = Vec::new();
        
        if let Some(hashtags) = request.hashtags {
            old_hashtags = article.hashtags.clone();
            new_hashtags = hashtags.clone();
            article.hashtags = hashtags;
        }
        
        // Process all article updates first
        if let Some(token_mentions) = request.token_mentions.clone() {
            article.token_mentions = token_mentions;
        }

        // Store any other fields we need to update after releasing the borrow
        let hashtags_to_process = if !old_hashtags.is_empty() || !new_hashtags.is_empty() {
            Some((old_hashtags, new_hashtags))
        } else {
            None
        };
        
        // Release the article borrow completely
        let _ = article;
        
        // Now we can safely process hashtags without conflicting borrows
        if let Some((old_tags, new_tags)) = hashtags_to_process {
            // Remove old hashtags from trending topics
            for hashtag in &old_tags {
                if let Some(count) = storage.trending_topics.get_mut(hashtag) {
                    if *count > 0 {
                        *count -= 1;
                    }
                }
            }
            
            // Add new hashtags to trending topics
            for hashtag in &new_tags {
                let count = storage.trending_topics.entry(hashtag.clone()).or_insert(0);
                *count += 1;
            }
        }
        
        // Store visibility update if provided
        let visibility_update = request.visibility;
        
        // Get a fresh reference to the article after hashtag processing
        let article = storage.articles.get_mut(&request.id).unwrap();
        
        // Apply visibility update if provided
        if let Some(visibility) = visibility_update {
            article.visibility = visibility;
        }
        
        // Update timestamp
        article.updated_at = time() / 1_000_000;
        
        // Return updated article
        get_article(request.id)
    })
}

pub fn delete_article(id: String, caller: Principal) -> SquareResult<()> {
    const MODULE: &str = "services::content";
    const FUNCTION: &str = "delete_article";
    
    STORAGE.with(|storage| {
        let mut storage = storage.borrow_mut();
        
        // Check if article exists and belongs to caller
        let article = storage.articles.get(&id)
            .ok_or_else(|| not_found_error(
                "Article", 
                &id, 
                MODULE, 
                FUNCTION
            ).with_details(format!("Article not found: {}", id)))?;
            
        if article.author != caller && is_admin().is_err() {
            return log_and_return(unauthorized_error(
                "You can only delete your own articles", 
                MODULE, 
                FUNCTION
            ));
        }
        
        // Mark article as deleted instead of removing it
        let article = storage.articles.get_mut(&id).unwrap();
        article.status = ContentStatus::Deleted;
        article.updated_at = time() / 1_000_000;
        
        // Clone hashtags to avoid borrow issues
        let hashtags = article.hashtags.clone();
        
        // Remove hashtags from trending topics
        for hashtag in &hashtags {
            if let Some(count) = storage.trending_topics.get_mut(hashtag) {
                if *count > 0 {
                    *count -= 1;
                }
            }
        }
        
        // Update user stats
        if let Some(user_stats) = storage.user_stats.get_mut(&caller) {
            if user_stats.article_count > 0 {
                user_stats.article_count -= 1;
            }
        }
        
        Ok(())
    })
}

// Comment management functions
pub fn get_comment(id: String, caller: Option<Principal>) -> SquareResult<CommentResponse> {
    STORAGE.with(|storage| {
        let storage = storage.borrow();
        
        let comment = storage.comments.get(&id)
            .ok_or_else(|| SquareError::NotFound(format!("Comment not found: {}", id)))?;
            
        // Check if comment is active
        if comment.status != ContentStatus::Active {
            return Err(SquareError::NotFound(format!("Comment not available: {}", id)));
        }
        
        // Get likes and check if caller has liked
        let likes = storage.likes.get(&id);
        let likes_count = likes
            .map(|likes| likes.len() as u64)
            .unwrap_or(0);
        
        // Check if the caller has liked this comment
        let is_liked = if let Some(caller) = caller {
            likes.map(|likes| likes.contains(&caller)).unwrap_or(false)
        } else {
            false
        };

        // Get author info
        let author_info = crate::services::user::get_user_social_info(comment.author, None)?;
            
        Ok(CommentResponse {
            id: comment.id.clone(),
            author: comment.author,
            content: comment.content.clone(),
            parent_id: comment.parent_id.clone(),
            parent_type: comment.parent_type.clone(),
            created_at: comment.created_at,
            updated_at: comment.updated_at,
            status: comment.status.clone(),
            likes_count,
            comments_count: comment.child_comments.len() as u64,
            shares_count: 0, // TODO: Implement share functionality
            visibility: ContentVisibility::Public, // Default to public for existing comments
            child_comments: comment.child_comments.clone(),
            author_info,
            is_liked,
        })
    })
}

pub fn update_comment(request: UpdateCommentRequest, caller: Principal) -> SquareResult<CommentResponse> {
    const MODULE: &str = "services::content";
    const FUNCTION: &str = "update_comment";
    
    // Validate content length
    if request.content.len() > MAX_COMMENT_LENGTH {
        return log_and_return(validation_error(
            &format!("Comment content exceeds maximum length of {} characters", MAX_COMMENT_LENGTH),
            MODULE,
            FUNCTION
        ));
    }
    
    // Update comment
    STORAGE.with(|storage| {
        let mut storage = storage.borrow_mut();
        
        // Check if comment exists and belongs to caller
        let comment = storage.comments.get_mut(&request.id)
            .ok_or_else(|| not_found_error(
                "Comment", 
                &request.id, 
                MODULE, 
                FUNCTION
            ).with_details(format!("Comment not found: {}", request.id)))?;
            
        if comment.author != caller && is_admin().is_err() {
            return log_and_return(unauthorized_error(
                "You can only update your own comments", 
                MODULE, 
                FUNCTION
            ));
        }
        
        // Store necessary info before updating
        let comment_id = comment.id.clone();
        let comment_author = comment.author;
        let parent_id = comment.parent_id.clone();
        let parent_type = comment.parent_type.clone();
        let created_at = comment.created_at;
        let child_comments = comment.child_comments.clone();
        
        // Update content
        comment.content = request.content.clone();
        
        // Update timestamp
        comment.updated_at = time() / 1_000_000;
        
        // Drop mutable borrow
        drop(storage);
        
        // Get author info
        let author_info = crate::services::user::get_user_social_info(comment_author, None)?;
        
        // Get likes and check if caller has liked with a new borrow
        let (likes_count, is_liked) = STORAGE.with(|storage| {
            let storage = storage.borrow();
            let likes = storage.likes.get(&comment_id);
            let likes_count = likes
                .map(|likes| likes.len() as u64)
                .unwrap_or(0);
            let is_liked = likes
                .map(|likes| likes.contains(&caller))
                .unwrap_or(false);
            (likes_count, is_liked)
        });
        
        Ok(CommentResponse {
            id: comment_id,
            author: comment_author,
            content: request.content,
            parent_id,
            parent_type,
            created_at,
            updated_at: time() / 1_000_000,
            status: ContentStatus::Active,
            likes_count,
            comments_count: child_comments.len() as u64,
            shares_count: 0, // TODO: Implement share functionality
            visibility: ContentVisibility::Public, // Default to public for existing comments
            child_comments,
            author_info,
            is_liked,
        })
    })
}

pub fn delete_comment(id: String, caller: Principal) -> SquareResult<()> {
    const MODULE: &str = "services::content";
    const FUNCTION: &str = "delete_comment";
    
    STORAGE.with(|storage| {
        let mut storage = storage.borrow_mut();
        
        // Check if comment exists and belongs to caller
        let comment = storage.comments.get(&id)
            .ok_or_else(|| not_found_error(
                "Comment", 
                &id, 
                MODULE, 
                FUNCTION
            ).with_details(format!("Comment not found: {}", id)))?;
            
        if comment.author != caller && is_admin().is_err() {
            return log_and_return(unauthorized_error(
                "You can only delete your own comments", 
                MODULE, 
                FUNCTION
            ));
        }
        
        // Mark comment as deleted instead of removing it
        let comment = storage.comments.get_mut(&id).unwrap();
        comment.status = ContentStatus::Deleted;
        comment.updated_at = time() / 1_000_000;
        
        // Update user stats
        if let Some(user_stats) = storage.user_stats.get_mut(&caller) {
            if user_stats.comment_count > 0 {
                user_stats.comment_count -= 1;
            }
        }
        
        Ok(())
    })
}

pub fn get_comments(parent_id: String, parent_type_str: String, pagination: PaginationParams, caller: Option<Principal>) -> SquareResult<CommentsResponse> {
    const MODULE: &str = "services::content";
    const FUNCTION: &str = "get_comments";
    
    // Convert parent_type string to enum
    let parent_type = match parent_type_str.to_lowercase().as_str() {
        "post" => ParentType::Post,
        "article" => ParentType::Article,
        "comment" => ParentType::Comment,
        _ => return log_and_return(validation_error(
            &format!("Invalid parent type: {}", parent_type_str),
            MODULE,
            FUNCTION
        )),
    };
    
    STORAGE.with(|storage| {
        let storage = storage.borrow();
        
        // Check if parent exists
        match parent_type {
            ParentType::Post => {
                if !storage.posts.contains_key(&parent_id) {
                    return Err(SquareError::NotFound(format!("Parent post not found: {}", parent_id)));
                }
            },
            ParentType::Article => {
                if !storage.articles.contains_key(&parent_id) {
                    return Err(SquareError::NotFound(format!("Parent article not found: {}", parent_id)));
                }
            },
            ParentType::Comment => {
                if !storage.comments.contains_key(&parent_id) {
                    return Err(SquareError::NotFound(format!("Parent comment not found: {}", parent_id)));
                }
            },
        }
        
        // Get comments for this parent
        let all_comments: Vec<CommentResponse> = storage.comments.values()
            .filter(|comment| 
                comment.parent_id == parent_id && 
                comment.parent_type == parent_type &&
                comment.status == ContentStatus::Active
            )
            .filter_map(|comment| {
                let likes = storage.likes.get(&comment.id);
                let likes_count = likes
                    .map(|likes| likes.len() as u64)
                    .unwrap_or(0);
                
                // Check if the caller has liked this comment
                let is_liked = if let Some(caller) = caller {
                    likes.map(|likes| likes.contains(&caller)).unwrap_or(false)
                } else {
                    false
                };
                    
                let author_info = match crate::services::user::get_user_social_info(comment.author, None) {
                    Ok(info) => info,
                    Err(_) => return None,
                };
                    
                Some(CommentResponse {
                    id: comment.id.clone(),
                    author: comment.author,
                    content: comment.content.clone(),
                    parent_id: comment.parent_id.clone(),
                    parent_type: comment.parent_type.clone(),
                    created_at: comment.created_at,
                    updated_at: comment.updated_at,
                    status: comment.status.clone(),
                    likes_count,
                    comments_count: comment.child_comments.len() as u64,
                    shares_count: 0, // TODO: Implement share functionality
                    visibility: ContentVisibility::Public, // Default to public for existing comments
                    child_comments: comment.child_comments.clone(),
                    author_info,
                    is_liked,
                })
            })
            .collect();
            
        // Apply pagination
        let total = all_comments.len();
        let start = std::cmp::min(pagination.offset, total);
        let end = std::cmp::min(start + pagination.limit, total);
        let comments = all_comments[start..end].to_vec();
        
        Ok(CommentsResponse {
            comments,
            total: total as u64,
            has_more: end < total,
            next_offset: if end < total { end } else { total },
        })
    })
}

pub fn get_user_content(principal: Principal, content_type: Option<ContentType>, pagination: PaginationParams) -> SquareResult<FeedResponse> {
    STORAGE.with(|storage| {
        let storage = storage.borrow();
        
        // Check if user exists
        if !storage.users.contains_key(&principal) {
            return Err(SquareError::NotFound(format!("User not found: {}", principal)));
        }
        
        let mut posts = Vec::new();
        let mut articles = Vec::new();
        
        // Get user's content based on content type
        match content_type {
            None | Some(ContentType::Post) => {
                // Get user's posts
                if let Some(user_posts) = storage.user_posts.get(&principal) {
                    for post_id in user_posts {
                        if let Some(post) = storage.posts.get(post_id) {
                            if post.status == ContentStatus::Active {
                                if let Ok(post_response) = get_post(post_id.clone()) {
                                    posts.push(post_response);
                                }
                            }
                        }
                    }
                }
            },
            _ => {}
        }
        
        match content_type {
            None | Some(ContentType::Article) => {
                // Get user's articles
                if let Some(user_articles) = storage.user_articles.get(&principal) {
                    for article_id in user_articles {
                        if let Some(article) = storage.articles.get(article_id) {
                            if article.status == ContentStatus::Active {
                                if let Ok(article_response) = get_article(article_id.clone()) {
                                    articles.push(article_response);
                                }
                            }
                        }
                    }
                }
            },
            _ => {}
        }
        
        // Sort by created_at (newest first)
        posts.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        articles.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        
        // Apply pagination (simplified - in a real app we would paginate the combined list)
        let start = std::cmp::min(pagination.offset, posts.len());
        let end = std::cmp::min(start + pagination.limit, posts.len());
        posts = posts[start..end].to_vec();
        
        let start = std::cmp::min(pagination.offset, articles.len());
        let end = std::cmp::min(start + pagination.limit, articles.len());
        articles = articles[start..end].to_vec();
        
        // Clone for use in has_more calculation
        let posts_len = posts.len();
        let articles_len = articles.len();
        
        Ok(FeedResponse {
            posts,
            articles,
            has_more: posts_len >= pagination.limit || articles_len >= pagination.limit,
            next_offset: pagination.offset + pagination.limit,
        })
    })
}

// Helper functions for content display
pub fn get_content_detail(content_id: String, content_type: ContentType, caller: Option<Principal>) -> SquareResult<ContentDetailResponse> {
    STORAGE.with(|storage| {
        let storage = storage.borrow();
        
        match content_type {
            ContentType::Post => {
                let post = get_post(content_id.clone())?;
                
                // Get comments for this post
                let comments: Vec<CommentResponse> = storage.comments.values()
                    .filter(|comment| 
                        comment.parent_id == content_id && 
                        comment.parent_type == ParentType::Post &&
                        comment.status == ContentStatus::Active
                    )
                    .filter_map(|comment| {
                        let likes = storage.likes.get(&comment.id);
                        let likes_count = likes
                            .map(|likes| likes.len() as u64)
                            .unwrap_or(0);
                        
                        // Check if the caller has liked this comment
                        let is_liked = if let Some(caller) = caller {
                            likes.map(|likes| likes.contains(&caller)).unwrap_or(false)
                        } else {
                            false
                        };
                            
                        let author_info = match crate::services::user::get_user_social_info(comment.author, None) {
                            Ok(info) => info,
                            Err(_) => return None,
                        };
                            
                        Some(CommentResponse {
                            id: comment.id.clone(),
                            author: comment.author,
                            content: comment.content.clone(),
                            parent_id: comment.parent_id.clone(),
                            parent_type: comment.parent_type.clone(),
                            created_at: comment.created_at,
                            updated_at: comment.updated_at,
                            status: comment.status.clone(),
                            likes_count,
                            comments_count: comment.child_comments.len() as u64,
                            shares_count: 0, // TODO: Implement share functionality
                            visibility: ContentVisibility::Public, // Default to public for existing comments
                            child_comments: comment.child_comments.clone(),
                            author_info,
                            is_liked,
                        })
                    })
                    .collect();
                
                Ok(ContentDetailResponse {
                    post: Some(post),
                    article: None,
                    comments,
                    has_more_comments: false,
                    next_comment_offset: 0,  
                })
            },
            ContentType::Article => {
                let article = get_article(content_id.clone())?;
                
                // Get comments for this article
                let comments: Vec<CommentResponse> = storage.comments.values()
                    .filter(|comment| 
                        comment.parent_id == content_id && 
                        comment.parent_type == ParentType::Article &&
                        comment.status == ContentStatus::Active
                    )
                    .filter_map(|comment| {
                        let likes = storage.likes.get(&comment.id);
                        let likes_count = likes
                            .map(|likes| likes.len() as u64)
                            .unwrap_or(0);
                        
                        // Check if the caller has liked this comment
                        let is_liked = if let Some(caller) = caller {
                            likes.map(|likes| likes.contains(&caller)).unwrap_or(false)
                        } else {
                            false
                        };
                            
                        let author_info = match crate::services::user::get_user_social_info(comment.author, None) {
                            Ok(info) => info,
                            Err(_) => return None,
                        };
                            
                        Some(CommentResponse {
                            id: comment.id.clone(),
                            author: comment.author,
                            content: comment.content.clone(),
                            parent_id: comment.parent_id.clone(),
                            parent_type: comment.parent_type.clone(),
                            created_at: comment.created_at,
                            updated_at: comment.updated_at,
                            status: comment.status.clone(),
                            likes_count,
                            comments_count: comment.child_comments.len() as u64,
                            shares_count: 0, // TODO: Implement share functionality
                            visibility: ContentVisibility::Public, // Default to public for existing comments
                            child_comments: comment.child_comments.clone(),
                            author_info,
                            is_liked,
                        })
                    })
                    .collect();
                
                Ok(ContentDetailResponse {
                    post: None,
                    article: Some(article),
                    comments,
                    has_more_comments: false,
                    next_comment_offset: 0,  
                })
            },
            ContentType::Comment => {
                Err(SquareError::InvalidOperation("Cannot get detail view of a comment".to_string()))
            },
        }
    })
}
