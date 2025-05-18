use candid::Principal;
use ic_cdk::api::time;
use std::collections::{HashSet, HashMap};
use std::cell::RefCell;
use crate::storage::UserStats;

use crate::auth::is_manager_or_admin;
use crate::models::interaction::*;
use crate::models::content::ContentType;
use crate::models::error::{SquareError, SquareResult};
use crate::storage::{ContentStatus, ParentType, STORAGE};
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
    
    // Verify content exists in main storage
    match content_type {
        ContentType::Post => {
            let post_exists = STORAGE.with(|storage| {
                let store = storage.borrow();
                store.posts.contains_key(&content_id)
            });
            
            if !post_exists {
                return log_and_return(not_found_error(
                    "Post", 
                    &content_id, 
                    MODULE, 
                    FUNCTION
                ));
            }
        },
        ContentType::Comment => {
            let comment_exists = STORAGE.with(|storage| {
                let store = storage.borrow();
                store.comments.contains_key(&content_id)
            });
            
            if !comment_exists {
                return log_and_return(not_found_error(
                    "Comment", 
                    &content_id, 
                    MODULE, 
                    FUNCTION
                ));
            }
        },
    }
    
    // Get likes from main storage
    let likes = STORAGE.with(|storage| {
        let store = storage.borrow();
        
        if let Some(likes_set) = store.likes.get(&content_id) {
            // Convert likes to UserLikeInfo objects
            likes_set.iter().map(|principal| {
                // Get user profile if available
                let username = if let Some(profiles) = &store.user_profiles {
                    if let Some(profile) = profiles.get(principal) {
                        profile.username.clone()
                    } else {
                        principal.to_string()
                    }
                } else {
                    principal.to_string()
                };
                
                UserLikeInfo {
                    principal: *principal,
                    username,
                    timestamp: 0, // We don't store like timestamps currently
                }
            }).collect::<Vec<UserLikeInfo>>()
        } else {
            Vec::new()
        }
    });
    
    // Get total before moving likes
    let total = likes.len() as u64;
    
    Ok(LikesResponse {
        content_id: content_id.clone(),
        content_type,
        likes,
        total,
    })
}

// Internal function to handle like requests
pub fn like_content_request(request: LikeContentRequest, caller: Principal) -> SquareResult<InteractionResponse> {
    const MODULE: &str = "services::interaction";
    const FUNCTION: &str = "like_content_request";
    
    // Verify content exists and is active in main storage
    let content_author = match request.content_type {
        ContentType::Post => {
            // Get post from main storage
            let post_result = STORAGE.with(|storage| {
                let store = storage.borrow();
                store.posts.get(&request.content_id).cloned()
            });
            
            // Check if post exists
            let post = post_result.ok_or_else(|| not_found_error("Post", &request.content_id, MODULE, FUNCTION))?;
            
            // Check if post is active
            if post.status != ContentStatus::Active {
                return log_and_return(invalid_operation_error(
                    "like_content", 
                    "Cannot like inactive content", 
                    MODULE, 
                    FUNCTION
                ));
            }
            
            // Return post author
            Some(post.author)
        },
        ContentType::Comment => {
            // Get comment from main storage
            let comment_result = STORAGE.with(|storage| {
                let store = storage.borrow();
                store.comments.get(&request.content_id).cloned()
            });
            
            // Check if comment exists
            let comment = comment_result.ok_or_else(|| not_found_error("Comment", &request.content_id, MODULE, FUNCTION))?;
            
            // Check if comment is active
            if comment.status != ContentStatus::Active {
                return log_and_return(invalid_operation_error(
                    "like_content", 
                    "Cannot like inactive content", 
                    MODULE, 
                    FUNCTION
                ));
            }
            
            // Return comment author
            Some(comment.author)
        },
    };
    
    // Check if already liked using main storage
    let already_liked = STORAGE.with(|storage| {
        let store = storage.borrow();
        store.likes.get(&request.content_id)
            .map_or(false, |principals| principals.contains(&caller))
    });
    
    if already_liked {
        // Unlike if already liked
        STORAGE.with(|storage| {
            let mut store = storage.borrow_mut();
            // Remove like from main storage
            if let Some(principals) = store.likes.get_mut(&request.content_id) {
                principals.remove(&caller);
                
                if principals.is_empty() {
                    store.likes.remove(&request.content_id);
                }
            }
        });
        
        // Update author stats
        if let Some(author) = content_author {
            STORAGE.with(|storage| {
                let mut store = storage.borrow_mut();
                if let Some(stats) = store.user_stats.as_mut().and_then(|stats| stats.get_mut(&author)) {
                    if stats.like_count > 0 {
                        stats.like_count -= 1;
                    }
                }
            });
        }
        
        return Ok(InteractionResponse {
            success: true,
            message: "Content unliked successfully".to_string(),
        });
    } else {
        // Add like using main storage
        STORAGE.with(|storage| {
            let mut store = storage.borrow_mut();
            // Add like to main storage
            // likes 是 HashMap<String, HashSet<Principal>>
            store.likes.entry(request.content_id.clone())
                .or_insert_with(HashSet::new)
                .insert(caller);
        });
        
        // Update author stats
        if let Some(author) = content_author {
            STORAGE.with(|storage| {
                let mut store = storage.borrow_mut();
                if let Some(stats) = store.user_stats.as_mut().and_then(|stats| stats.get_mut(&author)) {
                    stats.like_count += 1;
                }
            });
        }
        
        return Ok(InteractionResponse {
            success: true,
            message: "Content liked successfully".to_string(),
        })
    }
}

// Report functionality
pub fn report_content(request: ReportContentRequest, caller: Principal) -> SquareResult<()> {
    const MODULE: &str = "services::interaction";
    const FUNCTION: &str = "report_content";
    
    // Verify content exists in main storage
    match request.content_type {
        ContentType::Post => {
            let post_exists = STORAGE.with(|storage| {
                let store = storage.borrow();
                store.posts.contains_key(&request.content_id)
            });
            
            if !post_exists {
                return log_and_return(not_found_error(
                    "Post", 
                    &request.content_id, 
                    MODULE, 
                    FUNCTION
                ));
            }
        },
        ContentType::Comment => {
            let comment_exists = STORAGE.with(|storage| {
                let store = storage.borrow();
                store.comments.contains_key(&request.content_id)
            });
            
            if !comment_exists {
                return log_and_return(not_found_error(
                    "Comment", 
                    &request.content_id, 
                    MODULE, 
                    FUNCTION
                ));
            }
        },
    }
    
    // Generate report ID based on timestamp
    let current_time = time() / 1_000_000;
    let report_id = format!("report_{}", current_time / 1_000_000);
    
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
        created_at: current_time,
        resolved_at: None,
        resolver: None,
        resolution_notes: None,
    };
    
    // Store in main storage
    STORAGE.with(|storage| {
        let mut store = storage.borrow_mut();
        
        // Add report to storage
        store.reports.insert(report_id.clone(), report.clone());
    });
    
    // For now, just set content under review if it's the first report
    match request.content_type {
        ContentType::Post => {
            STORAGE.with(|storage| {
                let mut store = storage.borrow_mut();
                if let Some(mut post) = store.posts.get(&request.content_id).cloned() {
                    if post.status == ContentStatus::Active {
                        post.status = ContentStatus::UnderReview;
                        store.posts.insert(request.content_id.clone(), post);
                    }
                }
            });
        },
        ContentType::Comment => {
            STORAGE.with(|storage| {
                let mut store = storage.borrow_mut();
                if let Some(mut comment) = store.comments.get(&request.content_id).cloned() {
                    if comment.status == ContentStatus::Active {
                        comment.status = ContentStatus::UnderReview;
                        store.comments.insert(request.content_id.clone(), comment);
                    }
                }
            });
        },
    }
    
    Ok(())
}

// Resolve report (admin function)
pub fn resolve_report(request: ResolveReportRequest, caller: Principal) -> SquareResult<InteractionResponse> {
    // Check if caller is admin or manager
    is_manager_or_admin()?;
    
    // Get report from main storage
    let report = STORAGE.with(|storage| {
        let store = storage.borrow();
        store.reports.get(&request.report_id).cloned()
    });
    
    // Check if report exists
    let report_info = report
        .ok_or_else(|| SquareError::NotFound(format!("Report not found: {}", request.report_id)))?;
    
    // Check if report is already resolved
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
                STORAGE.with(|storage| {
                    let mut store = storage.borrow_mut();
                    if let Some(mut post) = store.posts.get(&content_id).cloned() {
                        post.status = ContentStatus::Removed;
                        store.posts.insert(content_id.clone(), post);
                    }
                });
            },
            ContentType::Comment => {
                STORAGE.with(|storage| {
                    let mut store = storage.borrow_mut();
                    if let Some(mut comment) = store.comments.get(&content_id).cloned() {
                        comment.status = ContentStatus::Removed;
                        store.comments.insert(content_id.clone(), comment);
                    }
                });
            },
        }
    } else if request.status == ReportStatus::Rejected {
        // Report rejected, content remains active
        match content_type {
            ContentType::Post => {
                STORAGE.with(|storage| {
                    let mut store = storage.borrow_mut();
                    if let Some(mut post) = store.posts.get(&content_id).cloned() {
                        if post.status == ContentStatus::UnderReview {
                            post.status = ContentStatus::Active;
                            store.posts.insert(content_id.clone(), post);
                        }
                    }
                });
            },
            ContentType::Comment => {
                STORAGE.with(|storage| {
                    let mut store = storage.borrow_mut();
                    if let Some(mut comment) = store.comments.get(&content_id).cloned() {
                        if comment.status == ContentStatus::UnderReview {
                            comment.status = ContentStatus::Active;
                            store.comments.insert(content_id.clone(), comment);
                        }
                    }
                });
            },
        }
    }
    
    // Update report in main storage
    let current_time = time() / 1_000_000;
    let mut updated_report = report_info.clone();
    updated_report.status = request.status.clone();
    updated_report.resolved_at = Some(current_time);
    updated_report.resolver = Some(caller);
    updated_report.resolution_notes = request.notes.clone();
    
    STORAGE.with(|storage| {
        let mut store = storage.borrow_mut();
        // 更新报告
        store.reports.insert(request.report_id.clone(), updated_report);
    });
    
    Ok(InteractionResponse {
        success: true,
        message: format!("Report {} successfully", 
            if request.status == ReportStatus::Resolved { "resolved" } else { "rejected" }),
    })
}

// Get interaction counts
#[allow(dead_code)]
pub fn get_interaction_counts(content_id: String, caller: Option<Principal>) -> SquareResult<InteractionCountsResponse> {
    // Get likes count from main storage
    let likes_count = STORAGE.with(|storage| {
        let store = storage.borrow();
        store.likes.iter()
            .filter(|(content_id_ref, _)| **content_id_ref == content_id)
            .count() as u64
    });
    
    // Check if caller liked the content using main storage
    let is_liked_by_caller = match caller {
        Some(principal) => STORAGE.with(|storage| {
            let store = storage.borrow();
            store.likes.iter().any(|(content_id_ref, principal_set)| {
                **content_id_ref == content_id && principal_set.contains(&principal)
            })
        }),
        None => false,
    };
    
    // Get comments count from main storage (assuming content is a post)
    let comments_count = STORAGE.with(|storage| {
        let store = storage.borrow();
        store.comments.iter()
            .filter(|(_, comment)| 
                comment.parent_id == content_id && 
                comment.parent_type == ParentType::Post &&
                comment.status == ContentStatus::Active
            )
            .count() as u64
    });
    
    Ok(InteractionCountsResponse {
        likes: likes_count,
        comments: comments_count,
        is_liked_by_caller,
    })
}

// View tracking
#[allow(dead_code)]
pub fn view_content(request: ViewContentRequest, _caller: Principal) -> SquareResult<()> {
    // Get content author from main storage
    let content_author = match request.content_type {
        ContentType::Post => {
            // Get post from main storage
            STORAGE.with(|storage| {
                let store = storage.borrow();
                store.posts.get(&request.content_id).map(|post| post.author)
            })
        },
        ContentType::Comment => return Ok(()), // Don't track views for comments
    };
    
    // Update author stats in main storage
    if let Some(author) = content_author {
        STORAGE.with(|storage| {
            let mut store = storage.borrow_mut();
            
            if store.user_stats.is_none() {
                store.user_stats = Some(HashMap::new());
            }
            
            let author_principal = author;
            
            if let Some(stats_map) = &mut store.user_stats {
                let mut user_stats = stats_map.get(&author_principal).cloned().unwrap_or_else(|| {
                    UserStats {
                        principal: author_principal,
                        post_count: 0,
                        comment_count: 0,
                        like_count: 0,
                        points: 0,
                        reputation: 0,
                    }
                });
                
                user_stats.reputation += 1;
                
                stats_map.insert(author_principal, user_stats);
            }
        });
    }
    
    Ok(())
}
