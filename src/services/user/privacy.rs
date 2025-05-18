use candid::Principal;
use std::borrow::{Borrow, BorrowMut};

use crate::models::user::*;
use crate::models::error::{SquareError, SquareResult};
use crate::storage::{STORAGE};
use crate::utils::error_handler::*;

// Privacy settings functions
pub fn update_privacy_settings(principal: Principal, privacy_settings: UserPrivacySettings) -> SquareResult<()> {
    const MODULE: &str = "services::user::privacy";
    const FUNCTION: &str = "update_privacy_settings";
    
    // Update privacy settings in main storage
    STORAGE.with(|storage| -> SquareResult<()> {
        let mut store = storage.borrow_mut();
        
        if let Some(profiles) = &mut store.user_profiles {
            if let Some(profile) = profiles.get_mut(&principal) {
                profile.privacy_settings = Some(privacy_settings);
                Ok(())
            } else {
                log_and_return(not_found_error(
                    "UserProfile", 
                    &principal.to_string(), 
                    MODULE, 
                    FUNCTION
                ).with_details("User profile not found"))
            }
        } else {
            log_and_return(not_found_error(
                "UserProfiles", 
                "storage", 
                MODULE, 
                FUNCTION
            ).with_details("User profiles not initialized"))
        }
    })
}

pub fn get_privacy_settings(principal: Principal) -> SquareResult<UserPrivacySettings> {
    const MODULE: &str = "services::user::privacy";
    const FUNCTION: &str = "get_privacy_settings";
    
    // Get privacy settings from main storage
    let settings_result = STORAGE.with(|storage| {
        let store = storage.borrow();
        
        if let Some(profiles) = &store.user_profiles {
            if let Some(profile) = profiles.get(&principal) {
                profile.privacy_settings.clone()
            } else {
                None
            }
        } else {
            None
        }
    });
    
    // Return settings or default
    match settings_result {
        Some(settings) => Ok(settings),
        None => {
            // Create default privacy settings
            let default_settings = UserPrivacySettings {
                profile_visibility: crate::models::user::ProfileVisibility::Public,
                content_visibility: crate::models::user::ContentVisibility::Public,
                interaction_preferences: crate::models::user::InteractionPreferences::default(),
                notification_preferences: crate::models::user::NotificationPreferences::default(),
            };
            
            // Store default settings
            update_privacy_settings(principal, default_settings.clone())?;
            
            Ok(default_settings)
        }
    }
}