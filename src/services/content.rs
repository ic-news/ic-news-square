use candid::Principal;
use ic_cdk::api::time;

use crate::auth::is_admin;
use crate::models::content::*;
use crate::models::interaction::*;
use crate::models::user::*;
use crate::models::display::{FeedResponse, ContentDetailResponse};
use crate::storage_main::*;
use crate::storage_main::Storage;
use crate::utils::*;
use crate::utils::content_utils::{calculate_content_length_excluding_base64, calculate_content_length_excluding_base64_and_html};
use crate::{SquareError, SquareResult};
use crate::storage::{STORAGE, Post, Comment, ContentStatus, ParentType, ContentVisibility, NewsReference};
use crate::storage::sharded_ops::insert_post_sharded;
use crate::utils::error_handler::{content_too_long_error, not_found_error, unauthorized_error, log_and_return, validation_error};

// Post management functions
pub fn create_post(request: CreatePostRequest, caller: Principal) -> SquareResult<PostResponse> {
    const MODULE: &str = "services::content";
    const FUNCTION: &str = "create_post";
    
    // Validate content length (excluding HTML tags and base64 images/videos)
    let content_length = calculate_content_length_excluding_base64_and_html(&request.content);
    if content_length > MAX_POST_LENGTH {
        return log_and_return(content_too_long_error(
            "Post", 
            MAX_POST_LENGTH, 
            content_length, 
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
            news_reference: request.news_reference.map(|nr| NewsReference {
                metadata: nr.metadata,
                canister_id: nr.canister_id,
            }),
        };
        
        // Store post in both traditional and sharded storage
        storage.posts.insert(id.clone(), post.clone());
        
        // Also store in sharded storage
        insert_post_sharded(id.clone(), post.clone());
        
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
    let author_info = crate::services::user::get_user_social_info(post.author.to_string(), None)?;
    
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
        author_info,
        news_reference: post.news_reference.map(|nr| NewsReferenceResponse {
            metadata: nr.metadata,
            canister_id: nr.canister_id,
        }),
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
        let post_news_reference = post.news_reference.clone();
        
        Ok((post_id, post_author, post_content, post_media_urls, post_hashtags, 
            post_token_mentions, post_tags, post_created_at, post_updated_at, 
            post_status, post_visibility, likes_count, comments_count, post_news_reference))
    })?;
    
    // Destructure the retrieved data
    let (post_id, post_author, post_content, post_media_urls, post_hashtags, 
         post_token_mentions, post_tags, post_created_at, post_updated_at, 
         post_status, post_visibility, likes_count, comments_count, post_news_reference) = post_data;
    
    // After STORAGE borrowing ends, get user information
    let author_info = crate::services::user::get_user_social_info(post_author.to_string(), None)?;
    
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
        author_info,
        news_reference: post_news_reference.map(|nr| NewsReferenceResponse {
            metadata: nr.metadata,
            canister_id: nr.canister_id,
        }),
    })
}

pub fn get_posts(pagination: PaginationParams) -> SquareResult<PostsResponse> {
    const MODULE: &str = "services::content";
    const FUNCTION: &str = "get_posts";
    
    // First collect all necessary post data, but don't get user information yet
    let post_data = STORAGE.with(|storage| {
        let storage = storage.borrow();
        
        // Collect post data and statistics
        let posts_data: Vec<(Post, u64, u64)> = storage.posts.values()
            .filter(|post| post.status == ContentStatus::Active)
            .map(|post| {
                // Calculate likes, comments count for each post
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
                
                // Return post and statistics data
                (post.clone(), likes_count, comments_count)
            })
            .collect();
        
        posts_data
    });
    
    // After STORAGE borrowing ends, get user info for each post and build PostResponse
    let mut posts: Vec<PostResponse> = Vec::new();
    for (post, likes_count, comments_count) in post_data {
        // Get user information
        if let Ok(author_info) = crate::services::user::get_user_social_info(post.author.to_string(), None) {
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
                author_info,
                news_reference: post.news_reference.map(|nr| NewsReferenceResponse {
                    metadata: nr.metadata,
                    canister_id: nr.canister_id,
                }),
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
        // Validate content length (excluding base64 images and videos)
        let content_length = calculate_content_length_excluding_base64(&request.content);
        if content_length > MAX_POST_LENGTH {
            return log_and_return(content_too_long_error(
                "Post", 
                MAX_POST_LENGTH, 
                content_length, 
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
        
        // Update news reference if provided
        if let Some(news_reference) = request.news_reference {
            post.news_reference = Some(NewsReference {
                metadata: news_reference.metadata,
                canister_id: news_reference.canister_id,
            });
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
        let post_news_reference = post.news_reference.clone();
        
        // Count likes
        let likes_count = storage.likes.get(&post_id)
            .map(|likes| likes.len() as u64)
            .unwrap_or(0);
        
        // Return all necessary data
        Ok((post_id, post_author, post_content, post_media_urls, post_hashtags, 
            post_token_mentions, post_created_at, post_updated_at, post_status, 
            post_tags, post_visibility, likes_count, post_news_reference))
    })?;
    
    // Destructure the retrieved data
    let (post_id, post_author, post_content, post_media_urls, post_hashtags, 
         post_token_mentions, post_created_at, post_updated_at, post_status, 
         post_tags, post_visibility, likes_count, post_news_reference) = post_data;
    
    // After STORAGE's mutable borrow ends, get user information
    let author_info = crate::services::user::get_user_social_info(post_author.to_string(), None)?;
    
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
        tags: post_tags,
        comments_count: 0, // We'll need to count comments in a separate function
        author_info,
        news_reference: post_news_reference.map(|nr| NewsReferenceResponse {
            metadata: nr.metadata,
            canister_id: nr.canister_id,
        }),
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

// Comment management functions
pub fn create_comment(request: CreateCommentRequest, caller: Principal) -> SquareResult<CommentResponse> {
    const MODULE: &str = "services::content";
    const FUNCTION: &str = "create_comment";
    
    // Validate content length (excluding HTML tags and base64 images/videos)
    let content_length = calculate_content_length_excluding_base64_and_html(&request.content);
    if content_length > MAX_COMMENT_LENGTH {
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
        let author_info = crate::services::user::get_user_social_info(caller.to_string(), None)?;
        
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
            visibility: ContentVisibility::Public,
            child_comments: Vec::<Box<CommentResponse>>::new(), // Empty vector of boxed comment responses
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
        let author_info = crate::services::user::get_user_social_info(comment.author.to_string(), None)?;
            
        // Get child comments
        let child_comments_full = get_child_comments(&storage, &comment.child_comments, caller);
            
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
            visibility: ContentVisibility::Public, // Default to public for existing comments
            child_comments: child_comments_full, // Use the child comments we retrieved earlier
            author_info,
            is_liked,
        })
    })
}

pub fn update_comment(request: UpdateCommentRequest, caller: Principal) -> SquareResult<CommentResponse> {
    const MODULE: &str = "services::content";
    const FUNCTION: &str = "update_comment";
    
    // Validate content length (excluding HTML tags and base64 images/videos)
    let content_length = calculate_content_length_excluding_base64_and_html(&request.content);
    if content_length > MAX_COMMENT_LENGTH {
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
        let author_info = crate::services::user::get_user_social_info(comment_author.to_string(), None)?;
        
        // Get likes and check if caller has liked with a new borrow
        let (likes_count, is_liked, child_comments_full) = STORAGE.with(|storage| {
            let storage = storage.borrow();
            let likes = storage.likes.get(&comment_id);
            let likes_count = likes
                .map(|likes| likes.len() as u64)
                .unwrap_or(0);
            let is_liked = likes
                .map(|likes| likes.contains(&caller))
                .unwrap_or(false);
            
            // Get child comments
            let child_comments_full = get_child_comments(&storage, &child_comments, Some(caller));
            
            (likes_count, is_liked, child_comments_full)
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
            visibility: ContentVisibility::Public, // Default to public for existing comments
            child_comments: child_comments_full, // Use the child comments we retrieved earlier
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

// Helper function to recursively get child comments
fn get_child_comments(storage: &Storage, child_ids: &Vec<String>, caller: Option<Principal>) -> Vec<Box<CommentResponse>> {
    let mut child_comments = Vec::new();
    
    for child_id in child_ids {
        if let Some(comment) = storage.comments.get(child_id) {
            // Skip inactive comments
            if comment.status != ContentStatus::Active {
                continue;
            }
            
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
            
            let author_info = match crate::services::user::get_user_social_info(comment.author.to_string(), None) {
                Ok(info) => info,
                Err(_) => continue, // Skip if we can't get author info
            };
            
            // Recursively get child comments
            let nested_child_comments = get_child_comments(storage, &comment.child_comments, caller);
            
            child_comments.push(Box::new(CommentResponse {
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
                visibility: ContentVisibility::Public, // Default to public for existing comments
                child_comments: nested_child_comments,
                author_info,
                is_liked,
            }));
        }
    }
    
    child_comments
}

pub fn get_comments(parent_id: String, parent_type_str: String, pagination: PaginationParams, caller: Option<Principal>) -> SquareResult<CommentsResponse> {
    const MODULE: &str = "services::content";
    const FUNCTION: &str = "get_comments";
    
    // Convert parent_type string to enum
    let parent_type = match parent_type_str.to_lowercase().as_str() {
        "post" => ParentType::Post,
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
                    
                let author_info = match crate::services::user::get_user_social_info(comment.author.to_string(), None) {
                    Ok(info) => info,
                    Err(_) => return None,
                };
                    
                // Get child comments recursively
                let child_comments = get_child_comments(&storage, &comment.child_comments, caller);
                
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
                    visibility: ContentVisibility::Public, // Default to public for existing comments
                    child_comments: child_comments,
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

pub fn get_user_content(user_identifier: String, content_type: Option<ContentType>, pagination: PaginationParams) -> SquareResult<FeedResponse> {
    // Determine if the identifier is a Principal or a handle
    let principal = if let Ok(principal) = Principal::from_text(&user_identifier) {
        // It's a valid Principal
        principal
    } else {
        // Try to find user by handle
        crate::services::user::find_user_by_handle(&user_identifier)?
    };
    STORAGE.with(|storage| {
        let storage = storage.borrow();
        
        // Check if user exists
        if !storage.users.contains_key(&principal) {
            return Err(SquareError::NotFound(format!("User not found: {}", principal)));
        }
        
        let mut posts = Vec::new();
        let mut comments = Vec::new();
        
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
            None | Some(ContentType::Comment) => {
                // Get user's comments
                if let Some(user_comments) = storage.user_comments.get(&principal) {
                    for comment_id in user_comments {
                        if let Some(comment) = storage.comments.get(comment_id) {
                            if comment.status == ContentStatus::Active {
                                // 使用 get_comment 函数获取评论详情，传入当前用户作为可选参数
                                if let Ok(comment_response) = get_comment(comment_id.clone(), Some(principal)) {
                                    comments.push(comment_response);
                                }
                            }
                        }
                    }
                }
            },
            _ => {}
        }
        
        // Sort by created_at (newest first)
        posts.sort_by(|a: &PostResponse, b: &PostResponse| b.created_at.cmp(&a.created_at));
        comments.sort_by(|a: &CommentResponse, b: &CommentResponse| b.created_at.cmp(&a.created_at));
        
        // Apply pagination (simplified - in a real app we would paginate the combined list)
        let start = std::cmp::min(pagination.offset, posts.len());
        let end = std::cmp::min(start + pagination.limit, posts.len());
        posts = posts[start..end].to_vec();
        
        let start = std::cmp::min(pagination.offset, comments.len());
        let end = std::cmp::min(start + pagination.limit, comments.len());
        comments = comments[start..end].to_vec();
        
        // Clone for use in has_more calculation
        let posts_len = posts.len();
        let comments_len = comments.len();
        
        Ok(FeedResponse {
            posts,
            comments,
            has_more: posts_len >= pagination.limit || comments_len >= pagination.limit,
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
                            
                        let author_info = match crate::services::user::get_user_social_info(comment.author.to_string(), None) {
                            Ok(info) => info,
                            Err(_) => return None,
                        };
                        
                        // Get child comments
                        let child_comments_full = get_child_comments(&storage, &comment.child_comments, caller);
                            
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
                            visibility: ContentVisibility::Public, // Default to public for existing comments
                            child_comments: child_comments_full, // Use the child comments we retrieved earlier
                            author_info,
                            is_liked,
                        })
                    })
                    .collect();
                
                Ok(ContentDetailResponse {
                    post: Some(post),
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
