use crate::storage::STORAGE;
use ic_cdk::api::time;
use super::types::*;

// Migration function for storage upgrades
pub fn migrate_all() -> bool {
    // No migration needed in current version
    ic_cdk::println!("Using unified storage, no migration needed");
    true
}

// User data migration function
pub fn migrate_users() {
    ic_cdk::println!("Using unified storage, no user migration needed");
}

// Content data migration function
pub fn migrate_content() {
    ic_cdk::println!("Using unified storage, no content migration needed");
}

// Interaction data migration function
pub fn migrate_interactions() {
    ic_cdk::println!("Using unified storage, no interactions migration needed");
}

// Reward data migration function
pub fn migrate_rewards() {
    ic_cdk::println!("Using unified storage, no rewards migration needed");
}

// Discovery data migration function
pub fn migrate_discovery() {
    ic_cdk::println!("Using unified storage, no discovery data migration needed");
}

// Migrate header from version 1 or 2 to version 3
pub fn migrate_header_to_v3(old_header: &[u8]) -> Result<StableStorageHeader, String> {
    
    if let Ok(header_v2) = candid::decode_one::<StableStorageHeader>(old_header) {

        let mut header_v3 = header_v2.clone();
        header_v3.version = STABLE_STORAGE_VERSION;
        
        header_v3.main_storage_version = Some(MAIN_STORAGE_VERSION);
        header_v3.reserved_storage_version = None;
        header_v3.data_format = Some("candid".to_string());
        
        if header_v3.canister_id.is_none() {
            header_v3.canister_id = match ic_cdk::api::id() {
                id => Some(id.to_string()),
            };
        }
        
        let upgrade_count = UPGRADE_COUNT.with(|count| {
            let current = *count.borrow();
            *count.borrow_mut() = current + 1;
            current + 1
        });
        header_v3.upgrade_count = Some(upgrade_count);
        
        return Ok(header_v3);
    }
    
    let current_time = time();
    crate::utils::logger::log(&format!("Creating new header with current timestamp: {}", current_time / 1_000_000));
    
    let header_v3 = StableStorageHeader {
        version: STABLE_STORAGE_VERSION,
        timestamp: current_time,
        backup_flag: BACKUP_FLAG_MAGIC,
        main_storage_offset: MAIN_STORAGE_REGION_START,
        reserved_storage_offset: RESERVED_STORAGE_REGION_START,
        main_storage_size: 0,
        reserved_storage_size: 0,
        checksum: 0,
        main_storage_version: Some(MAIN_STORAGE_VERSION),
        reserved_storage_version: None,
        data_format: Some("candid".to_string()),
        canister_id: match ic_cdk::api::id() {
            id => Some(id.to_string()),
        },
        upgrade_count: Some(1),
    };
    
    crate::utils::logger::log("Created a new header as migration failed");
    Ok(header_v3)
}
