use std::collections::HashMap;
use ic_cdk::api::time;
use ic_cdk::api::stable::{stable_size, stable_write};
use crate::storage::STORAGE;
use super::types::*;
use super::utils::*;

// Save state to stable storage before canister upgrade
pub fn save_state_for_upgrade() {
    crate::utils::logger::log("========== SAVING STATE TO STABLE STORAGE ==========");
    
    let stable_size_before = stable_size();
    crate::utils::logger::log(&format!("Initial stable storage size: {} pages ({} bytes)", 
        stable_size_before, 
        stable_size_before * 65536));
        
    // Write magic bytes at the beginning of stable memory to identify version
    let mut magic_bytes = vec![0u8; 16];
    magic_bytes[0..8].copy_from_slice(&MAGIC_BYTES_V3);
    // Write the magic bytes to a specific location in stable memory
    match write_to_stable_memory(0, &magic_bytes) {
        Ok(_) => crate::utils::logger::log("✅ Wrote magic bytes to identify data format version"),
        Err(e) => crate::utils::logger::log(&format!("❌ Failed to write magic bytes: {}", e))
    };
    
    // Create a backup status to track progress
    let mut backup_status = BackupStatus {
        header_saved: false,
        main_storage_saved: false,
        timestamp: time() / 1_000_000,
    };
    
    crate::utils::logger::log("Checking data state before backup:");
    STORAGE.with(|storage| {
        let store = storage.borrow();
        crate::utils::logger::log(&format!("Main storage: {} users, {} posts, {} comments", 
            store.users.len(), store.posts.len(), store.comments.len()));
    });
    
    // Call storage preparation function
    synchronize_storage_before_upgrade();
    
    // Save main storage first
    crate::utils::logger::log("Saving main storage backup...");
    
    // Log current data counts before backup
    STORAGE.with(|storage| {
        let store = storage.borrow();
        crate::utils::logger::log("Current main storage data counts BEFORE backup:");
        crate::utils::logger::log(&format!("\n- Users: {}", store.users.len()));
        crate::utils::logger::log(&format!("\n- User profiles: {}", store.user_profiles.as_ref().map_or(0, |profiles| profiles.len())));
        crate::utils::logger::log(&format!("\n- User stats: {}", store.user_stats.as_ref().map_or(0, |stats| stats.len())));
        crate::utils::logger::log(&format!("\n- Posts: {}", store.posts.len()));
        crate::utils::logger::log(&format!("\n- Comments: {}", store.comments.len()));
        crate::utils::logger::log(&format!("\n- Likes: {}", store.likes.len()));
        crate::utils::logger::log(&format!("\n- User rewards: {}", store.user_rewards.len()));
        crate::utils::logger::log(&format!("\n- User tasks: {}", store.user_tasks.len()));
    });
    
    // Save main storage to stable memory
    let main_storage_result = STORAGE.with(|storage| {
        let store = storage.borrow();
        let cloned_store = (*store).clone();
        
        match candid::encode_one(cloned_store) {
            Ok(bytes) => {
                let checksum = calculate_checksum(&bytes);
                match write_to_stable_memory(MAIN_STORAGE_REGION_START, &bytes) {
                    Ok(_) => {
                        crate::utils::logger::log(&format!("Main storage saved successfully, size: {} bytes, checksum: {}", bytes.len(), checksum));
                        backup_status.main_storage_saved = true;
                        Ok((bytes.len() as u64, checksum))
                    },
                    Err(e) => {
                        crate::utils::logger::log(&format!("Failed to save main storage: {}", e));
                        Err(e)
                    }
                }
            },
            Err(e) => {
                crate::utils::logger::log(&format!("Failed to encode main storage: {:?}", e));
                Err(format!("Failed to encode main storage: {:?}", e))
            }
        }
    });
    
    // Create an empty placeholder for reserved storage area
    crate::utils::logger::log("Creating empty placeholder for reserved storage area");
    
    // Create an empty HashMap as placeholder
    let reserved_data = HashMap::<String, String>::new();
    
    // Serialize empty reserved data as placeholder
    let reserved_storage_result = match serde_json::to_vec(&reserved_data) {
        Ok(bytes) => {
            let checksum = calculate_checksum(&bytes);
            match write_to_stable_memory(RESERVED_STORAGE_REGION_START, &bytes) {
                Ok(_) => {
                    crate::utils::logger::log("Empty placeholder for reserved storage saved successfully");
                    Ok((bytes.len() as u64, checksum))
                },
                Err(e) => {
                    crate::utils::logger::log(&format!("Failed to save reserved storage placeholder: {}", e));
                    Err(e)
                }
            }
        },
        Err(e) => {
            crate::utils::logger::log(&format!("Failed to encode reserved storage placeholder: {:?}", e));
            Err(format!("Failed to encode reserved storage placeholder: {:?}", e))
        }
    };
    
    // Now save the header
    // Increment upgrade count
    let upgrade_count = UPGRADE_COUNT.with(|count| {
        let current = *count.borrow();
        *count.borrow_mut() = current + 1;
        current + 1
    });
    
    // Get canister ID if available
    let canister_id = match ic_cdk::api::id() {
        id => Some(id.to_string()),
    };
    
    // Create a storage header with enhanced metadata
    let header = StableStorageHeader {
        version: STABLE_STORAGE_VERSION,
        timestamp: time(),
        backup_flag: BACKUP_FLAG_MAGIC,
        main_storage_offset: MAIN_STORAGE_REGION_START,
        reserved_storage_offset: RESERVED_STORAGE_REGION_START,
        main_storage_size: main_storage_result.as_ref().map(|(size, _)| *size).unwrap_or(0),
        reserved_storage_size: reserved_storage_result.as_ref().map(|(size, _)| *size).unwrap_or(0),
        checksum: main_storage_result.as_ref().map(|(_, checksum)| *checksum).unwrap_or(0) ^ 
                 reserved_storage_result.as_ref().map(|(_, checksum)| *checksum).unwrap_or(0),
        main_storage_version: Some(MAIN_STORAGE_VERSION),
        reserved_storage_version: None,
        data_format: Some("candid".to_string()),
        canister_id,
        upgrade_count: Some(upgrade_count),
    };
    
    crate::utils::logger::log(&format!("Saving storage header with version {} and timestamp {}", 
        header.version, header.timestamp));
    
    // Serialize and save header
    match candid::encode_one(header.clone()) {
        Ok(bytes) => {
            match write_to_stable_memory(HEADER_REGION_START, &bytes) {
                Ok(_) => {
                    crate::utils::logger::log("✅ Saved storage header successfully");
                    backup_status.header_saved = true;
                },
                Err(e) => {
                    crate::utils::logger::log(&format!("❌ CRITICAL ERROR: Failed to save storage header: {}", e));
                }
            }
        },
        Err(e) => {
            crate::utils::logger::log(&format!("❌ CRITICAL ERROR: Failed to encode storage header: {:?}", e));
        }
    }
    
    // Also save a copy of the header using stable_save as a fallback
    match ic_cdk::storage::stable_save((header,)) {
        Ok(_) => crate::utils::logger::log("✅ Saved storage header using stable_save as fallback"),
        Err(e) => {
            crate::utils::logger::log(&format!("⚠️ Warning: Failed to save storage header using stable_save: {:?}", e));
        }
    }
    
    // Log backup status
    crate::utils::logger::log(&format!("Backup status: Header saved: {}, Main storage saved: {}",
        backup_status.header_saved, backup_status.main_storage_saved));
    
    let stable_size_after = stable_size();
    crate::utils::logger::log(&format!("Final stable storage size: {} pages ({} bytes)", 
        stable_size_after, 
        stable_size_after * 65536));
    
    crate::utils::logger::log("========== STATE SAVED SUCCESSFULLY ==========");
}

// Save main storage as a backup
pub fn save_main_storage_backup() {
    ic_cdk::println!("========== SAVING MAIN STORAGE BACKUP ==========");
    
    // Log current data counts before backup
    STORAGE.with(|storage| {
        let store = storage.borrow();
        ic_cdk::println!("Current main storage data counts BEFORE backup:");
        ic_cdk::println!("- Users: {}", store.users.len());
        ic_cdk::println!("- User profiles: {}", store.user_profiles.as_ref().map_or(0, |profiles| profiles.len()));
        ic_cdk::println!("- User stats: {}", store.user_stats.as_ref().map_or(0, |stats| stats.len()));
        ic_cdk::println!("- Posts: {}", store.posts.len());
        ic_cdk::println!("- Comments: {}", store.comments.len());
        ic_cdk::println!("- Likes: {}", store.likes.len());
        ic_cdk::println!("- User rewards: {}", store.user_rewards.len());
        ic_cdk::println!("- User tasks: {}", store.user_tasks.len());
    });
    
    // Use a special offset in stable memory for the backup flag
    let backup_flag_offset = 0;
    let backup_flag: u64 = 0x1234567890ABCDEF;
    
    // Write backup flag to indicate that a backup exists
    {
        ic_cdk::api::stable::stable_write(backup_flag_offset, &backup_flag.to_le_bytes());
    }
    
    // Serialize the main storage
    STORAGE.with(|storage| {
        let store = storage.borrow();
        let cloned_store = (*store).clone();
        
        // Serialize the storage to bytes
        match candid::encode_one(cloned_store) {
            Ok(bytes) => {
                // Write the size of the serialized data
                let size = bytes.len() as u64;
                let size_offset = 8; // After the backup flag
                {
                    ic_cdk::api::stable::stable_write(size_offset, &size.to_le_bytes());
                }
                
                // Write the serialized data
                let data_offset = 16; // After the backup flag and size
                {
                    ic_cdk::api::stable::stable_write(data_offset, &bytes);
                }
                
                ic_cdk::println!("Saved main storage backup successfully, size: {} bytes", size);
            },
            Err(e) => {
                let error_msg = format!("Failed to serialize main storage backup: {:?}", e);
                ic_cdk::println!("{}", error_msg);
            }
        }
    });
}

// Synchronize data between main storage before upgrade
pub fn synchronize_storage_before_upgrade() {
    ic_cdk::println!("========== PREPARING STORAGE BEFORE UPGRADE ==========");
    
    ic_cdk::println!("Storage state before upgrade:");

    STORAGE.with(|storage| {
        let store = storage.borrow();
        ic_cdk::println!("Main storage counts:");
        ic_cdk::println!("- Users: {}", store.users.len());
        ic_cdk::println!("- User profiles: {}", store.user_profiles.as_ref().map_or(0, |profiles| profiles.len()));
        ic_cdk::println!("- User stats: {}", store.user_stats.as_ref().map_or(0, |stats| stats.len()));
        ic_cdk::println!("- Posts: {}", store.posts.len());
        ic_cdk::println!("- Comments: {}", store.comments.len());
        ic_cdk::println!("- Likes: {}", store.likes.len());
        ic_cdk::println!("- User rewards: {}", store.user_rewards.len());
        ic_cdk::println!("- User tasks: {}", store.user_tasks.len());
    });
    
    // Using unified storage model
    ic_cdk::println!("Using unified storage model");
    
    // No synchronization needed with unified storage
    ic_cdk::println!("No synchronization needed with unified storage");
        
    // No more synchronization needed
    ic_cdk::println!("========== STORAGE PREPARATION COMPLETE ==========");
    
    
    ic_cdk::println!("========== STORAGE PREPARATION BEFORE UPGRADE COMPLETE ==========");
}
