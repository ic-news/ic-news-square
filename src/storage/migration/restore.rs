use std::collections::HashMap;
use ic_cdk::api::time;
use ic_cdk::storage::stable_restore;
use ic_cdk::api::stable::{stable_read, stable_size};
use crate::storage::STORAGE;
use super::types::*;
use super::utils::*;
use super::migrate::migrate_header_to_v3;

// Restore state from stable storage after canister upgrade
pub fn restore_state_after_upgrade() -> bool {
    crate::utils::logger::log("\n========== RESTORING STATE FROM STABLE STORAGE ==========\n");
    
    let stable_size = stable_size();
    crate::utils::logger::log(&format!("Current stable storage size: {} pages ({} bytes)\n", 
        stable_size, 
        stable_size * 65536));
    
    let mut restoration_status = RestorationStatus {
        header_restored: false,
        main_storage_restored: false,
        timestamp: time() / 1_000_000,
    };
    
    crate::utils::logger::log("\n========== DATA STATE BEFORE RESTORATION ==========\n");
    
    STORAGE.with(|storage| {
        let store = storage.borrow();
        crate::utils::logger::log(&format!("Main storage data counts:\n"));
        crate::utils::logger::log(&format!("\n- Users: {}", store.users.len()));
        crate::utils::logger::log(&format!("\n- User profiles: {}", store.user_profiles.as_ref().map_or(0, |profiles| profiles.len())));
        crate::utils::logger::log(&format!("\n- User stats: {}", store.user_stats.as_ref().map_or(0, |stats| stats.len())));
        crate::utils::logger::log(&format!("\n- User rewards: {}", store.user_rewards.len()));
        crate::utils::logger::log(&format!("\n- User tasks: {}", store.user_tasks.len()));
        crate::utils::logger::log(&format!("\n- Tasks: {}", store.tasks.as_ref().map_or(0, |t| t.len())));
        crate::utils::logger::log(&format!("\n- Posts: {}", store.posts.len()));
        crate::utils::logger::log(&format!("\n- Comments: {}", store.comments.len()));
        crate::utils::logger::log(&format!("\n- Likes: {}", store.likes.len()));
        crate::utils::logger::log(&format!("\n- User posts: {}", store.user_posts.len()));
        crate::utils::logger::log(&format!("\n- User comments: {}", store.user_comments.len()));
        crate::utils::logger::log(&format!("\n- Trending topics: {}", store.trending_topics.len()));
        crate::utils::logger::log(&format!("\n- Trending content: {}", store.trending_content.len()));
    });
    
    crate::utils::logger::log("\nUsing unified storage model\n");
    
    crate::utils::logger::log("==================================================\n");
    
    let detected_version = detect_data_version();
    crate::utils::logger::log(&format!("\nDetected data version: {}\n", detected_version));
    
    crate::utils::logger::log("\n========== RESTORING STORAGE HEADER ==========\n");
    crate::utils::logger::log("Attempting to restore storage header...");
    
    let mut header_opt: Option<StableStorageHeader> = None;
    
    if let Ok(bytes) = read_from_stable_memory(HEADER_REGION_START, 4096) {
        if let Ok(header) = candid::decode_one::<StableStorageHeader>(&bytes) {
            crate::utils::logger::log(&format!("✅ Restored storage header from standard position, version: {}", header.version));
            restoration_status.header_restored = true;
            header_opt = Some(header);
        } else {
            crate::utils::logger::log("Initial decode attempt failed, trying to find valid data length");
            
            let mut valid_data = bytes.clone();
            for len in (100..bytes.len()).rev() {
                valid_data.truncate(len);
                if let Ok(header) = candid::decode_one::<StableStorageHeader>(&valid_data) {
                    crate::utils::logger::log(&format!("✅ Restored header with truncated data (length: {}), version: {}", len, header.version));
                    restoration_status.header_restored = true;
                    header_opt = Some(header);
                    break;
                }
            }
        }
    } else {
        crate::utils::logger::log("⚠️ Failed to read header from standard position");
    }
    
    if header_opt.is_none() {
        crate::utils::logger::log("\nAttempting to restore header by skipping magic bytes...");
        if let Ok(bytes) = read_from_stable_memory(HEADER_REGION_START + 16, 4096) {
            if let Ok(header) = candid::decode_one::<StableStorageHeader>(&bytes) {
                crate::utils::logger::log(&format!("✅ Restored header by skipping magic bytes, version: {}\n", header.version));
                restoration_status.header_restored = true;
                header_opt = Some(header);
            } else {
                crate::utils::logger::log("⚠️ Failed to decode header with magic bytes skipped\n");
            }
        } else {
            crate::utils::logger::log("⚠️ Failed to read header with magic bytes skipped\n");
        }
    }
    
    let header_result = if let Some(header) = header_opt {
        Ok(header)
    } else {
        Err("Failed to restore header from any position".to_string())
    };
    
    let header_result = if header_result.is_err() {
        crate::utils::logger::log("\nAttempting to create a new header through migration...\n");
        match read_from_stable_memory(HEADER_REGION_START, 8192) {
            Ok(bytes) => {
                match migrate_header_to_v3(&bytes) {
                    Ok(migrated_header) => {
                        crate::utils::logger::log("✅ Successfully created a new header through migration\n");
                        restoration_status.header_restored = true;
                        Ok(migrated_header)
                    },
                    Err(migration_err) => {
                        crate::utils::logger::log(&format!("⚠️ Failed to create header through migration: {}\n", migration_err));
                        Err(migration_err)
                    }
                }
            },
            Err(e) => {
                crate::utils::logger::log(&format!("⚠️ Failed to read data for header migration: {}\n", e));
                Err(e)
            }
        }
    } else {
        header_result
    };
    
    let header = if header_result.is_err() {
        crate::utils::logger::log("\nAttempting to restore storage header from fallback format...\n");
        match stable_restore::<(StableStorageHeader,)>() {
            Ok((header,)) => {
                crate::utils::logger::log(&format!("✅ Restored storage header from fallback format, version: {}, timestamp: {}\n", 
                    header.version, header.timestamp));
                restoration_status.header_restored = true;
                Ok(header)
            },
            Err(e) => {
                crate::utils::logger::log(&format!("❌ Failed to restore storage header from fallback format: {:?}\n", e));
                Err(format!("Failed to restore header: {:?}", e))
            }
        }
    } else {
        header_result
    };
    
    let header = match header {
        Ok(header) => {
            crate::utils::logger::log("\n========== CHECKING HEADER COMPATIBILITY ==========\n");
            if header.version != STABLE_STORAGE_VERSION && header.version != PREVIOUS_STABLE_STORAGE_VERSION {
                let msg = format!("Stable storage version mismatch: expected {} or {}, found {}", 
                    STABLE_STORAGE_VERSION, PREVIOUS_STABLE_STORAGE_VERSION, header.version);
                crate::utils::logger::log(&format!("⚠️ WARNING: {}\n", msg));
            } else {
                crate::utils::logger::log(&format!("\u{2705} Header version compatible: {}\n", header.version));
            }
            
            crate::utils::logger::log("\n========== CHECKING TIMESTAMP ==========\n");
            let current_time = time() / 1_000_000;
            let header_time = header.timestamp / 1_000_000; 
            
            if current_time >= header_time {
                let time_diff_seconds = (current_time - header_time) as i64;
                let time_diff_minutes = time_diff_seconds / 60;
                let time_diff_hours = time_diff_minutes / 60;
                
                crate::utils::logger::log(&format!("Data was saved {} hours {} minutes ago ({})\n", 
                    time_diff_hours, time_diff_minutes % 60, header_time));
            } else {
                crate::utils::logger::log(&format!("Warning: Header timestamp ({}) is in the future compared to current time ({})\n", 
                    header_time, current_time));
            }
            
            Some(header)
        },
        Err(_) => None
    };
    
    crate::utils::logger::log("Attempting to restore main storage...");
    
    let main_storage_restored = if let Some(header) = &header {
        if header.main_storage_size > 0 {
            match read_from_stable_memory(header.main_storage_offset, header.main_storage_size as usize) {
                Ok(bytes) => {
                    let _actual_checksum = calculate_checksum(&bytes);
                    
                    match candid::decode_one::<crate::storage::Storage>(&bytes) {
                        Ok(storage) => {
                            STORAGE.with(|s| {
                                *s.borrow_mut() = storage;
                            });
                            
                            STORAGE.with(|storage| {
                                let store = storage.borrow();
                                crate::utils::logger::log("Main storage data counts AFTER restoration:");
                                crate::utils::logger::log(&format!("\n- Users: {}", store.users.len()));
                                crate::utils::logger::log(&format!("\n- User profiles: {}", store.user_profiles.as_ref().map_or(0, |profiles| profiles.len())));
                                crate::utils::logger::log(&format!("\n- User stats: {}", store.user_stats.as_ref().map_or(0, |stats| stats.len())));
                                crate::utils::logger::log(&format!("\n- Posts: {}", store.posts.len()));
                                crate::utils::logger::log(&format!("\n- Comments: {}", store.comments.len()));
                                crate::utils::logger::log(&format!("\n- Likes: {}", store.likes.len()));
                                crate::utils::logger::log(&format!("\n- User rewards: {}", store.user_rewards.len()));
                                crate::utils::logger::log(&format!("\n- User tasks: {}", store.user_tasks.len()));
                            });
                            
                            crate::utils::logger::log(&format!("✅ Successfully restored main storage, size: {} bytes", bytes.len()));
                            restoration_status.main_storage_restored = true;
                            true
                        },
                        Err(e) => {
                            crate::utils::logger::log(&format!("❌ Failed to decode main storage: {:?}", e));
                            false
                        }
                    }
                },
                Err(e) => {
                    crate::utils::logger::log(&format!("❌ Failed to read main storage: {}", e));
                    false
                }
            }
        } else {
            crate::utils::logger::log("⚠️ Main storage size is 0, skipping restoration");
            false
        }
    } else {
        crate::utils::logger::log("⚠️ No valid header found, skipping main storage restoration");
        false
    };
    
    if !main_storage_restored {
        crate::utils::logger::log("Attempting to restore main storage using legacy method...");
        if restore_from_main_storage_backup() {
            crate::utils::logger::log("✅ Successfully restored main storage using legacy method");
            restoration_status.main_storage_restored = true;
        } else {
            crate::utils::logger::log("❌ Failed to restore main storage using legacy method");
        }
    }
    
    crate::utils::logger::log("Using unified storage model");
    
    crate::utils::logger::log("Data state after restoration:");
    STORAGE.with(|storage| {
        let store = storage.borrow();
        crate::utils::logger::log(&format!("Main storage: {} users, {} posts, {} comments", 
            store.users.len(), store.posts.len(), store.comments.len()));
    });
    
    crate::utils::logger::log(&format!("Restoration status: Header restored: {}, Main storage restored: {}",
        restoration_status.header_restored, restoration_status.main_storage_restored));
    
    crate::utils::logger::log("\n========== DATA STATE AFTER RESTORATION ==========");
    
    STORAGE.with(|storage| {
        let store = storage.borrow();
        crate::utils::logger::log(&format!("Main storage data counts after restoration:"));
        crate::utils::logger::log(&format!("\n- Users: {}", store.users.len()));
        crate::utils::logger::log(&format!("\n- User profiles: {}", store.user_profiles.as_ref().map_or(0, |profiles| profiles.len())));
        crate::utils::logger::log(&format!("\n- User stats: {}", store.user_stats.as_ref().map_or(0, |stats| stats.len())));
        crate::utils::logger::log(&format!("\n- User rewards: {}", store.user_rewards.len()));
        crate::utils::logger::log(&format!("\n- User tasks: {}", store.user_tasks.len()));
        crate::utils::logger::log(&format!("\n- Tasks: {}", store.tasks.as_ref().map_or(0, |t| t.len())));
        crate::utils::logger::log(&format!("\n- Posts: {}", store.posts.len()));
        crate::utils::logger::log(&format!("\n- Comments: {}", store.comments.len()));
        crate::utils::logger::log(&format!("\n- Likes: {}", store.likes.len()));
        crate::utils::logger::log(&format!("\n- User posts: {}", store.user_posts.len()));
        crate::utils::logger::log(&format!("\n- User comments: {}", store.user_comments.len()));
        crate::utils::logger::log(&format!("\n- Trending topics: {}", store.trending_topics.len()));
        crate::utils::logger::log(&format!("\n- Trending content: {}", store.trending_content.len()));
    });
    
    crate::utils::logger::log("\nUsing unified storage model");
    
    crate::utils::logger::log("\n========== STATE RESTORATION COMPLETE ==========\n");
    
    restoration_status.main_storage_restored
}

// Restore from main storage backup
pub fn restore_from_main_storage_backup() -> bool {
    ic_cdk::println!("Restoring from main storage backup...");
    
    // Check for backup flag
    let backup_flag_offset = 0;
    let mut backup_flag_bytes = [0u8; 8];
    {
        stable_read(backup_flag_offset, &mut backup_flag_bytes);
    }
    
    let backup_flag = u64::from_le_bytes(backup_flag_bytes);
    if backup_flag != 0x1234567890ABCDEF {
        let error_msg = format!("No valid backup flag found: {:X}", backup_flag);
        ic_cdk::println!("{}", error_msg);
        return false;
    }
    
    // Read the size of the serialized data
    let size_offset = 8; // After the backup flag
    let mut size_bytes = [0u8; 8];
    {
        stable_read(size_offset, &mut size_bytes);
    }
    
    let size = u64::from_le_bytes(size_bytes) as usize;
    if size == 0 {
        let error_msg = "Backup size is 0, no valid backup data";
        ic_cdk::println!("{}", error_msg);
        return false;
    }
    
    // Read the serialized data
    let data_offset = 16; // After the backup flag and size
    let mut bytes = vec![0u8; size];
    {
        stable_read(data_offset, &mut bytes);
    }
    
    // Deserialize the data
    match candid::decode_one(&bytes) {
        Ok(backup_storage) => {
            STORAGE.with(|storage| {
                *storage.borrow_mut() = backup_storage;
            });
            
            // Log data counts after restoration
            STORAGE.with(|storage| {
                let store = storage.borrow();
                ic_cdk::println!("Main storage data counts AFTER restoration:");
                ic_cdk::println!("- Users: {}", store.users.len());
                ic_cdk::println!("- User profiles: {}", store.user_profiles.as_ref().map_or(0, |profiles| profiles.len()));
                ic_cdk::println!("- User stats: {}", store.user_stats.as_ref().map_or(0, |stats| stats.len()));
                ic_cdk::println!("- Posts: {}", store.posts.len());
                ic_cdk::println!("- Comments: {}", store.comments.len());
                ic_cdk::println!("- Likes: {}", store.likes.len());
                ic_cdk::println!("- User rewards: {}", store.user_rewards.len());
                ic_cdk::println!("- User tasks: {}", store.user_tasks.len());
            });
            
            ic_cdk::println!("Restored from main storage backup successfully, size: {} bytes", size);
            true
        },
        Err(e) => {
            let error_msg = format!("Failed to deserialize main storage backup: {:?}", e);
            ic_cdk::println!("{}", error_msg);
            false
        }
    }
}

// Check storage state after upgrade
pub fn synchronize_storage_after_upgrade() {
    ic_cdk::println!("========== CHECKING STORAGE AFTER UPGRADE ==========");
    
    ic_cdk::println!("Storage state after upgrade:");
    
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
    
    ic_cdk::println!("Using unified storage model");
    
    ic_cdk::println!("========== STORAGE CHECK AFTER UPGRADE COMPLETE ==========");
}
