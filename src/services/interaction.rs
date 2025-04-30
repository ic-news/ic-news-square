use candid::Principal;
use ic_cdk::api::time;
use std::collections::HashSet;
use std::cell::RefCell;

use crate::auth::is_manager_or_admin;
use crate::models::interaction::*;
use crate::models::content::ContentType;
use crate::models::error::{SquareError, SquareResult};
use crate::storage::{STORAGE, ContentStatus, ParentType};
use crate::storage::sharded::ShardedStorage;
use crate::utils::error_handler::*;

// Like functionality
pub fn like_content(content_id: String, content_type: ContentType, caller: Principal) -> SquareResult<()> {
    const MODULE: &str = "services::interaction";
    const FUNCTION: &str = "like_content";
    
    
    let request = LikeContentRequest {
        content_id,
        content_type,
    };
    
    like_content_request(request, caller)
        .map(|_| ())
        .map_err(|e| {
            e
        })
}

pub fn unlike_content(content_id: String, content_type: ContentType, caller: Principal) -> SquareResult<()> {
    const MODULE: &str = "services::interaction";
    const FUNCTION: &str = "unlike_content";
    
    
    // We can reuse the like_content function since it toggles the like status
    like_content(content_id, content_type, caller).map_err(|e| {
        e
    })
}

pub fn get_likes(content_id: String, content_type: ContentType) -> SquareResult<LikesResponse> {
    const MODULE: &str = "services::interaction";
    const FUNCTION: &str = "get_likes";
    
    STORAGE.with(|storage| {
        let storage = storage.borrow();
        
        // Verify content exists
        match content_type {
            ContentType::Post => {
                if !storage.posts.contains_key(&content_id) {
                    return log_and_return(not_found_error(
                        "Post", 
                        &content_id, 
                        MODULE, 
                        FUNCTION
                    ));
                }
            },
            ContentType::Comment => {
                if !storage.comments.contains_key(&content_id) {
                    return log_and_return(not_found_error(
                        "Comment", 
                        &content_id, 
                        MODULE, 
                        FUNCTION
                    ));
                }
            },
        }
        
        // Get likes
        let likes = storage.likes.get(&content_id)
            .map(|likes_set| {
                likes_set.iter()
                    .map(|principal| {
                        // Get user profile if available
                        let username = storage.user_profiles.get(principal)
                            .map(|profile| profile.username.clone())
                            .unwrap_or_else(|| principal.to_string());
                            
                        UserLikeInfo {
                            principal: *principal,
                            username,
                            timestamp: 0, // We don't store like timestamps currently
                        }
                    })
                    .collect::<Vec<UserLikeInfo>>()
            })
            .unwrap_or_default();
            
        // Get total before moving likes
        let total = likes.len() as u64;
        
        Ok(LikesResponse {
            content_id: content_id.clone(),
            content_type,
            likes,
            total,
        })
    })
}

// Internal function to handle like requests
pub fn like_content_request(request: LikeContentRequest, caller: Principal) -> SquareResult<InteractionResponse> {
    const MODULE: &str = "services::interaction";
    const FUNCTION: &str = "like_content_request";
    
    STORAGE.with(|storage| {
        let mut storage = storage.borrow_mut();
        
        // Verify content exists and is active
        match request.content_type {
            ContentType::Post => {
                let post = storage.posts.get(&request.content_id)
                    .ok_or_else(|| not_found_error("Post", &request.content_id, MODULE, FUNCTION))?;
                    
                if post.status != ContentStatus::Active {
                    return log_and_return(invalid_operation_error(
                        "like_content", 
                        "Cannot like inactive content", 
                        MODULE, 
                        FUNCTION
                    ));
                }
            },
            ContentType::Comment => {
                let comment = storage.comments.get(&request.content_id)
                    .ok_or_else(|| not_found_error("Comment", &request.content_id, MODULE, FUNCTION))?;
                    
                if comment.status != ContentStatus::Active {
                    return log_and_return(invalid_operation_error(
                        "like_content", 
                        "Cannot like inactive content", 
                        MODULE, 
                        FUNCTION
                    ));
                }
            },
        }
        
        // Add like
        let likes = storage.likes.entry(request.content_id.clone()).or_insert_with(HashSet::new);
        
        // Check if already liked
        if likes.contains(&caller) {
            // Unlike if already liked
            likes.remove(&caller);
            
            // Update author stats
            let content_author = match request.content_type {
                ContentType::Post => storage.posts.get(&request.content_id).map(|post| post.author),
                ContentType::Comment => storage.comments.get(&request.content_id).map(|comment| comment.author),
            };
            
            if let Some(author) = content_author {
                if let Some(stats) = storage.user_stats.get_mut(&author) {
                    if stats.likes_received > 0 {
                        stats.likes_received -= 1;
                    }
                }
            }
            
            Ok(InteractionResponse {
                success: true,
                message: "Content unliked successfully".to_string(),
            })
        } else {
            // Add like
            likes.insert(caller);
            
            // Update author stats
            let content_author = match request.content_type {
                ContentType::Post => storage.posts.get(&request.content_id).map(|post| post.author),
                ContentType::Comment => storage.comments.get(&request.content_id).map(|comment| comment.author),
            };
            
            if let Some(author) = content_author {
                if let Some(stats) = storage.user_stats.get_mut(&author) {
                    stats.likes_received += 1;
                }
            }
            
            Ok(InteractionResponse {
                success: true,
                message: "Content liked successfully".to_string(),
            })
        }
    })
}

// Report functionality
pub fn report_content(request: ReportContentRequest, caller: Principal) -> SquareResult<()> {
    const MODULE: &str = "services::interaction";
    const FUNCTION: &str = "report_content";
    
    STORAGE.with(|storage| {
        let mut storage = storage.borrow_mut();
        
        // Verify content exists
        match request.content_type {
            ContentType::Post => {
                if !storage.posts.contains_key(&request.content_id) {
                    return log_and_return(not_found_error(
                        "Post", 
                        &request.content_id, 
                        MODULE, 
                        FUNCTION
                    ));
                }
            },
            ContentType::Comment => {
                if !storage.comments.contains_key(&request.content_id) {
                    return log_and_return(not_found_error(
                        "Comment", 
                        &request.content_id, 
                        MODULE, 
                        FUNCTION
                    ));
                }
            },
        }
        
        // Generate report ID
        let content_counter = storage.content_counter + 1;
        storage.content_counter = content_counter;
        let report_id = format!("report_{}", content_counter);
        
        // Clone content_type to avoid move
        let content_type = request.content_type.clone();
        
        // Create report
        let report = ContentReport {
            id: report_id.clone(),
            content_id: request.content_id.clone(),
            content_type,
            reporter: caller,
            reason: request.reason,
            description: request.description,
            status: ReportStatus::Pending,
            created_at: time(),
            resolved_at: None,
            resolver: None,
            resolution_notes: None,
        };
        
        // Store report in the reports collection
        storage.reports.insert(report_id.clone(), report.clone());
        
        // Also store in sharded storage if available
        thread_local! {
            static SHARDED_REPORTS: RefCell<ShardedStorage<crate::models::interaction::ContentReport>> = 
                RefCell::new(ShardedStorage::default());
        }
        
        SHARDED_REPORTS.with(|reports| {
            let mut reports_store = reports.borrow_mut();
            reports_store.insert(report_id.clone(), report.clone());
        });
        
        // For now, just set content under review if it's the first report
        match request.content_type {
            ContentType::Post => {
                if let Some(post) = storage.posts.get_mut(&request.content_id) {
                    if post.status == ContentStatus::Active {
                        post.status = ContentStatus::UnderReview;
                    }
                }
            },
            ContentType::Comment => {
                if let Some(comment) = storage.comments.get_mut(&request.content_id) {
                    if comment.status == ContentStatus::Active {
                        comment.status = ContentStatus::UnderReview;
                    }
                }
            },
        }
        
        Ok(())
    })
}

// Resolve report (admin function)
pub fn resolve_report(request: ResolveReportRequest, caller: Principal) -> SquareResult<InteractionResponse> {
    // Check if caller is admin or manager
    is_manager_or_admin()?;
    
    STORAGE.with(|storage| {
        let mut storage = storage.borrow_mut();
        
        let report_info = storage.reports.get(&request.report_id)
            .ok_or_else(|| SquareError::NotFound(format!("Report not found: {}", request.report_id)))?;
            
        if report_info.status != ReportStatus::Pending {
            return Err(SquareError::InvalidOperation("Report is already resolved or rejected".to_string()));
        }
        
        let content_id = report_info.content_id.clone();
        let content_type = report_info.content_type.clone();
        
        // Update content based on request status
        if request.status == ReportStatus::Resolved {
            // Content violates rules, remove it
            match content_type {
                ContentType::Post => {
                    if let Some(post) = storage.posts.get_mut(&content_id) {
                        post.status = ContentStatus::Removed;
                    }
                },
                ContentType::Comment => {
                    if let Some(comment) = storage.comments.get_mut(&content_id) {
                        comment.status = ContentStatus::Removed;
                    }
                },
            }
        } else if request.status == ReportStatus::Rejected {
            // Report rejected, content remains active
            match content_type {
                ContentType::Post => {
                    if let Some(post) = storage.posts.get_mut(&content_id) {
                        if post.status == ContentStatus::UnderReview {
                            post.status = ContentStatus::Active;
                        }
                    }
                },
                ContentType::Comment => {
                    if let Some(comment) = storage.comments.get_mut(&content_id) {
                        if comment.status == ContentStatus::UnderReview {
                            comment.status = ContentStatus::Active;
                        }
                    }
                },
            }
        }
        
        if let Some(report) = storage.reports.get_mut(&request.report_id) {
            report.status = request.status.clone();
            report.resolved_at = Some(time());
            report.resolver = Some(caller);
            report.resolution_notes = request.notes.clone();
        }
        
        // Update report in sharded storage if available
        thread_local! {
            static SHARDED_REPORTS: RefCell<ShardedStorage<crate::models::interaction::ContentReport>> = 
                RefCell::new(ShardedStorage::default());
        }
        
        // Get updated report for sharded storage
        let report_clone = storage.reports.get(&request.report_id).unwrap().clone();
        let report_id = request.report_id.clone();
        
        SHARDED_REPORTS.with(|reports| {
            let mut reports_store = reports.borrow_mut();
            reports_store.insert(report_id, report_clone);
        });
        
        Ok(InteractionResponse {
            success: true,
            message: format!("Report {} successfully", 
                if request.status == ReportStatus::Resolved { "resolved" } else { "rejected" }),
        })
    })
}

// Get interaction counts
#[allow(dead_code)]
pub fn get_interaction_counts(content_id: String, caller: Option<Principal>) -> SquareResult<InteractionCountsResponse> {
    STORAGE.with(|storage| {
        let storage = storage.borrow();
        
        // Get likes count
        let likes_count = storage.likes.get(&content_id)
            .map(|likes| likes.len() as u64)
            .unwrap_or(0);
            
        // Check if caller liked the content
        let is_liked_by_caller = match caller {
            Some(principal) => storage.likes.get(&content_id)
                .map(|likes| likes.contains(&principal))
                .unwrap_or(false),
            None => false,
        };
        
        // Get comments count (assuming content is a post)
        let comments_count = storage.comments.values()
            .filter(|comment| 
                comment.parent_id == content_id && 
                comment.parent_type == ParentType::Post &&
                comment.status == ContentStatus::Active
            )
            .count() as u64;
            
        Ok(InteractionCountsResponse {
            likes: likes_count,
            comments: comments_count,
            is_liked_by_caller,
        })
    })
}

// View tracking
#[allow(dead_code)]
pub fn view_content(request: ViewContentRequest, _caller: Principal) -> SquareResult<()> {
    STORAGE.with(|storage| {
        let mut storage = storage.borrow_mut();
        
        // Update author stats
        let content_author = match request.content_type {
            ContentType::Post => storage.posts.get(&request.content_id).map(|post| post.author),
            ContentType::Comment => return Ok(()), // Don't track views for comments
        };
        
        if let Some(author) = content_author {
            if let Some(stats) = storage.user_stats.get_mut(&author) {
                stats.views_received += 1;
            }
        }
        
        Ok(())
    })
}
