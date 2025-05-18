use candid::{CandidType, Deserialize, Encode, Nat, Principal};
use ic_cdk::api::{time, call};
use ic_cdk::id;
use std::collections::HashMap;
use crate::Value;
use crate::auth::is_manager_or_admin;
use crate::models::reward::*;
use crate::models::error::{SquareError, SquareResult};
use crate::storage::STORAGE;
use crate::utils::error_handler::*;

// Initialize default tasks with configurable active state
pub fn init_default_tasks(enable_daily_post: bool, enable_social_engagement: bool) {
    const MODULE: &str = "services::reward";
    const FUNCTION: &str = "init_default_tasks";
    let now = time() / 1_000_000;
    // Check if tasks already exist in main storage
    let tasks_exist = STORAGE.with(|storage| {
        let store = storage.borrow();
        store.tasks.as_ref().map_or(false, |tasks| !tasks.is_empty())
    });
    
    // Only initialize if no tasks exist yet
    if !tasks_exist {
        // Daily post task
        let daily_post = TaskDefinition {
            id: "daily_post".to_string(),
            title: "Daily Post".to_string(),
            description: "Create a post daily to earn points".to_string(),
            points: 50,
            task_type: TaskType::Daily,
            completion_criteria: "Create at least one post".to_string(),
            expiration_time: None,
            created_at: now,
            updated_at: now,
            is_active: enable_daily_post,
            requirements: None,
            canister_id: ic_cdk::id(),
        };
        
        // Social engagement task
        let social_engagement = TaskDefinition {
            id: "social_engagement".to_string(),
            title: "Social Engagement".to_string(),
            description: "Engage with other users to earn points".to_string(),
            points: 100,
            task_type: TaskType::Daily,
            completion_criteria: "Like or comment on at least 3 posts".to_string(),
            expiration_time: None,
            created_at: now,
            updated_at: now,
            is_active: enable_social_engagement,
            requirements: None,
            canister_id: ic_cdk::id(),
        };
        
        // Add tasks to main storage
        STORAGE.with(|storage| {
            let mut store = storage.borrow_mut();
            if store.tasks.is_none() {
                store.tasks = Some(HashMap::new());
            }
            
            if let Some(tasks) = &mut store.tasks {
                tasks.insert(daily_post.id.clone(), daily_post);
                tasks.insert(social_engagement.id.clone(), social_engagement);
            }
        });
        
        ic_cdk::println!("Default tasks initialized in main storage");
    }
}

// Default initialization with all tasks enabled
pub fn init_default_tasks_all_enabled() {
    init_default_tasks(true, true);
}

// Initialize default tasks only if no tasks exist
pub fn init_default_tasks_if_empty() {
    // Check if tasks exist in main storage
    let tasks_exist = STORAGE.with(|storage| {
        let store = storage.borrow();
        store.tasks.as_ref().map_or(false, |tasks| !tasks.is_empty())
    });
    
    if !tasks_exist {
        ic_cdk::println!("Initializing default tasks");
        init_default_tasks_all_enabled();
    } else {
        ic_cdk::println!("Tasks already exist, skipping initialization");
    }
}

// Toggle task active status
pub fn toggle_task_status(task_id: &str, active: bool) -> SquareResult<()> {
    const MODULE: &str = "services::reward";
    const FUNCTION: &str = "toggle_task_status";
    let now = time()  / 1_000_000;
    // Only managers or admins can toggle task status
    is_manager_or_admin().map_err(|e| {
        SquareError::Unauthorized(format!("Only managers or admins can toggle task status: {}", e))
    })?;
    
    // Update task in main storage
    STORAGE.with(|storage| {
        let mut store = storage.borrow_mut();
        
        if let Some(tasks) = &mut store.tasks {
            // Check if task exists
            if let Some(mut task) = tasks.get(task_id).cloned() {
                // Update task status
                task.is_active = active;
                task.updated_at = now;
                
                // Save updated task
                tasks.insert(task_id.to_string(), task);
                Ok(())
            } else {
                Err(SquareError::NotFound(format!("Task with ID {} not found", task_id)))
            }
        } else {
            Err(SquareError::NotFound("Tasks collection not initialized".to_string()))
        }
    })
}

// Daily check-in
// Note: The daily check-in functionality has been moved to a separate canister
// See: canisters/daily_checkin_task/src/lib.rs
// Users should call that canister directly for daily check-ins

// Task completion
pub fn complete_task(request: CompleteTaskRequest, caller: Principal) -> SquareResult<TaskCompletionResponse> {
    const MODULE: &str = "services::reward";
    const FUNCTION: &str = "complete_task";
    let now = time() / 1_000_000;
    // Get task info from main storage
    let task_info = STORAGE.with(|storage| {
        let store = storage.borrow();
        store.tasks.as_ref()
            .and_then(|tasks| tasks.get(&request.task_id).cloned())
    });
    
    let _task_type = match task_info {
        Some(task) => task.task_type,
        None => {
            match request.task_id.as_str() {
                id if id.starts_with("daily_") => TaskType::Daily,
                id if id.starts_with("weekly_") => TaskType::Weekly,
                _ => TaskType::OneTime
            }
        }
    };
    
    let _proof = request.task_id.clone();
    
    // Check if the task has an expiration time and if it has expired in main storage
    // This check needs to happen for ALL tasks, including custom tasks
    // Check if task has already been completed today for daily tasks
    let already_completed = STORAGE.with(|storage| {
        let store = storage.borrow();
        if let Some(user_tasks) = store.user_tasks.get(&caller) {
            if request.task_id.starts_with("daily_") {
                // For daily tasks, check if completed today
                let today_start = (now / SECONDS_IN_DAY) * SECONDS_IN_DAY;
                if let Some(completion_time) = user_tasks.completed_tasks.get(&request.task_id) {
                    return *completion_time >= today_start;
                }
            } else {
                // For other tasks, check if ever completed
                return user_tasks.completed_tasks.contains_key(&request.task_id);
            }
        }
        false
    });

    if already_completed {
        return Err(SquareError::InvalidOperation(
            format!("Task {} has already been completed today", request.task_id)
        ));
    }

    // Check task expiration
    let task_expired = STORAGE.with(|storage| {
        let store = storage.borrow();
        if let Some(tasks) = &store.tasks {
            if let Some(task) = tasks.get(&request.task_id) {
                if let Some(expiry) = task.expiration_time {
                    // 确保时间单位一致，统一使用毫秒
                    let expiry_ms = expiry / 1_000_000;
                    if expiry_ms < now {
                        ic_cdk::println!("Task {} has expired: expiry={}, now={}", request.task_id, expiry_ms, now);
                        return true;
                    }
                }
                return false;
            }
        }
        
        // If task not found in store, check if it's a default task
        match request.task_id.as_str() {
            "daily_post" | "social_engagement" => false,
            _ => {
                ic_cdk::println!("Task {} not found and not a default task", request.task_id);
                true // Unknown task is considered expired
            }
        }
    });

    if task_expired {
        return Err(SquareError::InvalidOperation(
            format!("Task {} has expired or does not exist", request.task_id)
        ));
    }
        
    // Now get the task details from main storage
    let (task_reward, task_type, _expiration_time) = STORAGE.with(|storage| {
        let store = storage.borrow();
        if let Some(tasks) = &store.tasks {
            if let Some(task) = tasks.get(&request.task_id) {
                Ok((task.points, task.task_type.clone(), task.expiration_time))
            } else {
                // Check for hardcoded tasks from get_available_tasks
                match request.task_id.as_str() {
                    "daily_post" => Ok((50, TaskType::Daily, None)),
                    "social_engagement" => Ok((100, TaskType::Daily, None)),
                    _ => Err(SquareError::NotFound(format!("Task with ID {} not found", request.task_id)))
                }
            }
        } else {
            // Check for hardcoded tasks from get_available_tasks
            match request.task_id.as_str() {
                "daily_post" => Ok((50, TaskType::Daily, None)),
                "social_engagement" => Ok((100, TaskType::Daily, None)),
                _ => Err(SquareError::NotFound(format!("Task with ID {} not found", request.task_id)))
            }
        }
    })?;
    
    // Get user tasks
    let user_tasks_exists = STORAGE.with(|storage| {
        let store = storage.borrow();
        store.user_tasks.contains_key(&caller)
    });
    
    if !user_tasks_exists {
        // Initialize user tasks in main storage
        STORAGE.with(|storage| {
            let mut store = storage.borrow_mut();
            // Ensure user_tasks exists
            if store.user_tasks.is_empty() {
                store.user_tasks = HashMap::new();
            }
            
            // Add user task
            store.user_tasks.insert(caller, UserTasks {
                principal: caller,
                completed_tasks: HashMap::new(),
                daily_tasks_reset: now,
                last_check_in: None,
                last_updated: now,
            });
        });
    }
    
    // We already have the task_type from earlier, no need to fetch it again
    
    // Check if task already completed
    let already_completed = STORAGE.with(|storage| {
        let store = storage.borrow();
        // Check user tasks
        if let Some(user_tasks) = store.user_tasks.get(&caller) {
                match task_type {
                    TaskType::Daily => {
                        // For daily tasks, check if THIS SPECIFIC daily task was completed today
                        // Using day_id approach to prevent multiple completions of the same daily task in the same day
                        let current_day_id = time() / 1_000_000_000 / 86400;
                        
                        // Check if this specific task was completed today
                        if let Some(completion_time) = user_tasks.completed_tasks.get(&request.task_id) {
                            // Convert completion time to day_id
                            let completion_day_id = completion_time / 1_000_000_000 / 86400;
                            
                            // If the completion day_id matches current day_id, task was already completed today
                            if completion_day_id == current_day_id {
                                return true; // Already completed this specific daily task today
                            }
                        }
                        false
                    },
                    TaskType::Weekly => {
                        // For weekly tasks, check if ANY weekly task was completed this week
                        let now = time() / 1_000_000;
                        let week_start = now - (now % (SECONDS_IN_DAY * 7));
                        
                        // Check if this specific task was completed this week
                        if user_tasks.completed_tasks.contains_key(&request.task_id) {
                            let completion_time = *user_tasks.completed_tasks.get(&request.task_id).unwrap();
                            let completion_week = completion_time / 1_000_000 - (completion_time / 1_000_000 % (SECONDS_IN_DAY * 7));
                            if completion_week == week_start {
                                return true;
                            }
                        }
                        false
                    },
                    _ => user_tasks.completed_tasks.contains_key(&request.task_id) // For one-time tasks, check if this specific task was completed
                }
            } else {
                false
            }
    });
    
    if already_completed {
        return Err(SquareError::InvalidOperation(format!("Task already completed: {}", request.task_id)));
    }
    
    // Validate the proof based on task type
    if let Some(proof_str) = &request.proof {
        // For tasks that require proof, validate it
        match request.task_id.as_str() {
            "daily_post" => {
                if proof_str.is_empty() {
                    return Err(SquareError::ValidationFailed("Invalid proof for daily_post task".to_string()));
                }
            },
            _ => {
                // For other tasks, just ensure proof is not empty when provided
                if proof_str.is_empty() {
                    return Err(SquareError::ValidationFailed(format!("Invalid proof for task {}", request.task_id)));
                }
            }
        }
    } else {
        // Some tasks might not require proof
        match request.task_id.as_str() {
            "social_engagement" => {}, // No proof needed
            "daily_checkin" => {}, // No proof needed
            _ => {
                // For other tasks, proof is required
                return Err(SquareError::ValidationFailed(format!("Proof is required for task {}", request.task_id)));
            }
        }
    }
    
    // Mark task as completed
    STORAGE.with(|storage| {
        let mut store = storage.borrow_mut();
        
        // Ensure user_tasks exists
        if store.user_tasks.is_empty() {
            store.user_tasks = HashMap::new();
        }
        
        // Get user tasks
        let user_tasks = match store.user_tasks.get(&caller) {
                Some(tasks) => tasks.clone(),
                None => UserTasks {
                    principal: caller,
                    completed_tasks: HashMap::new(),
                    daily_tasks_reset: now,
                    last_check_in: None,
                    last_updated: now,
                }
            };
            
            let mut updated_tasks = user_tasks.clone();
            updated_tasks.completed_tasks.insert(request.task_id.clone(), now);
            updated_tasks.last_updated = now;
            
            store.user_tasks.insert(caller, updated_tasks);
    });
    
    // Award points
    let points = STORAGE.with(|storage| {
        let mut store = storage.borrow_mut();
        
        // Ensure user_rewards exists
        if store.user_rewards.is_empty() {
            store.user_rewards = HashMap::new();
        }
        
        // Get user rewards
        let user_rewards = match store.user_rewards.get(&caller) {
            Some(rewards) => rewards.clone(),
            None => UserRewards {
                principal: caller,
                points: 0,
                points_history: Vec::new(),
                last_claim_date: None,
                // consecutive_daily_logins field has been moved to daily_checkin_task canister
                transactions: Vec::new(),
                last_updated: now,
            }
        };
        
        let mut updated_rewards = user_rewards.clone();
        
        // Add points
        updated_rewards.points += task_reward;
        
        // Record transaction
        updated_rewards.points_history.push(PointsTransaction {
            amount: task_reward as i64,
            reason: format!("Completed task: {}", request.task_id),
            timestamp: now,
            reference_id: Some(request.task_id.clone()),
            points: task_reward,
        });
        
        updated_rewards.last_updated = now;
        
        // Save updated rewards and return points
        store.user_rewards.insert(caller, updated_rewards.clone());
        updated_rewards.points
    });
        
    // Return the response
    Ok(TaskCompletionResponse {
        success: true,
        points_earned: task_reward,
        total_points: points,
        message: format!("You earned {} points for completing {}", task_reward, request.task_id),
    })
}

// Get user rewards
pub fn get_user_rewards(principal: Principal) -> SquareResult<UserRewardsResponse> {
    const MODULE: &str = "services::reward";
    const FUNCTION: &str = "get_user_rewards";
    let now = time() / 1_000_000;
    // Create a new UserRewardsResponse
    let mut response = UserRewardsResponse::new();
    
    // Get user rewards from the main storage
    let user_rewards = STORAGE.with(|storage| {
        let store = storage.borrow();
        store.user_rewards.get(&principal).cloned()
    });
    
    let user_rewards = match user_rewards {
        Some(rewards) => rewards.clone(),
        None => {
            // If user rewards record doesn't exist, check user tasks record
            let has_completed_tasks = STORAGE.with(|storage| {
                let store = storage.borrow();
                let user_tasks_opt = store.user_tasks.get(&principal).cloned();
                if let Some(user_tasks) = user_tasks_opt {
                    // If user has completed tasks but rewards record doesn't exist, return empty rewards record
                    if !user_tasks.completed_tasks.is_empty() {
                        let completed_task_ids = user_tasks.completed_tasks.keys().cloned().collect::<Vec<String>>();
                        response.insert("completed_tasks_count".to_string(), Value::Nat(completed_task_ids.len() as u64));
                        
                        // Add completed tasks as an array
                        let task_ids: Vec<Value> = completed_task_ids.iter()
                            .map(|id| Value::Text(id.clone()))
                            .collect();
                        response.insert("completed_tasks".to_string(), Value::Array(task_ids));
                        
                        return Ok(response.clone());
                    }
                }
                Err(SquareError::NotFound("User tasks not found".to_string()))
            });
            
            if has_completed_tasks.is_ok() {
                return Ok(response);
            }
            
            // If user hasn't completed any tasks or tasks record doesn't exist, create a new rewards record
            // Initialize a new user rewards record
            let new_user_rewards = UserRewards {
                principal: principal.clone(),
                points: 0,
                points_history: Vec::new(),
                last_claim_date: None,
                // consecutive_daily_logins field has been moved to daily_checkin_task canister
                transactions: Vec::new(),
                last_updated: now,
            };
            
            // Store the new rewards record in main storage
            STORAGE.with(|storage| {
                let mut store = storage.borrow_mut();
                store.user_rewards.insert(principal, new_user_rewards.clone());
            });
            
            // Also create an empty user tasks record if it doesn't exist in main storage
            STORAGE.with(|storage| {
                let mut store = storage.borrow_mut();
                if !store.user_tasks.contains_key(&principal) {
                    store.user_tasks.insert(principal, UserTasks {
                        principal: principal.clone(),
                        completed_tasks: HashMap::new(),
                        daily_tasks_reset: now,
                        last_check_in: None,
                        last_updated: now,
                    });
                }
            });
            
            // Add basic info to the response
            response.insert("points".to_string(), Value::Nat(0));
        
            response.insert("completed_tasks_count".to_string(), Value::Nat(0));
            response.insert("completed_tasks".to_string(), Value::Array(Vec::new()));
            
            return Ok(response);
        }
    };
        

    
    // Get user's completed tasks from main storage
    let completed_tasks = STORAGE.with(|storage| {
        let store = storage.borrow();
        let user_tasks_opt = store.user_tasks.get(&principal).cloned();
        match user_tasks_opt {
            Some(user_tasks) => user_tasks.completed_tasks.keys().cloned().collect::<Vec<String>>(),
            None => Vec::new(),
        }
    });
    
    // Add main canister data to response
    response.insert("points".to_string(), Value::Nat(user_rewards.points));

    response.insert("completed_tasks_count".to_string(), Value::Nat(completed_tasks.len() as u64));
    
    // Add completed tasks as an array
    let task_ids: Vec<Value> = completed_tasks.iter()
        .map(|id| Value::Text(id.clone()))
        .collect();
    response.insert("completed_tasks".to_string(), Value::Array(task_ids));
    
    // Add points history count
    response.insert("points_history_count".to_string(), Value::Nat(user_rewards.points_history.len() as u64));
    
    // Add points history as an array (always include this field, even if empty)
    let history: Vec<Value> = user_rewards.points_history.iter()
        .map(|tx| {
            let mut map_entries = Vec::new();
            map_entries.push(("amount".to_string(), Value::Int(tx.amount)));
            map_entries.push(("reason".to_string(), Value::Text(tx.reason.clone())));
            map_entries.push(("timestamp".to_string(), Value::Nat(tx.timestamp)));
            if let Some(ref_id) = &tx.reference_id {
                map_entries.push(("reference_id".to_string(), Value::Text(ref_id.clone())));
            }
            map_entries.push(("points".to_string(), Value::Nat(tx.points)));
            Value::Map(map_entries)
        })
        .collect();
    response.insert("points_history".to_string(), Value::Array(history));
    
    // Add latest transaction if available
    if !user_rewards.points_history.is_empty() {
        let latest = &user_rewards.points_history[user_rewards.points_history.len() - 1];
        response.insert("latest_transaction_amount".to_string(), Value::Int(latest.amount));
        response.insert("latest_transaction_reason".to_string(), Value::Text(latest.reason.clone()));
        response.insert("latest_transaction_timestamp".to_string(), Value::Nat(latest.timestamp));
    }
    
    // Add last claim date if available
    if let Some(last_claim) = user_rewards.last_claim_date {
        response.insert("last_claim_date".to_string(), Value::Nat(last_claim));
    }
    
    // Add user principal
    response.insert("principal".to_string(), Value::Principal(principal));
    
    // Add last updated timestamp
    response.insert("last_updated".to_string(), Value::Nat(user_rewards.last_updated));
    
    Ok(response)
}

// Get available tasks
pub fn get_available_tasks(caller: Principal) -> SquareResult<Vec<TaskResponse>> {
    const MODULE: &str = "services::reward";
    const FUNCTION: &str = "get_available_tasks";
    
    // Get user tasks from main storage
    let user_tasks = STORAGE.with(|storage| {
        let store = storage.borrow();
        store.user_tasks.get(&caller).cloned()
    });
    
    let mut tasks = Vec::new();
    let now = time() / 1_000_000;
    
    // Get all tasks from main storage
    let task_definitions = STORAGE.with(|storage| {
        let store = storage.borrow();
        store.tasks.clone().unwrap_or_default()
    });
    
    // Iterate through all task definitions
    for (task_id, task_def) in task_definitions {
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
        let is_completed = if let Some(ut) = &user_tasks {
            if task_def.task_type == TaskType::Daily {
                // For daily tasks, check if completed today
                let today_start = now - (now % SECONDS_IN_DAY);
                ut.completed_tasks.get(&task_id)
                    .map(|completion_time| *completion_time >= today_start)
                    .unwrap_or(false)
            } else if task_def.task_type == TaskType::Weekly {
                // For weekly tasks, check if completed this week
                let week_start = now - (now % (SECONDS_IN_DAY * 7));
                ut.completed_tasks.get(&task_id)
                    .map(|completion_time| *completion_time >= week_start)
                    .unwrap_or(false)
            } else if task_def.task_type == TaskType::Monthly {
                // For monthly tasks, check if completed this month (approximate)
                let month_start = now - (now % (SECONDS_IN_DAY * 30));
                ut.completed_tasks.get(&task_id)
                    .map(|completion_time| *completion_time >= month_start)
                    .unwrap_or(false)
            } else {
                // For one-time tasks, check if ever completed
                ut.completed_tasks.contains_key(&task_id)
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
}

// Admin functions
pub fn award_points(request: AwardPointsRequest) -> SquareResult<()> {
    // Check if caller is admin or manager
    is_manager_or_admin()?;
    let now = time() / 1_000_000;
    let _points = STORAGE.with(|storage| {
        let mut store = storage.borrow_mut();
        let user_rewards = match store.user_rewards.get(&request.principal) {
            Some(rewards) => rewards.clone(),
            None => UserRewards {
                principal: request.principal,
                points: 0,
                points_history: Vec::new(),
                last_claim_date: None,
            // consecutive_daily_logins field has been moved to daily_checkin_task canister
            transactions: Vec::new(),
            last_updated: now,
        }
        };
        
        let mut updated_rewards = user_rewards.clone();
        
        // Add points
        updated_rewards.points += request.points;
        

        
        // Record transaction
        updated_rewards.points_history.push(PointsTransaction {
            amount: request.points as i64,
            reason: request.reason.clone(),
            timestamp: now,
            reference_id: request.reference_id.clone(),
            points: request.points,
        });
        
        updated_rewards.last_updated = now;
        
        store.user_rewards.insert(request.principal, updated_rewards.clone());
        
        updated_rewards.points
    });
    
    Ok(())
}

// Task management (admin functions)
pub fn create_task(request: CreateTaskRequest) -> SquareResult<String> {
    const MODULE: &str = "services::reward";
    const FUNCTION: &str = "create_task";
    
    let now = time() / 1_000_000;
    // Check if caller is admin or manager
    is_manager_or_admin().map_err(|e| {
        e
    })?;
    
    let task_id = if !request.id.is_empty() {
        request.id.clone()
    } else {
        format!("task_{}", now)
    };
    
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
        requirements: request.requirements,
        canister_id: request.canister_id,
    };
    
    // Store the task in main storage
    STORAGE.with(|storage| {
        let mut store = storage.borrow_mut();
        
        // Initialize tasks if it doesn't exist
        if store.tasks.is_none() {
            store.tasks = Some(HashMap::new());
        }
        
        if let Some(tasks) = &mut store.tasks {
            // Check if task ID already exists
            if tasks.contains_key(&task_id) {
                return log_and_return(already_exists_error(
                    "Task", 
                    &task_id, 
                    MODULE, 
                    FUNCTION
                ).with_details(format!("Task with ID {} already exists", task_id)));
            }
            
            // Add the task
            tasks.insert(task_id.clone(), task);
            Ok(task_id)
        } else {
            log_and_return(SquareError::SystemError("Failed to initialize tasks".to_string()))
        }
    })
}

pub fn update_task(request: UpdateTaskRequest) -> SquareResult<()> {
    const MODULE: &str = "services::reward";
    const FUNCTION: &str = "update_task";
    
    let now = time() / 1_000_000;
    // Check if caller is admin or manager
    is_manager_or_admin().map_err(|e| {
        e
    })?;
    
    // Update task in main storage
    STORAGE.with(|storage| {
        let mut store = storage.borrow_mut();
        
        if let Some(tasks) = &mut store.tasks {
            // Check if task exists
            let mut task = match tasks.get(&request.id) {
                Some(task) => task.clone(),
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
            task.updated_at = now;
            task.requirements = request.requirements;
            
            // Save updated task
            tasks.insert(request.id.clone(), task);
            
            Ok(())
        } else {
            log_and_return(SquareError::SystemError("Tasks storage not initialized".to_string()))
        }
    })
}

pub fn delete_task(task_id: String) -> SquareResult<()> {
    const MODULE: &str = "services::reward";
    const FUNCTION: &str = "delete_task";


    // Check if caller is admin or manager
    is_manager_or_admin().map_err(|e| {
        e
    })?;

    // Delete task from main storage
    STORAGE.with(|storage| {
        let mut store = storage.borrow_mut();
        
        if let Some(tasks) = &mut store.tasks {
            // Check if task exists
            if !tasks.contains_key(&task_id) {
                return log_and_return(not_found_error(
                    "Task",
                    &task_id,
                    MODULE,
                    FUNCTION
                ).with_details(format!("Task with ID {} not found", task_id)));
            }

            // Remove the task
            tasks.remove(&task_id);
            Ok(())
        } else {
            log_and_return(SquareError::SystemError("Tasks storage not initialized".to_string()))
        }
    })
}
