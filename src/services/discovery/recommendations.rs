use candid::Principal;
use std::collections::{HashMap, HashSet};

use crate::models::content::{ContentType, PostResponse};
use crate::models::discovery::*;
use crate::models::display::FeedResponse;
use crate::models::error::{SquareResult, SquareError};
use crate::storage::{STORAGE, Post};
use std::borrow::Borrow;
use crate::utils::error_handler::*;
use crate::services::user::social::get_user_social_info;


pub fn get_personalized_recommendations(request: PersonalizedRecommendationsRequest) -> SquareResult<FeedResponse> {
    const MODULE: &str = "services::discovery::recommendations";
    const FUNCTION: &str = "get_personalized_recommendations";
    
    let limit = request.pagination.limit;
    let offset = request.pagination.offset;
    let _content_type = request.content_types.unwrap_or_else(|| vec![ContentType::Post])[0].clone();
    let user = ic_cdk::caller();
    
    // Get user's interests and interactions
    let (user_likes, user_follows, user_interests) = STORAGE.with(|storage| {
        let store = storage.borrow();
        
        // Get user's likes
        let likes: HashSet<String> = store.likes
            .iter()
            .filter(|(_, users)| users.contains(&user))
            .map(|(content_id, _)| content_id.clone())
            .collect();
        
        // Get user's follows
        let follows: HashSet<Principal> = store.user_profiles.as_ref().and_then(|profiles| profiles.get(&user)).map(|profile| profile.followed_users.iter().cloned().collect()).unwrap_or_default();
        
        // Get user's interests
        let interests: HashSet<String> = if let Some(profile) = store.user_profiles.as_ref().and_then(|profiles| profiles.get(&user)) {
            profile.interests.iter().cloned().collect()
        } else {
            HashSet::new()
        };
        
        (likes, follows, interests)
    });
    
    // Get collaborative recommendations
    let limit_value = limit.unwrap_or(10);
    let collaborative_recs = get_collaborative_recommendations(user, limit_value);
    
    // Get content-based recommendations
    let content_based_recs = STORAGE.with(|storage| {
        let store = storage.borrow();
        let mut recommendations = Vec::new();
        
        for post in store.posts.values() {
            // Skip if content type doesn't match
            if !post.hashtags.iter().any(|tag| tag.starts_with("#")) {
                continue;
            }
            
            // Skip if user has already liked this content
            if user_likes.contains(&post.id) {
                continue;
            }
            
            // Calculate content-based score
            let mut score = 0.0;
            
            // Score based on author follows
            if user_follows.contains(&post.author) {
                score += 1.0;
            }
            
            // Score based on matching interests
            let matching_interests = post.tags
                .iter()
                .filter(|tag| user_interests.contains(*tag))
                .count();
            score += matching_interests as f64 * 0.5;
            
            if score > 0.0 {
                recommendations.push((post.id.clone(), ContentType::Post, score));
            }
        }
        
        recommendations
    });
    
    // Merge and sort recommendations
    let mut all_recs: Vec<(String, ContentType, f64)> = Vec::new();
    all_recs.extend(collaborative_recs);
    all_recs.extend(content_based_recs);
    
    // Sort by score
    all_recs.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap());
    
    // Remove duplicates
    let mut seen = HashSet::new();
    all_recs.retain(|(id, _, _)| seen.insert(id.clone()));
    
    // Apply pagination
    let total = all_recs.len() as u64;
    let start = offset.unwrap_or(0);
    let end = (start + limit_value).min(all_recs.len());
    let recommendations = all_recs[start..end].to_vec();
    
    // Convert to feed items
    let feed_items = STORAGE.with(|storage| {
        let store = storage.borrow();
        recommendations.into_iter()
            .filter_map(|(id, _, _)| store.posts.get(&id))
            .map(|post| Ok::<_, SquareError>(PostResponse {
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
                likes_count: 0, // TODO: Get from likes storage
                comments_count: 0, // TODO: Get from comments storage
                author_info: get_user_social_info(post.author.to_string(), None)?,
                news_reference: post.news_reference.clone(),
            }))
            .collect::<Vec<_>>()
    });
    
    Ok(FeedResponse {
        posts: feed_items.into_iter().collect::<Result<Vec<_>, _>>()?,
        comments: vec![],
        has_more: (start as u64 + limit_value as u64) < total,
        next_offset: start + limit_value,
        total,
    })
}

pub fn get_collaborative_recommendations(user: Principal, limit: usize) -> Vec<(String, ContentType, f64)> {
    // Get all users and their interactions
    let (user_interactions, all_users) = STORAGE.with(|storage| {
        let store = storage.borrow();
        
        // Get all users who have interacted with content
        let mut users = HashSet::new();
        for likes in store.likes.values() {
            users.extend(likes.iter());
        }
        
        // Create user interaction map
        let mut interactions = HashMap::new();
        for (content_id, liking_users) in &store.likes {
            for user in liking_users {
                interactions
                    .entry(*user)
                    .or_insert_with(HashSet::new)
                    .insert(content_id.clone());
            }
        }
        
        (interactions, users)
    });
    
    // Calculate user similarities
    let mut user_similarities = Vec::new();
    for other_user in all_users {
        if other_user == user {
            continue;
        }
        
        let similarity = calculate_user_similarity(&user, &other_user);
        if similarity > 0.0 {
            user_similarities.push((other_user, similarity));
        }
    }
    
    // Sort by similarity
    user_similarities.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    
    // Get recommendations from similar users
    let mut recommendations = HashMap::new();
    let user_likes = user_interactions.get(&user).cloned().unwrap_or_default();
    
    for (similar_user, similarity) in user_similarities {
        if let Some(similar_user_likes) = user_interactions.get(&similar_user) {
            for content_id in similar_user_likes {
                if !user_likes.contains(content_id) {
                    let score = recommendations.entry(content_id.clone()).or_insert(0.0);
                    *score += similarity;
                }
            }
        }
    }
    
    // Convert to vector and sort by score
    let mut recommendations: Vec<_> = recommendations.into_iter().collect();
    recommendations.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    
    // Get content types and limit results
    STORAGE.with(|storage| {
        let store = storage.borrow();
        recommendations
            .into_iter()
            .filter_map(|(id, score)| {
                store.posts.get(&id).map(|_post| {
                    (id.clone(), ContentType::Post, score)
                })
            })
            .take(limit)
            .collect::<Vec<_>>()
    })
}

// Calculate similarity between two users based on their likes, follows, and content interactions
fn calculate_user_similarity(user1: &Principal, user2: &Principal) -> f64 {
    STORAGE.with(|storage| {
        let store = storage.borrow();
        
        // Get likes intersection
        let user1_likes = store.likes
            .iter()
            .filter(|(_, users)| users.contains(user1))
            .map(|(content_id, _)| content_id)
            .collect::<HashSet<_>>();
        
        let user2_likes = store.likes
            .iter()
            .filter(|(_, users)| users.contains(user2))
            .map(|(content_id, _)| content_id)
            .collect::<HashSet<_>>();
        
        let likes_intersection = user1_likes.intersection(&user2_likes).count();
        let likes_union = user1_likes.union(&user2_likes).count();
        
        // Get follows intersection
        let user1_follows: HashSet<Principal> = store.user_profiles.as_ref().and_then(|profiles| profiles.get(&user1)).map(|profile| profile.followed_users.iter().cloned().collect()).unwrap_or_default();
        let user2_follows: HashSet<Principal> = store.user_profiles.as_ref().and_then(|profiles| profiles.get(&user2)).map(|profile| profile.followed_users.iter().cloned().collect()).unwrap_or_default();
        
        let follows_intersection = user1_follows.intersection(&user2_follows).count();
        let follows_union = user1_follows.union(&user2_follows).count();
        
        // Calculate Jaccard similarity for likes and follows
        let likes_similarity = if likes_union > 0 {
            likes_intersection as f64 / likes_union as f64
        } else {
            0.0
        };
        
        let follows_similarity = if follows_union > 0 {
            follows_intersection as f64 / follows_union as f64
        } else {
            0.0
        };
        
        // Weighted average of similarities
        0.7 * likes_similarity + 0.3 * follows_similarity
    })
}
