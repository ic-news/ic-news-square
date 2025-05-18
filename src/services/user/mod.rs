// User service module
// This module contains all user-related functionality

// Re-export all submodules
pub mod profile;
pub mod social;
pub mod admin;
pub mod notification;
pub mod privacy;
pub mod sync;
pub mod utils;

// Re-export commonly used functions and types for convenience
pub use profile::{register_user, update_user_profile, get_user_profile, get_user_full_profile, debug_fix_user_profile};
pub use social::{follow_user, unfollow_user, follow_topic, unfollow_topic, get_followers, get_following, get_user_social_info};
pub use admin::{update_user_status, update_user_role, verify_user, debug_list_all_users};
pub use notification::{create_notification, get_user_notifications, mark_notification_as_read, mark_all_notifications_as_read};
pub use privacy::{update_privacy_settings, get_privacy_settings};
pub use sync::{sync_user_data, synchronize_all_user_data, debug_fix_user_data};
pub use utils::{find_user_by_handle, get_user_leaderboard};