use ic_cdk::api::stable::{stable_size, stable_write, stable_read, stable_grow};
use super::types::*;

// Detect the version of stored data by examining magic bytes or other indicators
pub fn detect_data_version() -> u32 {
    // Check for magic bytes at the beginning of stable memory
    let mut magic_buffer = [0u8; 8];
    stable_read(0, &mut magic_buffer);
    
    if magic_buffer == MAGIC_BYTES_V3 {
        return 3;
    } else if magic_buffer == MAGIC_BYTES_V2 {
        return 2;
    } else if magic_buffer == MAGIC_BYTES_V1 {
        return 1;
    }
    
    // If no magic bytes found, try to read a header and check its version
    match read_from_stable_memory(HEADER_REGION_START, 4096) {
        Ok(bytes) => {
            match candid::decode_one::<StableStorageHeader>(&bytes) {
                Ok(header) => header.version,
                Err(_) => 0 // Unknown version
            }
        },
        Err(_) => 0 // Unknown version
    }
}

// Ensure we have enough stable memory for our backup
pub fn ensure_stable_memory(required_pages: u64) -> bool {
    let current_pages = stable_size();
    if current_pages < required_pages {
        match stable_grow(required_pages - current_pages) {
            Ok(_) => true,
            Err(_) => false
        }
    } else {
        true
    }
}

// Write data to a specific region in stable memory
pub fn write_to_stable_memory(offset: u64, data: &[u8]) -> Result<(), String> {
    // Make sure we have enough memory
    let pages_needed = (offset + data.len() as u64 + 65535) / 65536;
    if !ensure_stable_memory(pages_needed) {
        return Err(format!("Failed to grow stable memory to {} pages", pages_needed));
    }
    
    // Write the data
    stable_write(offset, data);
    
    Ok(())
}

// Read data from a specific region in stable memory
pub fn read_from_stable_memory(offset: u64, size: usize) -> Result<Vec<u8>, String> {
    let mut buffer = vec![0u8; size];
    stable_read(offset, &mut buffer);
    Ok(buffer)
}

// Calculate a simple checksum for data integrity verification
pub fn calculate_checksum(data: &[u8]) -> u64 {
    let mut checksum: u64 = 0;
    for (i, &byte) in data.iter().enumerate() {
        checksum = checksum.wrapping_add((byte as u64).wrapping_mul(i as u64 + 1));
    }
    checksum
}
