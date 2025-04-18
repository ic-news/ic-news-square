use candid::Principal;

use crate::models::content::*;
use crate::models::error::{SquareError, SquareResult};
use crate::storage::{STORAGE, Post, Article, Comment, ContentStatus, ParentType, ContentVisibility};
use crate::storage::sharded::{SHARDED_POSTS, SHARDED_ARTICLES, SHARDED_COMMENTS, SHARDED_LIKES};
use crate::utils::error_handler::*;

// Helper function to get a post from sharded storage
pub fn get_post_sharded(id: &str) -> SquareResult<Post> {
    const MODULE: &str = "services::sharded_content";
    const FUNCTION: &str = "get_post_sharded";
    
    
    // Try to get from sharded storage first
    let post = SHARDED_POSTS.with(|sharded_posts| {
        let mut sharded_posts = sharded_posts.borrow_mut();
        sharded_posts.get(id)
    });
    
    // If found in sharded storage, return it
    if let Some(post) = post {
        // 移除日志调用以节省 cycles
        return Ok(post);
    }
    
    // Otherwise, try to get from main storage
    STORAGE.with(|storage| {
        let storage = storage.borrow();
        storage.posts.get(id)
            .cloned()
            .ok_or_else(|| {
                not_found_error("Post", id, MODULE, FUNCTION)
            })
    })
}

// Helper function to get an article from sharded storage
pub fn get_article_sharded(id: &str) -> SquareResult<Article> {
    const MODULE: &str = "services::sharded_content";
    const FUNCTION: &str = "get_article_sharded";
    
    
    // Try to get from sharded storage first
    let article = SHARDED_ARTICLES.with(|sharded_articles| {
        let mut sharded_articles = sharded_articles.borrow_mut();
        sharded_articles.get(id)
    });
    
    // If found in sharded storage, return it
    if let Some(article) = article {
        return Ok(article);
    }
    
    // Otherwise, try to get from main storage
    STORAGE.with(|storage| {
        let storage = storage.borrow();
        storage.articles.get(id)
            .cloned()
            .ok_or_else(|| {
                not_found_error("Article", id, MODULE, FUNCTION)
            })
    })
}

// Helper function to get a comment from sharded storage
pub fn get_comment_sharded(id: &str) -> SquareResult<Comment> {
    const MODULE: &str = "services::sharded_content";
    const FUNCTION: &str = "get_comment_sharded";
    
    
    // Try to get from sharded storage first
    let comment = SHARDED_COMMENTS.with(|sharded_comments| {
        let mut sharded_comments = sharded_comments.borrow_mut();
        sharded_comments.get(id)
    });
    
    // If found in sharded storage, return it
    if let Some(comment) = comment {
        return Ok(comment);
    }
    
    // Otherwise, try to get from main storage
    STORAGE.with(|storage| {
        let storage = storage.borrow();
        storage.comments.get(id)
            .cloned()
            .ok_or_else(|| {
                not_found_error("Comment", id, MODULE, FUNCTION)
            })
    })
}

// Helper function to check if a user has liked content
pub fn has_liked_sharded(content_id: &str, user: Principal) -> bool {
    // Check in sharded storage first
    let has_liked = SHARDED_LIKES.with(|sharded_likes| {
        let mut sharded_likes = sharded_likes.borrow_mut();
        sharded_likes.has_liked(content_id, user)
    });
    
    // If found in sharded storage, return result
    if has_liked {
        return true;
    }
    
    // Otherwise, check in main storage
    STORAGE.with(|storage| {
        let storage = storage.borrow();
        storage.likes.get(content_id)
            .map_or(false, |likes| likes.contains(&user))
    })
}

// Helper function to get likes count for content
pub fn get_likes_count_sharded(content_id: &str) -> usize {
    // Get from sharded storage
    let sharded_count = SHARDED_LIKES.with(|sharded_likes| {
        let mut sharded_likes = sharded_likes.borrow_mut();
        sharded_likes.count_likes(content_id)
    });
    
    // Get from main storage
    let main_count = STORAGE.with(|storage| {
        let storage = storage.borrow();
        storage.likes.get(content_id)
            .map_or(0, |likes| likes.len())
    });
    
    // Return the sum (in case data is partially migrated)
    sharded_count + main_count
}

// Get post with sharded storage
pub fn get_post(id: String) -> SquareResult<PostResponse> {
    // Get the post from sharded storage
    let post = get_post_sharded(&id)?;
    
    // Check if post is active
    if post.status != ContentStatus::Active {
        return Err(SquareError::NotFound(format!("Post not found: {}", id)));
    }
    
    // Get author profile
    let author_info = STORAGE.with(|storage| {
        let storage = storage.borrow();
        if let Some(profile) = storage.user_profiles.get(&post.author) {
            crate::models::user::UserSocialResponse {
                principal: post.author,
                username: profile.username.clone(),
                handle: profile.handle.clone(),
                avatar: profile.avatar.clone(),
                bio: profile.bio.clone(),
                is_followed_by_caller: false,
                is_following: false,
                followers_count: profile.followers.len() as u64,
                following_count: profile.followed_users.len() as u64,
            }
        } else {
            crate::models::user::UserSocialResponse {
                principal: post.author,
                username: "Unknown".to_string(),
                handle: "unknown".to_string(),
                avatar: Some("".to_string()),
                bio: Some("".to_string()),
                is_followed_by_caller: false,
                is_following: false,
                followers_count: 0,
                following_count: 0,
            }
        }
    });
    
    // Get likes count
    let likes_count = get_likes_count_sharded(&id) as u64;
    
    // Get comments count
    let comments_count = STORAGE.with(|storage| {
        let storage = storage.borrow();
        storage.comments.values()
            .filter(|c| c.parent_id == id && c.parent_type == ParentType::Post && c.status == ContentStatus::Active)
            .count() as u64
    });
    
    // Get shares count
    let shares_count = STORAGE.with(|storage| {
        let storage = storage.borrow();
        storage.shares.get(&id).copied().unwrap_or(0)
    });
    
    // Create post response
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
        likes_count,
        comments_count,
        shares_count,
        visibility: post.visibility,
        author_info,
    })
}

// Get article with sharded storage
pub fn get_article(id: String) -> SquareResult<ArticleResponse> {
    // Get the article from sharded storage
    let article = get_article_sharded(&id)?;
    
    // Check if article is active
    if article.status != ContentStatus::Active {
        return Err(SquareError::NotFound(format!("Article not found: {}", id)));
    }
    
    // Get author profile
    let author_info = STORAGE.with(|storage| {
        let storage = storage.borrow();
        if let Some(profile) = storage.user_profiles.get(&article.author) {
            crate::models::user::UserSocialResponse {
                principal: article.author,
                username: profile.username.clone(),
                handle: profile.handle.clone(),
                avatar: profile.avatar.clone(),
                bio: profile.bio.clone(),
                is_followed_by_caller: false,
                is_following: false,
                followers_count: profile.followers.len() as u64,
                following_count: profile.followed_users.len() as u64,
            }
        } else {
            crate::models::user::UserSocialResponse {
                principal: article.author,
                username: "Unknown".to_string(),
                handle: "unknown".to_string(),
                avatar: Some("".to_string()),
                bio: Some("".to_string()),
                is_followed_by_caller: false,
                is_following: false,
                followers_count: 0,
                following_count: 0,
            }
        }
    });
    
    // Get likes count
    let likes_count = get_likes_count_sharded(&id) as u64;
    
    // Get comments count
    let comments_count = STORAGE.with(|storage| {
        let storage = storage.borrow();
        storage.comments.values()
            .filter(|c| c.parent_id == id && c.parent_type == ParentType::Article && c.status == ContentStatus::Active)
            .count() as u64
    });
    
    // Get shares count
    let shares_count = STORAGE.with(|storage| {
        let storage = storage.borrow();
        storage.shares.get(&id).copied().unwrap_or(0)
    });
    
    // Create article response
    Ok(ArticleResponse {
        id: article.id,
        author: article.author,
        content: article.content,
        media_urls: article.media_urls,
        hashtags: article.hashtags,
        token_mentions: article.token_mentions,
        created_at: article.created_at,
        updated_at: article.updated_at,
        status: article.status,
        visibility: article.visibility,
        likes_count,
        comments_count,
        shares_count,
        author_info,
    })
}

// Get comment with sharded storage
pub fn get_comment(id: String, caller: Option<Principal>) -> SquareResult<CommentResponse> {
    // Get the comment from sharded storage
    let comment = get_comment_sharded(&id)?;
    
    // Check if comment is active
    if comment.status != ContentStatus::Active {
        return Err(SquareError::NotFound(format!("Comment not found: {}", id)));
    }
    
    // Get author profile
    let author_info = STORAGE.with(|storage| {
        let storage = storage.borrow();
        if let Some(profile) = storage.user_profiles.get(&comment.author) {
            crate::models::user::UserSocialResponse {
                principal: comment.author,
                username: profile.username.clone(),
                handle: profile.handle.clone(),
                avatar: profile.avatar.clone(),
                bio: profile.bio.clone(),
                is_followed_by_caller: false,
                is_following: false,
                followers_count: profile.followers.len() as u64,
                following_count: profile.followed_users.len() as u64,
            }
        } else {
            crate::models::user::UserSocialResponse {
                principal: comment.author,
                username: "Unknown".to_string(),
                handle: "unknown".to_string(),
                avatar: Some("".to_string()),
                bio: Some("".to_string()),
                is_followed_by_caller: false,
                is_following: false,
                followers_count: 0,
                following_count: 0,
            }
        }
    });
    
    // Check if user has liked the comment
    let is_liked = caller.map_or(false, |user| has_liked_sharded(&id, user));
    
    // Create comment response
    Ok(CommentResponse {
        id: comment.id,
        author: comment.author,
        content: comment.content,
        parent_id: comment.parent_id,
        parent_type: comment.parent_type,
        created_at: comment.created_at,
        updated_at: comment.updated_at,
        status: comment.status,
        likes_count: comment.likes_count,
        comments_count: 0, // Will be populated by caller if needed
        shares_count: 0,   // Will be populated by caller if needed
        visibility: ContentVisibility::Public, // Default visibility for comments
        child_comments: Vec::new(), // Empty for now, will be populated by caller if needed
        author_info,
        is_liked,
    })
}

// Insert a post into sharded storage
pub fn insert_post_sharded(id: String, post: Post) {
    SHARDED_POSTS.with(|sharded_posts| {
        let mut sharded_posts = sharded_posts.borrow_mut();
        sharded_posts.insert(id, post);
    });
}

// Insert an article into sharded storage
pub fn insert_article_sharded(id: String, article: Article) {
    SHARDED_ARTICLES.with(|sharded_articles| {
        let mut sharded_articles = sharded_articles.borrow_mut();
        sharded_articles.insert(id, article);
    });
}

// Insert a comment into sharded storage
pub fn insert_comment_sharded(id: String, comment: Comment) {
    SHARDED_COMMENTS.with(|sharded_comments| {
        let mut sharded_comments = sharded_comments.borrow_mut();
        sharded_comments.insert(id, comment);
    });
}

// Add a like to sharded storage
pub fn add_like_sharded(content_id: &str, user: Principal) -> bool {
    SHARDED_LIKES.with(|sharded_likes| {
        let mut sharded_likes = sharded_likes.borrow_mut();
        sharded_likes.add_like(content_id, user)
    })
}

// Remove a like from sharded storage
pub fn remove_like_sharded(content_id: &str, user: Principal) -> bool {
    SHARDED_LIKES.with(|sharded_likes| {
        let mut sharded_likes = sharded_likes.borrow_mut();
        sharded_likes.remove_like(content_id, user)
    })
}
