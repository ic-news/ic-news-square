use ic_cdk::api::time;
use candid::Principal;
use std::collections::HashSet;

use crate::models::content::ContentType;
use crate::models::discovery::*;
use crate::models::display::FeedResponse;
use crate::models::error::SquareResult;
use crate::storage::STORAGE;
use crate::utils::error_handler::*;

// Discovery API implementation
pub fn discover_content(request: DiscoverContentRequest) -> SquareResult<FeedResponse> {
    const MODULE: &str = "services::discovery";
    const FUNCTION: &str = "discover_content";
    
    
    // Implementation of content discovery logic
    // Filter content based on the request parameters and return a feed of posts and articles
    
    let content_types = request.content_types.unwrap_or_else(|| vec![ContentType::Post, ContentType::Article]);
    let tags = request.tags;
    let pagination = request.pagination;
    
    let mut posts = vec![];
    let mut articles = vec![];
    let mut total_posts = 0;
    let mut total_articles = 0;
    
    STORAGE.with(|storage| {
        let storage = storage.borrow();
        
        // Filter posts by content type and tags
        if content_types.contains(&ContentType::Post) {
            // Collect all matching posts
            let mut matching_posts = Vec::new();
            
            for (id, post) in &storage.posts {
                if post.status == crate::storage::ContentStatus::Active {
                    let matches_tags = if let Some(ref tag_list) = tags {
                        // Only include posts with matching tags
                        post.hashtags.iter().any(|tag| tag_list.contains(tag))
                    } else {
                        // Include all active posts if no tags specified
                        true
                    };
                    
                    if matches_tags {
                        matching_posts.push(id.clone());
                    }
                }
            }
            
            // Sort by creation date (newest first)
            matching_posts.sort_by(|a, b| {
                let post_a = storage.posts.get(a).unwrap();
                let post_b = storage.posts.get(b).unwrap();
                post_b.created_at.cmp(&post_a.created_at)
            });
            
            // Store total count for pagination
            total_posts = matching_posts.len();
            
            // Apply pagination
            let start = pagination.offset.min(matching_posts.len());
            let end = (pagination.offset + pagination.limit).min(matching_posts.len());
            let paginated_posts = &matching_posts[start..end];
            
            // Convert to PostResponse
            for post_id in paginated_posts {
                if let Some(post) = storage.posts.get(post_id) {
                    // Get author info
                    if let Ok(author_info) = crate::services::user::get_user_social_info(post.author, None) {
                        // Count likes
                        let likes_count = storage.likes.get(post_id)
                            .map(|likes| likes.len() as u64)
                            .unwrap_or(0);
                        
                        // Count comments
                        let comments_count = storage.comments.values()
                            .filter(|comment| 
                                comment.parent_id == *post_id && 
                                comment.parent_type == crate::storage::ParentType::Post &&
                                comment.status == crate::storage::ContentStatus::Active
                            )
                            .count() as u64;
                        
                        // Get shares count
                        let shares_count = storage.shares.get(post_id).copied().unwrap_or(0);
                        
                        // Create post response
                        posts.push(crate::models::content::PostResponse {
                            id: post.id.clone(),
                            author: post.author,
                            content: post.content.clone(),
                            media_urls: post.media_urls.clone(),
                            hashtags: post.hashtags.clone(),
                            token_mentions: post.token_mentions.clone(),
                            tags: post.tags.clone(),
                            created_at: post.created_at,
                            updated_at: post.updated_at,
                            status: post.status.clone(),
                            visibility: post.visibility.clone(),
                            likes_count,
                            comments_count,
                            shares_count,
                            author_info,
                        });
                    }
                }
            }
        }
        
        // Similar logic for articles
        if content_types.contains(&ContentType::Article) {
            // Collect all matching articles
            let mut matching_articles = Vec::new();
            
            for (id, article) in &storage.articles {
                if article.status == crate::storage::ContentStatus::Active {
                    let matches_tags = if let Some(ref tag_list) = tags {
                        // Only include articles with matching tags
                        article.hashtags.iter().any(|tag| tag_list.contains(tag))
                    } else {
                        // Include all active articles if no tags specified
                        true
                    };
                    
                    if matches_tags {
                        matching_articles.push(id.clone());
                    }
                }
            }
            
            // Sort by creation date (newest first)
            matching_articles.sort_by(|a, b| {
                let article_a = storage.articles.get(a).unwrap();
                let article_b = storage.articles.get(b).unwrap();
                article_b.created_at.cmp(&article_a.created_at)
            });
            
            // Store total count for pagination
            total_articles = matching_articles.len();
            
            // Apply pagination
            let start = pagination.offset.min(matching_articles.len());
            let end = (pagination.offset + pagination.limit).min(matching_articles.len());
            let paginated_articles = &matching_articles[start..end];
            
            // Convert to ArticleResponse
            for article_id in paginated_articles {
                if let Some(article) = storage.articles.get(article_id) {
                    // Get author info
                    if let Ok(author_info) = crate::services::user::get_user_social_info(article.author, None) {
                        // Count likes
                        let likes_count = storage.likes.get(article_id)
                            .map(|likes| likes.len() as u64)
                            .unwrap_or(0);
                        
                        // Count comments
                        let comments_count = storage.comments.values()
                            .filter(|comment| 
                                comment.parent_id == *article_id && 
                                comment.parent_type == crate::storage::ParentType::Article &&
                                comment.status == crate::storage::ContentStatus::Active
                            )
                            .count() as u64;
                        
                        // Get shares count
                        let shares_count = storage.shares.get(article_id).copied().unwrap_or(0);
                        
                        // Create article response
                        articles.push(crate::models::content::ArticleResponse {
                            id: article.id.clone(),
                            author: article.author,
                            content: article.content.clone(),
                            media_urls: article.media_urls.clone(),
                            hashtags: article.hashtags.clone(),
                            token_mentions: article.token_mentions.clone(),
                            created_at: article.created_at,
                            updated_at: article.updated_at,
                            status: article.status.clone(),
                            visibility: article.visibility.clone(),
                            likes_count,
                            comments_count,
                            shares_count,
                            author_info,
                        });
                    }
                }
            }
        }
    });
    
    // Determine if there are more items
    let total_items = if content_types.contains(&ContentType::Post) && content_types.contains(&ContentType::Article) {
        total_posts + total_articles
    } else if content_types.contains(&ContentType::Post) {
        total_posts
    } else {
        total_articles
    };
    
    let has_more = total_items > pagination.offset + pagination.limit;
    let next_offset = pagination.offset + pagination.limit;
    
    Ok(FeedResponse {
        posts,
        articles,
        has_more,
        next_offset
    })
}

pub fn search_content(request: SearchRequest) -> SquareResult<Vec<SearchResultResponse>> {
    const MODULE: &str = "services::discovery";
    const FUNCTION: &str = "search_content";
    
    // Implementation of content search logic
    // Search content based on the query string and return matching results
    
    // Validate query length
    if request.query.is_empty() {
        return log_and_return(validation_error(
            "Search query cannot be empty",
            MODULE,
            FUNCTION
        ));
    }
    
    if request.query.len() < 2 {
        return log_and_return(validation_error(
            "Search query must be at least 2 characters long",
            MODULE,
            FUNCTION
        ));
    }
    
    let query = request.query.to_lowercase();
    let content_types = request.content_types.unwrap_or_else(|| vec![ContentType::Post, ContentType::Article]);
    let pagination = request.pagination;
    
    let mut results = Vec::new();
    
    STORAGE.with(|storage| {
        let storage = storage.borrow();
        
        // Search in posts
        if content_types.contains(&ContentType::Post) {
            for (id, post) in &storage.posts {
                if post.status == crate::storage::ContentStatus::Active {
                    // Check if content or hashtags match the query
                    let content_match = post.content.to_lowercase().contains(&query);
                    let hashtag_match = post.hashtags.iter().any(|tag| tag.to_lowercase().contains(&query));
                    let tag_match = post.tags.iter().any(|tag| tag.to_lowercase().contains(&query));
                    
                    if content_match || hashtag_match || tag_match {
                        // Calculate relevance score
                        // Higher score for title/hashtag matches than content matches
                        let mut relevance_score = 0.0;
                        
                        if hashtag_match || tag_match {
                            relevance_score += 1.0; // Higher relevance for tag matches
                        }
                        
                        if content_match {
                            // Count occurrences in content for relevance
                            let occurrences = post.content.to_lowercase().matches(&query).count();
                            relevance_score += 0.5 + (occurrences as f64 * 0.1);
                        }
                        
                        // Create snippet from content
                        let snippet = create_snippet(&post.content, &query);
                        
                        // Add to search results
                        results.push(SearchResultResponse {
                            id: id.clone(),
                            content_type: ContentType::Post,
                            title: None, // Posts don't have titles
                            snippet,
                            author: post.author,
                            created_at: post.created_at,
                            relevance_score,
                        });
                    }
                }
            }
        }
        
        // Search in articles
        if content_types.contains(&ContentType::Article) {
            for (id, article) in &storage.articles {
                if article.status == crate::storage::ContentStatus::Active {
                    // Check if content or hashtags match the query
                    let content_match = article.content.to_lowercase().contains(&query);
                    let hashtag_match = article.hashtags.iter().any(|tag| tag.to_lowercase().contains(&query));
                    
                    if content_match || hashtag_match {
                        // Calculate relevance score
                        let mut relevance_score = 0.0;
                        
                        if hashtag_match {
                            relevance_score += 1.0; // Higher relevance for tag matches
                        }
                        
                        if content_match {
                            // Count occurrences in content for relevance
                            let occurrences = article.content.to_lowercase().matches(&query).count();
                            relevance_score += 0.5 + (occurrences as f64 * 0.1);
                        }
                        
                        // Extract title from article content (first line or first 50 chars)
                        let title = article.content.lines().next()
                            .map(|line| {
                                if line.len() > 50 {
                                    format!("{}..", &line[0..47])
                                } else {
                                    line.to_string()
                                }
                            });
                        
                        // Create snippet from content
                        let snippet = create_snippet(&article.content, &query);
                        
                        // Add to search results
                        results.push(SearchResultResponse {
                            id: id.clone(),
                            content_type: ContentType::Article,
                            title,
                            snippet,
                            author: article.author,
                            created_at: article.created_at,
                            relevance_score,
                        });
                    }
                }
            }
        }
        
        // Search in comments if requested
        if content_types.contains(&ContentType::Comment) {
            for (id, comment) in &storage.comments {
                if comment.status == crate::storage::ContentStatus::Active {
                    // Check if content matches the query
                    if comment.content.to_lowercase().contains(&query) {
                        // Calculate relevance score
                        let occurrences = comment.content.to_lowercase().matches(&query).count();
                        let relevance_score = 0.3 + (occurrences as f64 * 0.05); // Lower base relevance for comments
                        
                        // Create snippet from content
                        let snippet = create_snippet(&comment.content, &query);
                        
                        // Add to search results
                        results.push(SearchResultResponse {
                            id: id.clone(),
                            content_type: ContentType::Comment,
                            title: None,
                            snippet,
                            author: comment.author,
                            created_at: comment.created_at,
                            relevance_score,
                        });
                    }
                }
            }
        }
    });
    
    // Sort by relevance score (highest first)
    results.sort_by(|a, b| b.relevance_score.partial_cmp(&a.relevance_score).unwrap_or(std::cmp::Ordering::Equal));
    
    // Apply pagination
    let start = pagination.offset.min(results.len());
    let end = (pagination.offset + pagination.limit).min(results.len());
    let paginated_results = results[start..end].to_vec();
    
    Ok(paginated_results)
}

// Helper function to create a snippet from content with query highlighted
fn create_snippet(content: &str, query: &str) -> String {
    // Find the first occurrence of the query
    if let Some(pos) = content.to_lowercase().find(query) {
        // Determine start and end positions for the snippet
        let start = if pos > 50 { pos - 50 } else { 0 };
        let end = if pos + query.len() + 100 < content.len() { pos + query.len() + 100 } else { content.len() };
        
        // Create the snippet
        let mut snippet = if start > 0 { "..." } else { "" }.to_string();
        snippet.push_str(&content[start..end]);
        if end < content.len() { snippet.push_str("..."); }
        
        snippet
    } else {
        // If query not found (might happen with hashtag matches), take the first 150 chars
        if content.len() > 150 {
            format!("{}..", &content[0..147])
        } else {
            content.to_string()
        }
    }
}

pub fn get_trending_topics(request: GetTrendingTopicsRequest) -> SquareResult<Vec<TrendingTopicResponse>> {
    const MODULE: &str = "services::discovery";
    const FUNCTION: &str = "get_trending_topics";
    
    // Implementation of trending topics logic
    // This would typically return the most popular hashtags
    
    // Validate limit
    if let Some(limit) = request.limit {
        if limit == 0 {
            return log_and_return(validation_error(
                "Limit must be greater than zero",
                MODULE,
                FUNCTION
            ));
        }
        
        if limit > 100 {
            return log_and_return(validation_error(
                "Limit cannot exceed 100",
                MODULE,
                FUNCTION
            ));
        }
    }
    
    // Validate time range
    if let Some(time_range) = request.time_range_hours {
        if time_range == 0 {
            return log_and_return(validation_error(
                "Time range must be greater than zero",
                MODULE,
                FUNCTION
            ));
        }
        
        if time_range > 720 { // 30 days
            return log_and_return(validation_error(
                "Time range cannot exceed 720 hours (30 days)",
                MODULE,
                FUNCTION
            ));
        }
    }
    
    let limit = request.limit.unwrap_or(10);
    let _time_range_hours = request.time_range_hours.unwrap_or(24);
    
    let mut trending_topics = vec![];
    
    STORAGE.with(|storage| {
        let storage = storage.borrow();
        
        // Get trending topics from storage
        for (topic, count) in &storage.trending_topics {
            trending_topics.push(TrendingTopicResponse {
                name: topic.clone(),
                count: *count,
                trend_direction: TrendDirection::Stable // Simplified implementation
            });
        }
    });
    
    // Sort by count and limit results
    trending_topics.sort_by(|a, b| b.count.cmp(&a.count));
    trending_topics.truncate(limit as usize);
    
    Ok(trending_topics)
}

pub fn get_hot_tags(request: GetHotTagsRequest) -> SquareResult<HotTagsResponse> {
    const MODULE: &str = "services::discovery";
    const FUNCTION: &str = "get_hot_tags";
    
    
    // Implementation of hot tags logic
    // This would typically return the most popular tags
    
    let _tag_type = request.tag_type;
    let _limit = request.limit.unwrap_or(20);
    
    // 移除日志调用以节省 cycles
    
    let hot_tags = vec![]; // Empty for now
    let now = time();
    
    // In a real implementation, this would analyze content and count tag occurrences
    // For now, return a simplified implementation
    
    Ok(HotTagsResponse {
        tags: hot_tags,
        updated_at: now
    })
}

pub fn get_personalized_recommendations(request: PersonalizedRecommendationsRequest) -> SquareResult<FeedResponse> {
    const MODULE: &str = "services::discovery";
    const FUNCTION: &str = "get_personalized_recommendations";
    
    // Get the caller's principal from the context
    let caller = match ic_cdk::caller() {
        principal if principal == Principal::anonymous() => {
            // Anonymous users get non-personalized recommendations
            return discover_content(DiscoverContentRequest {
                content_types: request.content_types,
                tags: None,
                pagination: request.pagination
            });
        },
        principal => principal,
    };
    
    // Get collaborative filtering recommendations
    let collaborative_recommendations = get_collaborative_recommendations(caller, 20);
    let collab_content_ids: HashSet<String> = collaborative_recommendations.iter()
        .map(|(id, _, _)| id.clone())
        .collect();
    
    // Set default values for optional parameters
    let include_followed_users = request.include_followed_users.unwrap_or(true);
    let include_followed_topics = request.include_followed_topics.unwrap_or(true);
    let include_trending = request.include_trending.unwrap_or(true);
    let include_similar_to_liked = request.include_similar_to_liked.unwrap_or(true);
    let diversity_factor = request.diversity_factor.unwrap_or(0.3);
    let recency_weight = request.recency_weight.unwrap_or(0.7);
    
    // Get content types
    let content_types = request.content_types.unwrap_or_else(|| vec![ContentType::Post, ContentType::Article]);
    
    // Prepare result containers
    let mut posts = vec![];
    let mut articles = vec![];
    let mut total_posts = 0;
    let mut total_articles = 0;
    
    // Content scoring and collection
    STORAGE.with(|storage| {
        let storage = storage.borrow();
        
        // Get user profile and stats
        let user_profile = match storage.user_profiles.get(&caller) {
            Some(profile) => profile,
            None => {
                return log_and_return(not_found_error(
                    "UserProfile", 
                    &caller.to_string(), 
                    MODULE, 
                    FUNCTION
                ).with_details("User profile is required for personalized recommendations"));
            }
        };
        
        // Get user's followed users and topics
        let followed_users = &user_profile.followed_users;
        let followed_topics = &user_profile.followed_topics;
        
        // Get user's liked content
        let mut liked_content_ids = HashSet::new();
        let mut liked_content_tags = HashSet::new();
        
        // Collect all content IDs and score them
        let mut scored_posts = Vec::new();
        let mut scored_articles = Vec::new();
        
        // Process posts
        if content_types.contains(&ContentType::Post) {
            for (id, post) in &storage.posts {
                if post.status != crate::storage::ContentStatus::Active {
                    continue;
                }
                
                // Calculate content score based on multiple factors
                let mut score = 0.0;
                
                // 1. Content from followed users
                if include_followed_users && followed_users.contains(&post.author) {
                    score += 2.0;
                }
                
                // 2. Content with followed topics/tags
                if include_followed_topics {
                    for tag in &post.hashtags {
                        if followed_topics.contains(tag) {
                            score += 1.5;
                            break;
                        }
                    }
                }
                
                // 3. Trending content
                if include_trending && storage.trending_content.contains(id) {
                    score += 1.0;
                }
                
                // 3.5 Collaborative filtering recommendations
                if collab_content_ids.contains(id) {
                    // Find the score from collaborative filtering
                    for (rec_id, rec_type, rec_score) in &collaborative_recommendations {
                        if rec_id == id && *rec_type == ContentType::Post {
                            score += rec_score * 1.5; // Give higher weight to collaborative recommendations
                            break;
                        }
                    }
                }
                
                // 4. Similar to liked content
                if include_similar_to_liked {
                    // Check if user has liked this content
                    let user_liked = storage.likes.get(id)
                        .map(|likes| likes.contains(&caller))
                        .unwrap_or(false);
                    
                    if user_liked {
                        // Add to liked content for similarity calculations
                        liked_content_ids.insert(id.clone());
                        liked_content_tags.extend(post.hashtags.clone());
                    }
                    
                    // Score based on tag similarity with liked content
                    let common_tags = post.hashtags.iter()
                        .filter(|tag| liked_content_tags.contains(*tag))
                        .count();
                    
                    if common_tags > 0 {
                        score += 0.5 * (common_tags as f64);
                    }
                }
                
                // 5. Content recency
                let now = time() / 1_000_000;
                let age_hours = (now - post.created_at) / 3600;
                let recency_score = if age_hours < 24 {
                    1.0
                } else if age_hours < 72 {
                    0.7
                } else if age_hours < 168 { // 1 week
                    0.4
                } else {
                    0.2
                };
                
                score += recency_score * recency_weight;
                
                // 6. Engagement metrics
                let likes_count = storage.likes.get(id)
                    .map(|likes| likes.len() as f64)
                    .unwrap_or(0.0);
                
                let comments_count = storage.comments.values()
                    .filter(|comment| 
                        comment.parent_id == *id && 
                        comment.parent_type == crate::storage::ParentType::Post &&
                        comment.status == crate::storage::ContentStatus::Active
                    )
                    .count() as f64;
                
                let shares_count = storage.shares.get(id).copied().unwrap_or(0) as f64;
                
                // Normalize engagement metrics
                let engagement_score = (likes_count * 0.4 + comments_count * 0.4 + shares_count * 0.2) / 100.0;
                score += engagement_score.min(1.0); // Cap at 1.0
                
                // 7. Apply diversity factor - slightly randomize scores to increase diversity
                if diversity_factor > 0.0 {
                    use rand::{Rng, SeedableRng};
                    let mut rng = rand::rngs::StdRng::seed_from_u64(post.created_at);
                    let diversity_adjustment = rng.gen_range(-diversity_factor..diversity_factor);
                    score += diversity_adjustment;
                }
                
                // Add to scored posts
                scored_posts.push((id.clone(), score));
            }
            
            // Sort posts by score (highest first)
            scored_posts.sort_by(|(_, score_a), (_, score_b)| {
                score_b.partial_cmp(score_a).unwrap_or(std::cmp::Ordering::Equal)
            });
            
            // Store total count for pagination
            total_posts = scored_posts.len();
            
            // Apply pagination
            let start = request.pagination.offset.min(scored_posts.len());
            let end = (request.pagination.offset + request.pagination.limit).min(scored_posts.len());
            let paginated_posts = &scored_posts[start..end];
            
            // Convert to PostResponse
            for (post_id, _) in paginated_posts {
                if let Some(post) = storage.posts.get(post_id) {
                    // Get author info
                    if let Ok(author_info) = crate::services::user::get_user_social_info(post.author, Some(caller)) {
                        // Count likes
                        let likes_count = storage.likes.get(post_id)
                            .map(|likes| likes.len() as u64)
                            .unwrap_or(0);
                        
                        // Count comments
                        let comments_count = storage.comments.values()
                            .filter(|comment| 
                                comment.parent_id == *post_id && 
                                comment.parent_type == crate::storage::ParentType::Post &&
                                comment.status == crate::storage::ContentStatus::Active
                            )
                            .count() as u64;
                        
                        // Get shares count
                        let shares_count = storage.shares.get(post_id).copied().unwrap_or(0);
                        
                        // Create post response
                        posts.push(crate::models::content::PostResponse {
                            id: post.id.clone(),
                            author: post.author,
                            content: post.content.clone(),
                            media_urls: post.media_urls.clone(),
                            hashtags: post.hashtags.clone(),
                            token_mentions: post.token_mentions.clone(),
                            tags: post.tags.clone(),
                            created_at: post.created_at,
                            updated_at: post.updated_at,
                            status: post.status.clone(),
                            visibility: post.visibility.clone(),
                            likes_count,
                            comments_count,
                            shares_count,
                            author_info,
                        });
                    }
                }
            }
        }
        
        // Similar logic for articles
        if content_types.contains(&ContentType::Article) {
            for (id, article) in &storage.articles {
                if article.status != crate::storage::ContentStatus::Active {
                    continue;
                }
                
                // Calculate content score based on multiple factors
                let mut score = 0.0;
                
                // 1. Content from followed users
                if include_followed_users && followed_users.contains(&article.author) {
                    score += 2.0;
                }
                
                // 2. Content with followed topics/tags
                if include_followed_topics {
                    for tag in &article.hashtags {
                        if followed_topics.contains(tag) {
                            score += 1.5;
                            break;
                        }
                    }
                }
                
                // 3. Trending content
                if include_trending && storage.trending_content.contains(id) {
                    score += 1.0;
                }
                
                // 3.5 Collaborative filtering recommendations
                if collab_content_ids.contains(id) {
                    // Find the score from collaborative filtering
                    for (rec_id, rec_type, rec_score) in &collaborative_recommendations {
                        if rec_id == id && *rec_type == ContentType::Post {
                            score += rec_score * 1.5; // Give higher weight to collaborative recommendations
                            break;
                        }
                    }
                }
                
                // 4. Similar to liked content
                if include_similar_to_liked {
                    // Check if user has liked this content
                    let user_liked = storage.likes.get(id)
                        .map(|likes| likes.contains(&caller))
                        .unwrap_or(false);
                    
                    if user_liked {
                        // Add to liked content for similarity calculations
                        liked_content_ids.insert(id.clone());
                        liked_content_tags.extend(article.hashtags.clone());
                    }
                    
                    // Score based on tag similarity with liked content
                    let common_tags = article.hashtags.iter()
                        .filter(|tag| liked_content_tags.contains(*tag))
                        .count();
                    
                    if common_tags > 0 {
                        score += 0.5 * (common_tags as f64);
                    }
                }
                
                // 5. Content recency
                let now = time() / 1_000_000;
                let age_hours = (now - article.created_at) / 3600;
                let recency_score = if age_hours < 24 {
                    1.0
                } else if age_hours < 72 {
                    0.7
                } else if age_hours < 168 { // 1 week
                    0.4
                } else {
                    0.2
                };
                
                score += recency_score * recency_weight;
                
                // 6. Engagement metrics
                let likes_count = storage.likes.get(id)
                    .map(|likes| likes.len() as f64)
                    .unwrap_or(0.0);
                
                let comments_count = storage.comments.values()
                    .filter(|comment| 
                        comment.parent_id == *id && 
                        comment.parent_type == crate::storage::ParentType::Article &&
                        comment.status == crate::storage::ContentStatus::Active
                    )
                    .count() as f64;
                
                let shares_count = storage.shares.get(id).copied().unwrap_or(0) as f64;
                
                // Normalize engagement metrics
                let engagement_score = (likes_count * 0.4 + comments_count * 0.4 + shares_count * 0.2) / 100.0;
                score += engagement_score.min(1.0); // Cap at 1.0
                
                // 7. Apply diversity factor - slightly randomize scores to increase diversity
                if diversity_factor > 0.0 {
                    use rand::{Rng, SeedableRng};
                    let mut rng = rand::rngs::StdRng::seed_from_u64(article.created_at);
                    let diversity_adjustment = rng.gen_range(-diversity_factor..diversity_factor);
                    score += diversity_adjustment;
                }
                
                // Add to scored articles
                scored_articles.push((id.clone(), score));
            }
            
            // Sort articles by score (highest first)
            scored_articles.sort_by(|(_, score_a), (_, score_b)| {
                score_b.partial_cmp(score_a).unwrap_or(std::cmp::Ordering::Equal)
            });
            
            // Store total count for pagination
            total_articles = scored_articles.len();
            
            // Apply pagination
            let start = request.pagination.offset.min(scored_articles.len());
            let end = (request.pagination.offset + request.pagination.limit).min(scored_articles.len());
            let paginated_articles = &scored_articles[start..end];
            
            // Convert to ArticleResponse
            for (article_id, _) in paginated_articles {
                if let Some(article) = storage.articles.get(article_id) {
                    // Get author info
                    if let Ok(author_info) = crate::services::user::get_user_social_info(article.author, Some(caller)) {
                        // Count likes
                        let likes_count = storage.likes.get(article_id)
                            .map(|likes| likes.len() as u64)
                            .unwrap_or(0);
                        
                        // Count comments
                        let comments_count = storage.comments.values()
                            .filter(|comment| 
                                comment.parent_id == *article_id && 
                                comment.parent_type == crate::storage::ParentType::Article &&
                                comment.status == crate::storage::ContentStatus::Active
                            )
                            .count() as u64;
                        
                        // Get shares count
                        let shares_count = storage.shares.get(article_id).copied().unwrap_or(0);
                        
                        // Create article response
                        articles.push(crate::models::content::ArticleResponse {
                            id: article.id.clone(),
                            author: article.author,
                            content: article.content.clone(),
                            media_urls: article.media_urls.clone(),
                            hashtags: article.hashtags.clone(),
                            token_mentions: article.token_mentions.clone(),
                            created_at: article.created_at,
                            updated_at: article.updated_at,
                            status: article.status.clone(),
                            visibility: article.visibility.clone(),
                            likes_count,
                            comments_count,
                            shares_count,
                            author_info,
                        });
                    }
                }
            }
        }
        
        // Determine if there are more items
        let total_items = total_posts + total_articles;
        let has_more = total_items > request.pagination.offset + request.pagination.limit;
        let next_offset = request.pagination.offset + request.pagination.limit;
        
        Ok(FeedResponse {
            posts,
            articles,
            has_more,
            next_offset
        })
    })
}

// Calculate similarity between two users based on their likes, follows, and content interactions
fn calculate_user_similarity(user1: &Principal, user2: &Principal) -> f64 {
    STORAGE.with(|storage| {
        let storage = storage.borrow();
        
        // Get user profiles
        let profile1 = match storage.user_profiles.get(user1) {
            Some(p) => p,
            None => return 0.0, // No similarity if profile doesn't exist
        };
        
        let profile2 = match storage.user_profiles.get(user2) {
            Some(p) => p,
            None => return 0.0, // No similarity if profile doesn't exist
        };
        
        // Calculate similarity based on followed users
        let mut similarity = 0.0;
        
        // 1. Common followed users
        let common_followed_users = profile1.followed_users.intersection(&profile2.followed_users).count() as f64;
        let total_followed_users = (profile1.followed_users.len() + profile2.followed_users.len()) as f64;
        
        if total_followed_users > 0.0 {
            similarity += 0.3 * (2.0 * common_followed_users / total_followed_users);
        }
        
        // 2. Common followed topics
        let common_topics = profile1.followed_topics.intersection(&profile2.followed_topics).count() as f64;
        let total_topics = (profile1.followed_topics.len() + profile2.followed_topics.len()) as f64;
        
        if total_topics > 0.0 {
            similarity += 0.3 * (2.0 * common_topics / total_topics);
        }
        
        // 3. Common liked content
        let mut user1_likes = HashSet::new();
        let mut user2_likes = HashSet::new();
        
        // Collect likes for both users
        for (content_id, likers) in &storage.likes {
            if likers.contains(user1) {
                user1_likes.insert(content_id);
            }
            
            if likers.contains(user2) {
                user2_likes.insert(content_id);
            }
        }
        
        let common_likes = user1_likes.intersection(&user2_likes).count() as f64;
        let total_likes = (user1_likes.len() + user2_likes.len()) as f64;
        
        if total_likes > 0.0 {
            similarity += 0.4 * (2.0 * common_likes / total_likes);
        }
        
        similarity
    })
}

// Get content recommendations based on collaborative filtering
pub fn get_collaborative_recommendations(user: Principal, limit: usize) -> Vec<(String, ContentType, f64)> {
    const MODULE: &str = "services::discovery";
    const FUNCTION: &str = "get_collaborative_recommendations";
    
    
    STORAGE.with(|storage| {
        let storage = storage.borrow();
        
        // Get all users
        let all_users: Vec<Principal> = storage.users.keys().cloned().collect();
        
        // Calculate similarity with each user
        let mut user_similarities = Vec::new();
        for other_user in &all_users {
            if other_user == &user {
                continue; // Skip self
            }
            
            let similarity = calculate_user_similarity(&user, other_user);
            if similarity > 0.0 {
                user_similarities.push((other_user.clone(), similarity));
            }
        }
        
        // Sort by similarity (highest first)
        user_similarities.sort_by(|(_, sim_a), (_, sim_b)| {
            sim_b.partial_cmp(sim_a).unwrap_or(std::cmp::Ordering::Equal)
        });
        
        // Take top 10 most similar users
        let top_similar_users = user_similarities.iter().take(10).collect::<Vec<_>>();
        
        // Get content liked by similar users but not by the current user
        let mut user_liked_content = HashSet::new();
        
        // Get content already liked by the user
        for (content_id, likers) in &storage.likes {
            if likers.contains(&user) {
                user_liked_content.insert(content_id.clone());
            }
        }
        
        // Collect recommendations with scores
        let mut recommendations = Vec::new();
        
        // Check posts liked by similar users
        for (id, post) in &storage.posts {
            if post.status != crate::storage::ContentStatus::Active || user_liked_content.contains(id) {
                continue;
            }
            
            let mut score = 0.0;
            
            // Check if similar users liked this post
            if let Some(likers) = storage.likes.get(id) {
                for (similar_user, similarity) in &top_similar_users {
                    if likers.contains(similar_user) {
                        score += similarity;
                    }
                }
            }
            
            if score > 0.0 {
                recommendations.push((id.clone(), ContentType::Post, score));
            }
        }
        
        // Check articles liked by similar users
        for (id, article) in &storage.articles {
            if article.status != crate::storage::ContentStatus::Active || user_liked_content.contains(id) {
                continue;
            }
            
            let mut score = 0.0;
            
            // Check if similar users liked this article
            if let Some(likers) = storage.likes.get(id) {
                for (similar_user, similarity) in &top_similar_users {
                    if likers.contains(similar_user) {
                        score += similarity;
                    }
                }
            }
            
            if score > 0.0 {
                recommendations.push((id.clone(), ContentType::Article, score));
            }
        }
        
        // Sort recommendations by score
        recommendations.sort_by(|(_, _, score_a), (_, _, score_b)| {
            score_b.partial_cmp(score_a).unwrap_or(std::cmp::Ordering::Equal)
        });
        
        // Return top recommendations
        recommendations.into_iter().take(limit).collect()
    })
}

// Internal function to update trending content
pub fn update_trending_content() -> SquareResult<()> {
    const MODULE: &str = "services::discovery";
    const FUNCTION: &str = "update_trending_content";
    
    
    // Implementation of trending content update logic
    // This would typically analyze recent content and update trending topics
    
    let now = time();
    
    STORAGE.with(|storage| {
        let mut storage = storage.borrow_mut();
        
        // Only update if enough time has passed since last update
        if now - storage.last_trending_update > 3600 * 1_000_000_000 { // 1 hour in nanoseconds
            
            // Update trending topics
            // (Implementation details omitted for brevity)
            
            storage.last_trending_update = now;
        }
    });
    
    Ok(())
}
