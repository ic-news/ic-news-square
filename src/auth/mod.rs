use candid::Principal;
use ic_cdk::caller;
use crate::models::error::{SquareError, SquareResult};
use crate::storage::STORAGE;

pub fn init_admin() {
    STORAGE.with(|storage| {
        let mut store = storage.borrow_mut();
        store.admin = Some(caller());
    });
}

pub fn init_admin_if_empty() {
    STORAGE.with(|storage| {
        let mut store = storage.borrow_mut();
        if store.admin.is_none() {
            ic_cdk::println!("Initializing admin as caller");
            store.admin = Some(caller());
        }
    });
}

pub fn is_admin() -> Result<(), String> {
    let caller = caller();
    STORAGE.with(|storage| {
        let store = storage.borrow();
        if store.admin == Some(caller) {
            Ok(())
        } else {
            Err("Only admin can perform this action".to_string())
        }
    })
}

pub fn is_manager_or_admin() -> Result<(), String> {
    let caller = caller();
    
    STORAGE.with(|storage| {
        let store = storage.borrow();
        
        // Check if caller is admin
        let is_admin = store.admin == Some(caller);
        
        // Check if caller is manager
        let is_manager = store.managers.as_ref().map_or(false, |managers| managers.contains(&caller));
        
        if is_admin || is_manager {
            Ok(())
        } else {
            Err("Only managers or admin can perform this action".to_string())
        }
    })
}

#[ic_cdk_macros::update]
pub fn add_manager(manager: Principal) -> Result<(), String> {
    is_admin()?;
    STORAGE.with(|storage| {
        let mut store = storage.borrow_mut();
        // Initialize managers if it doesn't exist
        if store.managers.is_none() {
            store.managers = Some(std::collections::HashSet::new());
        }
        
        // Now we can safely insert the manager
        if let Some(managers) = &mut store.managers {
            managers.insert(manager);
        }
        Ok(())
    })
}

#[ic_cdk_macros::update]
pub fn remove_manager(manager: Principal) -> Result<(), String> {
    is_admin()?;
    STORAGE.with(|storage| {
        let mut store = storage.borrow_mut();
        // Remove manager if managers exists
        if let Some(managers) = &mut store.managers {
            managers.remove(&manager);
        }
        Ok(())
    })
}

#[ic_cdk_macros::query]
pub fn list_managers() -> Result<Vec<Principal>, String> {
    is_admin()?;
    Ok(STORAGE.with(|storage| {
        let store = storage.borrow();
        // Convert Option<HashSet<Principal>> to Vec<Principal>
        store.managers.as_ref().map_or(Vec::new(), |managers| {
            managers.iter().cloned().collect()
        })
    }))
}

// Get the authenticated caller or return an error
pub fn get_authenticated_caller() -> SquareResult<Principal> {
    let caller = caller();
    if caller == Principal::anonymous() {
        return Err(SquareError::Unauthorized("Anonymous principal not allowed".to_string()));
    }
    Ok(caller)
}

// Get the target principal if provided, otherwise return the authenticated caller
pub fn get_target_or_caller(principal: Option<Principal>) -> SquareResult<Principal> {
    match principal {
        Some(p) => Ok(p),
        None => get_authenticated_caller(),
    }
}

pub fn require_admin() -> SquareResult<()> {
    match is_admin() {
        Ok(_) => Ok(()),
        Err(e) => Err(SquareError::Unauthorized(e)),
    }
}