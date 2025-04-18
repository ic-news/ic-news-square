use candid::Principal;
use ic_cdk::api::time;
use std::collections::HashMap;

use crate::auth::is_manager_or_admin;
use crate::models::reward;
use crate::models::reward::*;
use crate::models::error::{SquareError, SquareResult};
use crate::storage::{STORAGE, UserRewards, UserTasks, PointsTransaction};
use crate::storage::sharded::{SHARDED_USER_REWARDS, SHARDED_USER_TASKS};
use crate::utils::error_handler::*;

// Helper function to calculate user level based on points
fn calculate_level(points: u64) -> u64 {
    // Simple level calculation: level = 1 + (points / 100)
    // This means every 100 points, user gains a level
    // Level 1 is the starting level (0-99 points)
    1 + (points / 100)
}

// Initialize default tasks with configurable active state
pub fn init_default_tasks(enable_daily_post: bool, enable_weekly_article: bool, enable_social_engagement: bool) {
    STORAGE.with(|storage| {
        let mut storage = storage.borrow_mut();
        
        // Only initialize if no tasks exist yet
        if storage.tasks.is_empty() {
            // Also initialize sharded storage for tasks
            // This ensures we're using sharded storage by default
            // Daily post task
            let daily_post = TaskDefinition {
                id: "daily_post".to_string(),
                title: "Daily Post".to_string(),
                description: "Create a post daily to earn points".to_string(),
                points: 50,
                task_type: TaskType::Daily,
                completion_criteria: "Create at least one post".to_string(),
                expiration_time: None,
                created_at: time(),
                updated_at: time(),
                is_active: enable_daily_post,
            };
            
            // Weekly article task
            let weekly_article = TaskDefinition {
                id: "weekly_article".to_string(),
                title: "Weekly Article".to_string(),
                description: "Create an article weekly to earn points".to_string(),
                points: 200,
                task_type: TaskType::Weekly,
                completion_criteria: "Create at least one article per week".to_string(),
                expiration_time: None,
                created_at: time(),
                updated_at: time(),
                is_active: enable_weekly_article,
            };
            
            // Social engagement task
            let social_engagement = TaskDefinition {
                id: "social_engagement".to_string(),
                title: "Social Engagement".to_string(),
                description: "Like or comment on at least 10 posts".to_string(),
                points: 100,
                task_type: TaskType::Weekly,
                completion_criteria: "Like or comment on 10 posts".to_string(),
                expiration_time: None,
                created_at: time(),
                updated_at: time(),
                is_active: enable_social_engagement,
            };
            
            // Add tasks to storage
            storage.tasks.insert(daily_post.id.clone(), daily_post);
            storage.tasks.insert(weekly_article.id.clone(), weekly_article);
            storage.tasks.insert(social_engagement.id.clone(), social_engagement);
        }
    });
}

// Default initialization with all tasks enabled
pub fn init_default_tasks_all_enabled() {
    init_default_tasks(true, true, true);
}

// Toggle task active status
pub fn toggle_task_status(task_id: &str, active: bool) -> SquareResult<()> {
    // Only managers or admins can toggle task status
    is_manager_or_admin().map_err(|e| {
        SquareError::Unauthorized(format!("Only managers or admins can toggle task status: {}", e))
    })?;
    
    STORAGE.with(|storage| {
        let mut storage = storage.borrow_mut();
        
        // Check if task exists
        if let Some(mut task) = storage.tasks.get(task_id).cloned() {
            // Update task status
            task.is_active = active;
            task.updated_at = time();
            
            // Save updated task
            storage.tasks.insert(task_id.to_string(), task);
            Ok(())
        } else {
            Err(SquareError::NotFound(format!("Task with ID {} not found", task_id)))
        }
    })
}

// Daily check-in
pub fn claim_daily_check_in(caller: Principal) -> SquareResult<DailyCheckInResponse> {
    STORAGE.with(|storage| {
        let mut storage = storage.borrow_mut();
        
        // First check if user rewards exist, if not create them
        if !storage.user_rewards.contains_key(&caller) {
            storage.user_rewards.insert(caller, UserRewards {
                principal: caller,
                points: 0,
                points_history: Vec::new(),
                last_claim_date: None,
                consecutive_daily_logins: 0,
                level: 1,
                transactions: Vec::new(),
                last_updated: time(),
            });
        }
        
        // First check if user tasks exist, if not create them
        if !storage.user_tasks.contains_key(&caller) {
            storage.user_tasks.insert(caller, UserTasks {
                principal: caller,
                completed_tasks: HashMap::new(),
                daily_tasks_reset: time()/ 1_000_000,
                last_check_in: None,
                last_updated: time(),
            });
        }
        
        // Get user tasks info before mutable borrow
        let last_check_in = storage.user_tasks.get(&caller).unwrap().last_check_in;
        let consecutive_daily_logins = storage.user_rewards.get(&caller).unwrap().consecutive_daily_logins;
        
        // Check if already claimed today
        let now = time()/ 1_000_000;
        let today_start = now - (now % SECONDS_IN_DAY);
        
        if let Some(last_check) = last_check_in {
            let last_check_in_day = last_check - (last_check % SECONDS_IN_DAY);
            
            if last_check_in_day == today_start {
                return Err(SquareError::InvalidOperation("Already claimed daily check-in today".to_string()));
            }
        }
        
        // Calculate consecutive days
        let mut consecutive_days = consecutive_daily_logins;
        let mut bonus_points = 0;
        
        if let Some(last_claim) = last_check_in {
            let yesterday_start = today_start - SECONDS_IN_DAY;
            let last_claim_day = last_claim - (last_claim % SECONDS_IN_DAY);
            
            if last_claim_day == yesterday_start {
                // Consecutive day
                consecutive_days += 1;
                
                // Apply bonus for consecutive days (capped at MAX_CONSECUTIVE_BONUS_DAYS)
                let bonus_multiplier = if consecutive_days > MAX_CONSECUTIVE_BONUS_DAYS {
                    MAX_CONSECUTIVE_BONUS_DAYS
                } else {
                    consecutive_days
                };
                
                bonus_points = (DAILY_CHECK_IN_POINTS * bonus_multiplier) / CONSECUTIVE_DAYS_BONUS_MULTIPLIER;
            } else {
                // Streak broken
                consecutive_days = 1;
            }
        } else {
            // First check-in
            consecutive_days = 1;
        }
        
        // Update user rewards
        let total_points = DAILY_CHECK_IN_POINTS + bonus_points;
        
        // Get mutable references to update the data
        {
            let user_rewards = storage.user_rewards.get_mut(&caller).unwrap();
            user_rewards.points += total_points;
            user_rewards.consecutive_daily_logins = consecutive_days;
            user_rewards.last_claim_date = Some(now);
            
            // Add points transaction
            user_rewards.points_history.push(PointsTransaction {
                amount: total_points as i64,
                reason: format!("Daily check-in (Day {})", consecutive_days),
                timestamp: now,
                reference_id: None,
                points: total_points,
            });
        }
        
        // Update user tasks in a separate scope to avoid multiple mutable borrows
        {
            let user_tasks = storage.user_tasks.get_mut(&caller).unwrap();
            user_tasks.last_check_in = Some(now);
        }
        
        // Calculate next claim time
        let next_claim_available_at = today_start + SECONDS_IN_DAY;
        
        Ok(DailyCheckInResponse {
            success: true,
            points_earned: DAILY_CHECK_IN_POINTS,
            consecutive_days,
            bonus_points,
            total_points,
            next_claim_available_at,
        })
    })
}

// Task completion
pub fn complete_task(request: CompleteTaskRequest, caller: Principal) -> SquareResult<TaskCompletionResponse> {
    const MODULE: &str = "services::reward";
    const FUNCTION: &str = "complete_task";
    
    // First check if proof is provided, all tasks must have proof
    let _proof = match &request.proof {
        Some(proof) => {
            // Check if the proof format is valid
            // For daily_post and weekly_article, we need specific proof formats
            // For other tasks, we accept any proof
            if request.task_id == "daily_post" && !proof.contains("post_") && !proof.contains("simple") {
                return log_and_return(validation_error(
                    &format!("Invalid proof format for daily_post: {}", proof),
                    MODULE,
                    FUNCTION
                ));
            } else if request.task_id == "weekly_article" && !proof.contains("article_") && !proof.contains("simple") {
                return log_and_return(validation_error(
                    &format!("Invalid proof format for weekly_article: {}", proof),
                    MODULE,
                    FUNCTION
                ));
            }
            
            // If we reach here, the proof is valid
            proof
        },
        None => {
            // For social_engagement and other tasks, we can accept null proof
            // Only daily_post and weekly_article require specific proof formats
            if request.task_id == "daily_post" || request.task_id == "weekly_article" {
                return log_and_return(validation_error(
                    &format!("Proof is required for {} task", request.task_id),
                    MODULE,
                    FUNCTION
                ));
            }
            
            // For other tasks, use a default proof
            "simple"
        }
    };
    
    STORAGE.with(|storage| {
        let storage = storage.borrow_mut();
        
        // First, check if the task exists and get its reward points
        // We need to do this before borrowing user_tasks to avoid borrowing conflicts
        
        // Check if the task has an expiration time and if it has expired
        // This check needs to happen for ALL tasks, including custom tasks
        let task_opt = storage.tasks.get(&request.task_id);
        if let Some(task) = task_opt {
            if let Some(expiry) = task.expiration_time {
                if expiry < time() {
                    return Err(SquareError::InvalidOperation(format!("Task has expired: {}", request.task_id)));
                }
            }
        }
        
        // Now get the task details
        let (task_reward, task_type, expiration_time) = match task_opt {
            Some(task) => {
                (task.points, task.task_type.clone(), task.expiration_time)
            },
            None => {
                // Check for hardcoded tasks from get_available_tasks
                match request.task_id.as_str() {
                    "daily_post" => (50, TaskType::Daily, None),
                    "weekly_article" => (200, TaskType::Weekly, None),
                    "social_engagement" => (100, TaskType::Weekly, None),
                    _ => return Err(SquareError::NotFound(format!("Task with ID {} not found", request.task_id)))
                }
            }
        };
        
        // Get user tasks
        let user_tasks_key = caller.to_string();
        let user_tasks_exists = SHARDED_USER_TASKS.with(|tasks| {
            tasks.borrow().contains_key(&user_tasks_key)
        });
        
        if !user_tasks_exists {
            // Initialize user tasks in sharded storage
            SHARDED_USER_TASKS.with(|tasks| {
                let mut tasks = tasks.borrow_mut();
                tasks.insert(user_tasks_key.clone(), UserTasks {
                    principal: caller,
                    completed_tasks: HashMap::new(),
                    daily_tasks_reset: time() / 1_000_000,
                    last_check_in: None,
                    last_updated: time(),
                });
            });
        }
        
        // We already have the task_type from earlier, no need to fetch it again
        
        // Check if task already completed
        let already_completed = SHARDED_USER_TASKS.with(|tasks| {
            let mut tasks = tasks.borrow_mut();
            if let Some(user_tasks) = tasks.get(&user_tasks_key) {
                if user_tasks.completed_tasks.contains_key(&request.task_id) {
                    match task_type {
                        TaskType::Daily => {
                            // For daily tasks, check if it was completed today
                            let completion_time = *user_tasks.completed_tasks.get(&request.task_id).unwrap();
                            let now = time() / 1_000_000;
                            let today_start = now - (now % SECONDS_IN_DAY);
                            let completion_day = completion_time / 1_000_000 - (completion_time / 1_000_000 % SECONDS_IN_DAY);
                            
                            // If completed on a different day, allow completion again
                            completion_day == today_start
                        },
                        TaskType::Weekly => {
                            // For weekly tasks, check if it was completed this week
                            let completion_time = *user_tasks.completed_tasks.get(&request.task_id).unwrap();
                            let now = time() / 1_000_000;
                            let week_start = now - (now % (SECONDS_IN_DAY * 7));
                            let completion_week = completion_time / 1_000_000 - (completion_time / 1_000_000 % (SECONDS_IN_DAY * 7));
                            
                            // If completed in a different week, allow completion again
                            completion_week == week_start
                        },
                        _ => true // For one-time tasks, always consider as completed
                    }
                } else {
                    false
                }
            } else {
                false
            }
        });
        
        if already_completed {
            return Err(SquareError::InvalidOperation(format!("Task already completed: {}", request.task_id)));
        }
        
        // We've already validated the proof in the previous step
        // No need for additional validation here
        // This allows for more flexibility in task types and proof formats
        
        // Mark task as completed
        SHARDED_USER_TASKS.with(|tasks| {
            let mut tasks = tasks.borrow_mut();
            let user_tasks = match tasks.get(&user_tasks_key) {
                Some(tasks) => tasks.clone(),
                None => UserTasks {
                    principal: caller,
                    completed_tasks: HashMap::new(),
                    daily_tasks_reset: time() / 1_000_000,
                    last_check_in: None,
                    last_updated: time(),
                }
            };
            
            let mut updated_tasks = user_tasks.clone();
            updated_tasks.completed_tasks.insert(request.task_id.clone(), time());
            updated_tasks.last_updated = time();
            
            tasks.insert(user_tasks_key.clone(), updated_tasks);
        });
        
        // Award points
        let user_rewards_key = caller.to_string();
        let (points, level, level_up) = SHARDED_USER_REWARDS.with(|rewards| {
            let mut rewards = rewards.borrow_mut();
            let user_rewards = match rewards.get(&user_rewards_key) {
                Some(rewards) => rewards.clone(),
                None => UserRewards {
                    principal: caller,
                    points: 0,
                    points_history: Vec::new(),
                    last_claim_date: None,
                    consecutive_daily_logins: 0,
                    level: 1,
                    transactions: Vec::new(),
                    last_updated: time(),
                }
            };
            
            let mut updated_rewards = user_rewards.clone();
            
            // Add points
            updated_rewards.points += task_reward;
            
            // Update level if needed
            let new_level = calculate_level(updated_rewards.points);
            let level_up = new_level > updated_rewards.level;
            updated_rewards.level = new_level;
            
            // Record transaction
            updated_rewards.points_history.push(PointsTransaction {
                amount: task_reward as i64,
                reason: format!("Completed task: {}", request.task_id),
                timestamp: time(),
                reference_id: Some(request.task_id.clone()),
                points: task_reward,
            });
            
            updated_rewards.last_updated = time();
            
            rewards.insert(user_rewards_key.clone(), updated_rewards.clone());
            
            (updated_rewards.points, updated_rewards.level, level_up)
        });
        
        // Return the response
        Ok(TaskCompletionResponse {
            success: true,
            points_earned: task_reward,
            total_points: points,
            level: level,
            level_up,
            message: format!("You earned {} points for completing {}", task_reward, request.task_id),
        })
    })
}

// Get user rewards
pub fn get_user_rewards(principal: Principal) -> SquareResult<UserRewardsResponse> {
    const MODULE: &str = "services::reward";
    const FUNCTION: &str = "get_user_rewards";
    
    let user_rewards = SHARDED_USER_REWARDS.with(|rewards| {
        let mut rewards = rewards.borrow_mut();
        rewards.get(&principal.to_string()).map(|r| r.clone())
    });
    
    let user_rewards = match user_rewards {
            Some(rewards) => rewards.clone(),
            None => {
                // If user rewards record doesn't exist, check user tasks record
                let has_completed_tasks = SHARDED_USER_TASKS.with(|tasks| {
                    let mut tasks = tasks.borrow_mut();
                    let user_tasks_opt = tasks.get(&principal.to_string()).map(|t| t.clone());
                    if let Some(user_tasks) = user_tasks_opt {
                        // If user has completed tasks but rewards record doesn't exist, return empty rewards record
                        if !user_tasks.completed_tasks.is_empty() {
                            let completed_task_ids = user_tasks.completed_tasks.keys().cloned().collect();
                            return Some(completed_task_ids);
                        }
                    }
                    None
                });
                
                if let Some(completed_tasks) = has_completed_tasks {
                    return Ok(UserRewardsResponse {
                        points: 0,
                        level: 0,
                        completed_tasks,
                        points_history: Vec::new(),
                    });
                }
                
                // If user hasn't completed any tasks or tasks record doesn't exist, create a new rewards record
                // Initialize a new user rewards record
                let new_user_rewards = UserRewards {
                    principal: principal.clone(),
                    points: 0,
                    points_history: Vec::new(),
                    last_claim_date: None,
                    consecutive_daily_logins: 0,
                    level: 0,
                    transactions: Vec::new(),
                    last_updated: time(),
                };
                
                // Store the new rewards record
                SHARDED_USER_REWARDS.with(|rewards| {
                    let mut rewards = rewards.borrow_mut();
                    rewards.insert(principal.to_string(), new_user_rewards.clone());
                });
                
                // Also create an empty user tasks record if it doesn't exist
                SHARDED_USER_TASKS.with(|tasks| {
                    let mut tasks = tasks.borrow_mut();
                    if !tasks.contains_key(&principal.to_string()) {
                        tasks.insert(principal.to_string(), UserTasks {
                            principal: principal.clone(),
                            completed_tasks: HashMap::new(),
                            daily_tasks_reset: time() / 1_000_000,
                            last_check_in: None,
                            last_updated: time(),
                        });
                    }
                });
                
                // Return the newly created rewards
                return Ok(UserRewardsResponse {
                    points: 0,
                    level: 0,
                    completed_tasks: Vec::new(),
                    points_history: Vec::new(),
                });
            }
        };
            
        // Calculate user level (simple example: level up every 100 points)
        let level = user_rewards.points / 100;
        
        // Get user's completed tasks
        let completed_tasks = SHARDED_USER_TASKS.with(|tasks| {
            let mut tasks = tasks.borrow_mut();
            let user_tasks_opt = tasks.get(&principal.to_string()).map(|tasks| tasks.clone());
            match user_tasks_opt {
            Some(user_tasks) => user_tasks.completed_tasks.keys().cloned().collect(),
            None => Vec::new(),
            }
        });
        
        // Convert storage_main::PointsTransaction to models::reward::PointsTransaction
        let points_history: Vec<reward::PointsTransaction> = user_rewards.points_history
            .iter()
            .map(|tx| reward::PointsTransaction {
                amount: tx.amount,
                reason: tx.reason.clone(),
                timestamp: tx.timestamp,
                reference_id: tx.reference_id.clone(),
            })
            .collect();
        
        Ok(UserRewardsResponse {
            points: user_rewards.points,
            level,
            completed_tasks,
            points_history,
    })
}

// Get available tasks
pub fn get_available_tasks(caller: Principal) -> SquareResult<Vec<TaskResponse>> {
    const MODULE: &str = "services::reward";
    const FUNCTION: &str = "get_available_tasks";
    
    
    STORAGE.with(|storage| {
        let storage = storage.borrow();
        
        // Get user tasks
        let user_tasks = storage.user_tasks.get(&caller);
        
        // Get all task definitions from storage
        let mut tasks = Vec::new();
        let now = time() / 1_000_000;
        
        // 移除日志调用以节省 cycles
        
        // Iterate through all task definitions
        for (task_id, task_def) in &storage.tasks {
            // Check if task is active
            if !task_def.is_active {
                continue;
            }
            
            // Check if task has expired
            if let Some(expiration) = task_def.expiration_time {
                if now > expiration {
                    continue;
                }
            }
            
            // Check if task is already completed (for daily tasks, check if completed today)
            let is_completed = if let Some(ut) = user_tasks {
                if task_def.task_type == TaskType::Daily {
                    // For daily tasks, check if completed today
                    let today_start = now - (now % SECONDS_IN_DAY);
                    ut.completed_tasks.get(task_id)
                        .map(|completion_time| *completion_time >= today_start)
                        .unwrap_or(false)
                } else if task_def.task_type == TaskType::Weekly {
                    // For weekly tasks, check if completed this week
                    let week_start = now - (now % (SECONDS_IN_DAY * 7));
                    ut.completed_tasks.get(task_id)
                        .map(|completion_time| *completion_time >= week_start)
                        .unwrap_or(false)
                } else if task_def.task_type == TaskType::Monthly {
                    // For monthly tasks, check if completed this month (approximate)
                    let month_start = now - (now % (SECONDS_IN_DAY * 30));
                    ut.completed_tasks.get(task_id)
                        .map(|completion_time| *completion_time >= month_start)
                        .unwrap_or(false)
                } else {
                    // For one-time tasks, check if ever completed
                    ut.completed_tasks.contains_key(task_id)
                }
            } else {
                false
            };
            
            // Create task response
            tasks.push(TaskResponse {
                id: task_id.clone(),
                title: task_def.title.clone(),
                description: task_def.description.clone(),
                points: task_def.points,
                task_type: task_def.task_type.clone(),
                is_completed,
                completion_criteria: task_def.completion_criteria.clone(),
                expiration_time: task_def.expiration_time,
                created_at: task_def.created_at,
            });
        }
        
        // If no tasks found, add default tasks
        if tasks.is_empty() {
            // Initialize tasks if they don't exist yet
            init_default_tasks_all_enabled();
            
            // Try again with default tasks
            return get_available_tasks(caller);
        }
        
        Ok(tasks)
    })
}

// Admin functions
pub fn award_points(request: AwardPointsRequest) -> SquareResult<()> {
    // Check if caller is admin or manager
    is_manager_or_admin()?;
    
    let user_rewards_key = request.principal.to_string();
    let (_points, _level, _level_up) = SHARDED_USER_REWARDS.with(|rewards| {
        let mut rewards = rewards.borrow_mut();
        let user_rewards = match rewards.get(&user_rewards_key) {
            Some(rewards) => rewards.clone(),
            None => UserRewards {
            principal: request.principal,
            points: 0,
            points_history: Vec::new(),
            last_claim_date: None,
            consecutive_daily_logins: 0,
            level: 1,
            transactions: Vec::new(),
            last_updated: time(),
        }
        };
        
        let mut updated_rewards = user_rewards.clone();
        
        // Add points
        updated_rewards.points += request.points;
        
        // Update level if needed
        let new_level = calculate_level(updated_rewards.points);
        let level_up = new_level > updated_rewards.level;
        updated_rewards.level = new_level;
        
        // Record transaction
        updated_rewards.points_history.push(PointsTransaction {
            amount: request.points as i64,
            reason: request.reason.clone(),
            timestamp: time(),
            reference_id: request.reference_id.clone(),
            points: request.points,
        });
        
        updated_rewards.last_updated = time();
        
        rewards.insert(user_rewards_key.clone(), updated_rewards.clone());
        
        (updated_rewards.points, updated_rewards.level, level_up)
    });
    
    Ok(())
}

// Task management (admin functions)
pub fn create_task(request: CreateTaskRequest) -> SquareResult<String> {
    const MODULE: &str = "services::reward";
    const FUNCTION: &str = "create_task";
    
    
    // Check if caller is admin or manager
    is_manager_or_admin().map_err(|e| {
        e
    })?;
    
    let task_id = if !request.id.is_empty() {
        request.id.clone()
    } else {
        format!("task_{}", time() / 1_000_000)
    };
    
    let now = time() / 1_000_000;
    
    // Create task definition
    // Use points_reward field for task points
    let points = request.points_reward;
    
    let task = TaskDefinition {
        id: task_id.clone(),
        title: request.title,
        description: request.description,
        points,
        task_type: request.task_type,
        completion_criteria: request.completion_criteria,
        expiration_time: request.end_time,
        created_at: now,
        updated_at: now,
        is_active: true,
    };
    
    // Store the task
    STORAGE.with(|storage| {
        let mut storage = storage.borrow_mut();
        
        // Check if task ID already exists
        if storage.tasks.contains_key(&task_id) {
            return log_and_return(already_exists_error(
                "Task", 
                &task_id, 
                MODULE, 
                FUNCTION
            ).with_details(format!("Task with ID {} already exists", task_id)));
        }
        
        // Add the task
        storage.tasks.insert(task_id.clone(), task);
        Ok(task_id)
    })
}

pub fn update_task(request: UpdateTaskRequest) -> SquareResult<()> {
    const MODULE: &str = "services::reward";
    const FUNCTION: &str = "update_task";
    
    
    // Check if caller is admin or manager
    is_manager_or_admin().map_err(|e| {
        e
    })?;
    
    STORAGE.with(|storage| {
        let mut storage = storage.borrow_mut();
        
        // Check if task exists
        let task = match storage.tasks.get_mut(&request.id) {
            Some(task) => task,
            None => return log_and_return(not_found_error(
                "Task", 
                &request.id, 
                MODULE, 
                FUNCTION
            ).with_details(format!("Task with ID {} not found", request.id))),
        };
        
        // Update task fields
        task.title = request.title;
        task.description = request.description;
        // Use points_reward field for task points
        task.points = request.points_reward;
        task.task_type = request.task_type;
        task.completion_criteria = request.completion_criteria;
        task.expiration_time = request.end_time;
        task.updated_at = time() / 1_000_000;
        
        Ok(())
    })
}

pub fn delete_task(task_id: String) -> SquareResult<()> {
    const MODULE: &str = "services::reward";
    const FUNCTION: &str = "delete_task";


    // Check if caller is admin or manager
    is_manager_or_admin().map_err(|e| {
        e
    })?;

    STORAGE.with(|storage| {
        let mut storage = storage.borrow_mut();

        // Check if task exists
        if !storage.tasks.contains_key(&task_id) {
            return log_and_return(not_found_error(
                "Task",
                &task_id,
                MODULE,
                FUNCTION
            ).with_details(format!("Task with ID {} not found", task_id)));
        }

        // Remove the task
        storage.tasks.remove(&task_id);
        Ok(())
    })
}
