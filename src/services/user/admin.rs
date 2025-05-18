use candid::Principal;
use std::collections::{HashSet, HashMap};
use std::borrow::{Borrow, BorrowMut};

use crate::auth::{is_admin, is_manager_or_admin};
use crate::models::user::*;
use crate::models::error::{SquareError, SquareResult};
use crate::storage::{STORAGE, UserStatus as StorageUserStatus, UserRole as StorageUserRole};
use crate::utils::error_handler::*;
use super::utils::{map_storage_status_to_model, map_storage_role_to_model};

// User management (admin functions)
pub fn update_user_status(request: UserStatusUpdateRequest) -> SquareResult<()> {
    const MODULE: &str = "services::user::admin";
    const FUNCTION: &str = "update_user_status";
    
    // Check if caller is admin
    if is_admin().is_err() {
        return log_and_return(permission_denied_error(
            "Only admins can update user status", 
            MODULE, 
            FUNCTION,
            "update_user_status"
        ));
    }
    
    // Get user principal
    let principal = request.principal;

    if principal == Principal::anonymous() {
        return log_and_return(validation_error(
            "Invalid principal format", 
            MODULE, 
            FUNCTION
        ));
    };
    
    // Update user status in main storage
    STORAGE.with(|storage| {
        let mut store = storage.borrow_mut();
        
        if let Some(mut user) = store.users.get(&principal).cloned() {
            user.status = request.status;
            store.users.insert(principal, user);
        }
    });
    
    Ok(())
}

pub fn update_user_role(request: UserRoleUpdateRequest) -> SquareResult<()> {
    const MODULE: &str = "services::user::admin";
    const FUNCTION: &str = "update_user_role";
    
    // Check if caller is admin
    if is_admin().is_err() {
        return log_and_return(permission_denied_error(
            "Only admins can update user roles", 
            MODULE, 
            FUNCTION,
            "update_user_role"
        ));
    }
    
    // Get user principal
    let principal = request.principal;

    if principal == Principal::anonymous() {
        return log_and_return(validation_error(
            "Invalid principal format", 
            MODULE, 
            FUNCTION
        ));
    };
    
    // Update user role in main storage
    STORAGE.with(|storage| {
        let mut store = storage.borrow_mut();
        
        if let Some(mut user) = store.users.get(&principal).cloned() {
            user.role = request.role;
            store.users.insert(principal, user);
        }
    });
    
    Ok(())
}

// User verification functions
pub fn verify_user(principal: Principal) -> SquareResult<()> {
    const MODULE: &str = "services::user::admin";
    const FUNCTION: &str = "verify_user";
    
    // Check if caller is admin or manager
    if is_manager_or_admin().is_err() {
        return log_and_return(permission_denied_error(
            "Only managers or admins can verify users", 
            MODULE, 
            FUNCTION,
            "verify_user"
        ));
    }
    
    // Update user role to Creator in main storage
    STORAGE.with(|storage| {
        let mut store = storage.borrow_mut();
        
        if let Some(mut user) = store.users.get(&principal).cloned() {
            user.role = UserRole::Creator;
            store.users.insert(principal, user);
        }
    });
    
    Ok(())
}

// Debug function to list all users
pub fn debug_list_all_users() -> SquareResult<Vec<(String, String)>> {
    const MODULE: &str = "services::user::admin";
    const FUNCTION: &str = "debug_list_all_users";
    
    // Check if caller is admin
    if is_admin().is_err() {
        return log_and_return(permission_denied_error(
            "Only admins can list all users", 
            MODULE, 
            FUNCTION,
            "list_all_users"
        ));
    }
    
    let mut users = Vec::new();
    
    // Get all users from main storage
    STORAGE.with(|storage| {
        let store = storage.borrow();
        
        for (principal, _) in &store.users {
            let principal_str = principal.to_string();
            
            // Get username from profile
            let username = if let Some(profiles) = &store.user_profiles {
                if let Some(profile) = profiles.get(principal) {
                    profile.username.clone()
                } else {
                    "<no profile>".to_string()
                }
            } else {
                "<no profile>".to_string()
            };
            
            users.push((principal_str, username));
        }
    });
    
    // Sort users by username
    users.sort_by(|a, b| a.1.cmp(&b.1));
    
    Ok(users)
}