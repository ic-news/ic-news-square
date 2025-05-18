use std::cell::RefCell;
use std::collections::HashMap;
use candid::{CandidType, Deserialize, Principal};
use serde::{Serialize, Deserializer, Serializer};

// Stable storage version for compatibility checks
pub const STABLE_STORAGE_VERSION: u32 = 4; // Increment when data structure changes
pub const PREVIOUS_STABLE_STORAGE_VERSION: u32 = 3; // Previous version for migration
pub const MAIN_STORAGE_VERSION: u32 = 2; // Version of main storage format

// Track upgrade count in a static variable
thread_local! {
    pub static UPGRADE_COUNT: std::cell::RefCell<u32> = std::cell::RefCell::new(0);
}

// Memory regions for different backup types
pub const HEADER_REGION_START: u64 = 0;
pub const MAIN_STORAGE_REGION_START: u64 = 4096; // 4KB offset
// Reserved memory region for future use
pub const RESERVED_STORAGE_REGION_START: u64 = 1048576; // 1MB offset
pub const BACKUP_FLAG_MAGIC: u64 = 0x1234567890ABCDEF;

// Magic bytes to identify data format in stable memory
pub const MAGIC_BYTES_V1: [u8; 8] = [0x49, 0x43, 0x4E, 0x45, 0x57, 0x53, 0x56, 0x31]; // "ICNEWSV1"
pub const MAGIC_BYTES_V2: [u8; 8] = [0x49, 0x43, 0x4E, 0x45, 0x57, 0x53, 0x56, 0x32]; // "ICNEWSV2"
pub const MAGIC_BYTES_V3: [u8; 8] = [0x49, 0x43, 0x4E, 0x45, 0x57, 0x53, 0x56, 0x33]; // "ICNEWSV3"

// Backup types
pub enum BackupType {
    Header,
    MainStorage,
    All
}

// Backup status for logging and debugging
#[derive(Debug)]
pub struct BackupStatus {
    pub header_saved: bool,
    pub main_storage_saved: bool,
    pub timestamp: u64,
}

// Restoration status for logging and debugging
#[derive(Debug)]
pub struct RestorationStatus {
    pub header_restored: bool,
    pub main_storage_restored: bool,
    pub timestamp: u64,
}

#[derive(CandidType, Deserialize, Debug, Clone)]
pub struct StableStorageHeader {
    pub version: u32,
    pub timestamp: u64,
    pub backup_flag: u64,
    pub main_storage_offset: u64,
    // Reserved field for backwards compatibility
    pub reserved_storage_offset: u64,
    pub main_storage_size: u64,
    // Reserved field for backwards compatibility
    pub reserved_storage_size: u64,
    pub checksum: u64,
    // Additional metadata for better diagnostics - all optional fields for backwards compatibility
    #[serde(default)]
    pub main_storage_version: Option<u32>,
    // Reserved field for backwards compatibility
    #[serde(default)]
    pub reserved_storage_version: Option<u32>,
    #[serde(default)]
    pub data_format: Option<String>, // "candid" or "json"
    #[serde(default)]
    pub canister_id: Option<String>,
    #[serde(default)]
    pub upgrade_count: Option<u32>,
}
