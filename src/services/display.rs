use candid::Principal;

use crate::models::display::*;
use crate::models::error::SquareResult;
use crate::services::content;
use crate::services::discovery;
use crate::services::user;


// Feed display functions
pub fn get_dashboard_feed(principal: Principal, pagination: crate::models::content::PaginationParams) -> SquareResult<DashboardResponse> {
    const MODULE: &str = "services::display";
    const FUNCTION: &str = "get_dashboard_feed";
    
    
    // Get trending topics
    let trending_topics = discovery::get_trending_topics(crate::models::discovery::GetTrendingTopicsRequest {
        limit: Some(5),
        time_range_hours: Some(24),
    }).map_err(|e| {
        e
    })?;
    
    // Get trending content
    let trending_content = discovery::discover_content(crate::models::discovery::DiscoverContentRequest {
        content_types: None,
        tags: None,
        pagination: pagination.clone(),
        sort_by: Some(crate::models::discovery::SortOption::Trending),
        filter: None,
    }).map_err(|e| {
        e
    })?;
    
    // Get followed users content
    let followed_users = user::get_following(principal.to_string(), Some(principal)).map_err(|e| {
        e
    })?;
    let followed_principals: Vec<Principal> = followed_users.iter().map(|u| u.principal).collect();
    
    // If user follows anyone, get their content, otherwise return empty feed
    let followed_users_content = if !followed_principals.is_empty() {
        // Implementation would filter content by followed users
        // For now, return general feed as placeholder
        discovery::discover_content(crate::models::discovery::DiscoverContentRequest {
            content_types: None,
            tags: None,
            pagination: pagination.clone(),
            sort_by: Some(crate::models::discovery::SortOption::Latest),
            filter: Some(crate::models::discovery::ContentFilter {
                hashtag: None,
                token_mention: None,
                created_after: None,
                created_before: None,
                author: Some(followed_principals[0]), // Filter by the first followed user as an example
            }),
        }).map_err(|e| {
            e
        })?
    } else {
        FeedResponse {
            posts: vec![],
            comments: vec![],
            has_more: false,
            next_offset: 0,
        }
    };
    
    // Get personalized recommendations
    let personalized_recommendations = discovery::get_personalized_recommendations(
        crate::models::discovery::PersonalizedRecommendationsRequest {
            content_types: None,
            pagination,
            include_followed_users: Some(true),
            include_followed_topics: Some(true),
            include_trending: Some(true),
            include_similar_to_liked: Some(true),
            diversity_factor: Some(0.5),
            recency_weight: Some(0.7)
        }
    )?;
    
    Ok(DashboardResponse {
        trending_topics,
        trending_content,
        followed_users_content,
        personalized_recommendations,
    })
}

pub fn get_user_feed(principal: Principal, pagination: crate::models::content::PaginationParams) -> SquareResult<UserFeedResponse> {
    // Get user profile
    let user_profile = user::get_user_social_info(principal.to_string(), None)?;
    
    // Get user content
    let user_content = content::get_user_content(principal.to_text(), None, pagination)?;
    
    Ok(UserFeedResponse {
        posts: user_content.posts,
        user: user_profile,
        has_more: user_content.has_more,
        next_offset: user_content.next_offset,
    })
}

pub fn get_content_detail(content_id: String, content_type: crate::models::content::ContentType, _pagination: crate::models::content::PaginationParams) -> SquareResult<ContentDetailResponse> {
    // Use the existing content detail function
    content::get_content_detail(content_id, content_type, None)
}

// Creator center functions
pub fn get_creator_center(principal: Principal) -> SquareResult<CreatorCenterResponse> {
    // Get recent posts (limited to 5)
    let recent_posts = content::get_user_content(
        principal.to_text(), 
        Some(crate::models::content::ContentType::Post), 
        crate::models::content::PaginationParams { offset: 0, limit: 5 }
    )?;
    
    // Get content stats
    let content_stats = get_content_stats(principal)?;
    
    // Get available tasks
    let available_tasks = crate::services::reward::get_available_tasks(principal)?
        .into_iter()
        .map(|task| CreatorTaskResponse {
            id: task.id,
            title: task.title,
            description: task.description,
            points_reward: task.points,
            deadline: task.expiration_time,
            is_completed: task.is_completed,
        })
        .collect();
    
    Ok(CreatorCenterResponse {
        recent_posts: recent_posts.posts,
        content_stats,
        available_tasks,
    })
}

// Helper function to get content stats
fn get_content_stats(principal: Principal) -> SquareResult<ContentStatsResponse> {
    // This would typically query the storage for user stats
    // For now, return placeholder data
    crate::storage::STORAGE.with(|storage| {
        let storage = storage.borrow();
        
        let user_stats = storage.user_stats.get(&principal)
            .ok_or_else(|| crate::models::error::SquareError::NotFound("User stats not found".to_string()))?;
        
        // Calculate engagement rate (likes + views)
        let total_interactions = user_stats.likes_received;
        let engagement_rate = if user_stats.views_received > 0 {
            (total_interactions as f64) / (user_stats.views_received as f64)
        } else {
            0.0
        };
        
        Ok(ContentStatsResponse {
            total_posts: user_stats.post_count,
            total_comments: user_stats.comment_count,
            total_likes_received: user_stats.likes_received,
            total_views: user_stats.views_received,
            engagement_rate,
        })
    })
}
