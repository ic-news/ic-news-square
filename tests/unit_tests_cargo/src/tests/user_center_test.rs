use crate::*;
use std::time::{SystemTime, UNIX_EPOCH};

// Mock User structures
#[derive(CandidType, Deserialize, Clone, Debug)]
struct UserProfile {
    principal: Principal,
    username: String,
    display_name: Option<String>,
    bio: Option<String>,
    avatar_url: Option<String>,
    created_at: u64,
    updated_at: Option<u64>,
    total_posts: u64,
    total_comments: u64,
    total_points: u64,
    level: u64,
    badges: Vec<String>,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
struct UpdateProfileRequest {
    display_name: Option<String>,
    bio: Option<String>,
    avatar_url: Option<String>,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
struct UserResponse {
    success: bool,
    message: String,
    user: Option<UserProfile>,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
struct UsersResponse {
    success: bool,
    message: String,
    users: Vec<UserProfile>,
    total: usize,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
struct Badge {
    id: String,
    name: String,
    description: String,
    image_url: Option<String>,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
struct BadgeResponse {
    success: bool,
    message: String,
    badge: Option<Badge>,
}

// Mock storage for users
thread_local! {
    static MOCK_USER_STORAGE: RefCell<MockUserStorage> = RefCell::new(MockUserStorage::default());
}

#[derive(Default, Clone)]
struct MockUserStorage {
    users: HashMap<Principal, UserProfile>,
    usernames: HashMap<String, Principal>,
    badges: HashMap<String, Badge>,
}

// Mock user functions
fn mock_register_user(principal: Principal, username: String) -> UserResponse {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    // Check if username is valid
    if username.len() < 3 || username.len() > 20 {
        return UserResponse {
            success: false,
            message: "Username must be between 3 and 20 characters".to_string(),
            user: None,
        };
    }
    
    // Check if username contains only allowed characters
    if !username.chars().all(|c| c.is_alphanumeric() || c == '_') {
        return UserResponse {
            success: false,
            message: "Username can only contain alphanumeric characters and underscores".to_string(),
            user: None,
        };
    }
    
    let result = MOCK_USER_STORAGE.with(|storage| {
        let mut storage = storage.borrow_mut();
        
        // Check if user already exists
        if storage.users.contains_key(&principal) {
            return UserResponse {
                success: false,
                message: "User already registered".to_string(),
                user: None,
            };
        }
        
        // Check if username is taken
        if storage.usernames.contains_key(&username) {
            return UserResponse {
                success: false,
                message: "Username already taken".to_string(),
                user: None,
            };
        }
        
        // Create user profile
        let user = UserProfile {
            principal,
            username: username.clone(),
            display_name: None,
            bio: None,
            avatar_url: None,
            created_at: now,
            updated_at: None,
            total_posts: 0,
            total_comments: 0,
            total_points: 0,
            level: 1,
            badges: Vec::new(),
        };
        
        // Store user
        storage.users.insert(principal, user.clone());
        storage.usernames.insert(username, principal);
        
        UserResponse {
            success: true,
            message: "User registered successfully".to_string(),
            user: Some(user),
        }
    });
    
    result
}

fn mock_get_user(principal: Principal) -> UserResponse {
    let user = MOCK_USER_STORAGE.with(|storage| {
        let storage = storage.borrow();
        storage.users.get(&principal).cloned()
    });
    
    match user {
        Some(user) => UserResponse {
            success: true,
            message: "User retrieved successfully".to_string(),
            user: Some(user),
        },
        None => UserResponse {
            success: false,
            message: "User not found".to_string(),
            user: None,
        },
    }
}

fn mock_get_user_by_username(username: String) -> UserResponse {
    let result = MOCK_USER_STORAGE.with(|storage| {
        let storage = storage.borrow();
        
        if let Some(principal) = storage.usernames.get(&username) {
            if let Some(user) = storage.users.get(principal) {
                return UserResponse {
                    success: true,
                    message: "User retrieved successfully".to_string(),
                    user: Some(user.clone()),
                };
            }
        }
        
        UserResponse {
            success: false,
            message: "User not found".to_string(),
            user: None,
        }
    });
    
    result
}

fn mock_update_profile(principal: Principal, request: UpdateProfileRequest) -> UserResponse {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    let result = MOCK_USER_STORAGE.with(|storage| {
        let mut storage = storage.borrow_mut();
        
        if let Some(user) = storage.users.get_mut(&principal) {
            // Update profile fields
            if let Some(display_name) = request.display_name {
                user.display_name = Some(display_name);
            }
            
            if let Some(bio) = request.bio {
                user.bio = Some(bio);
            }
            
            if let Some(avatar_url) = request.avatar_url {
                user.avatar_url = Some(avatar_url);
            }
            
            user.updated_at = Some(now);
            
            UserResponse {
                success: true,
                message: "Profile updated successfully".to_string(),
                user: Some(user.clone()),
            }
        } else {
            UserResponse {
                success: false,
                message: "User not found".to_string(),
                user: None,
            }
        }
    });
    
    result
}

fn mock_award_badge(principal: Principal, badge_id: String) -> UserResponse {
    let result = MOCK_USER_STORAGE.with(|storage| {
        let mut storage = storage.borrow_mut();
        
        // Check if badge exists
        if !storage.badges.contains_key(&badge_id) {
            return UserResponse {
                success: false,
                message: "Badge not found".to_string(),
                user: None,
            };
        }
        
        if let Some(user) = storage.users.get_mut(&principal) {
            // Check if user already has the badge
            if user.badges.contains(&badge_id) {
                return UserResponse {
                    success: false,
                    message: "User already has this badge".to_string(),
                    user: Some(user.clone()),
                };
            }
            
            // Award badge
            user.badges.push(badge_id);
            
            UserResponse {
                success: true,
                message: "Badge awarded successfully".to_string(),
                user: Some(user.clone()),
            }
        } else {
            UserResponse {
                success: false,
                message: "User not found".to_string(),
                user: None,
            }
        }
    });
    
    result
}

fn mock_create_badge(id: String, name: String, description: String, image_url: Option<String>) -> BadgeResponse {
    let result = MOCK_USER_STORAGE.with(|storage| {
        let mut storage = storage.borrow_mut();
        
        // Check if badge already exists
        if storage.badges.contains_key(&id) {
            return BadgeResponse {
                success: false,
                message: "Badge already exists".to_string(),
                badge: None,
            };
        }
        
        // Create badge
        let badge = Badge {
            id: id.clone(),
            name,
            description,
            image_url,
        };
        
        // Store badge
        storage.badges.insert(id, badge.clone());
        
        BadgeResponse {
            success: true,
            message: "Badge created successfully".to_string(),
            badge: Some(badge),
        }
    });
    
    result
}

fn mock_get_top_users(limit: usize) -> UsersResponse {
    let users = MOCK_USER_STORAGE.with(|storage| {
        let storage = storage.borrow();
        
        // Get all users
        let mut users: Vec<UserProfile> = storage.users.values().cloned().collect();
        
        // Sort by total points (descending)
        users.sort_by(|a, b| b.total_points.cmp(&a.total_points));
        
        // Limit the number of users
        users.truncate(limit);
        
        users
    });
    
    let total = users.len();
    UsersResponse {
        success: true,
        message: "Top users retrieved successfully".to_string(),
        users,
        total,
    }
}

fn mock_update_user_stats(principal: Principal, posts_delta: i64, comments_delta: i64, points_delta: i64) -> UserResponse {
    let result = MOCK_USER_STORAGE.with(|storage| {
        let mut storage = storage.borrow_mut();
        
        if let Some(user) = storage.users.get_mut(&principal) {
            // Update stats
            if posts_delta > 0 {
                user.total_posts += posts_delta as u64;
            } else if posts_delta < 0 && user.total_posts > (-posts_delta) as u64 {
                user.total_posts -= (-posts_delta) as u64;
            }
            
            if comments_delta > 0 {
                user.total_comments += comments_delta as u64;
            } else if comments_delta < 0 && user.total_comments > (-comments_delta) as u64 {
                user.total_comments -= (-comments_delta) as u64;
            }
            
            if points_delta > 0 {
                user.total_points += points_delta as u64;
            } else if points_delta < 0 && user.total_points > (-points_delta) as u64 {
                user.total_points -= (-points_delta) as u64;
            }
            
            // Update level based on points
            user.level = 1 + (user.total_points / 100); // Simple level calculation
            
            UserResponse {
                success: true,
                message: "User stats updated successfully".to_string(),
                user: Some(user.clone()),
            }
        } else {
            UserResponse {
                success: false,
                message: "User not found".to_string(),
                user: None,
            }
        }
    });
    
    result
}

// Unit tests
#[cfg(test)]
mod tests {
    use super::*;
    
    // Helper function to create a test principal
    fn test_principal() -> Principal {
        Principal::from_text("2vxsx-fae").unwrap()
    }
    
    // Helper function to reset the mock storage
    fn reset_mock_storage() {
        MOCK_USER_STORAGE.with(|storage| {
            *storage.borrow_mut() = MockUserStorage::default();
        });
    }
    
    // Helper function to create test badges
    fn create_test_badges() {
        mock_create_badge(
            "early_adopter".to_string(),
            "Early Adopter".to_string(),
            "Joined during the beta phase".to_string(),
            None,
        );
        
        mock_create_badge(
            "prolific_poster".to_string(),
            "Prolific Poster".to_string(),
            "Created more than 50 posts".to_string(),
            None,
        );
    }
    
    #[test]
    fn test_register_user() {
        reset_mock_storage();
        
        let user = test_principal();
        let username = "testuser".to_string();
        
        let response = mock_register_user(user, username);
        
        assert!(response.success);
        assert!(response.user.is_some());
        let profile = response.user.unwrap();
        assert_eq!(profile.principal, user);
        assert_eq!(profile.username, "testuser");
        assert_eq!(profile.level, 1);
        assert_eq!(profile.total_points, 0);
        assert!(profile.badges.is_empty());
    }
    
    #[test]
    fn test_register_invalid_username() {
        reset_mock_storage();
        
        let user = test_principal();
        
        // Username too short
        let response1 = mock_register_user(user, "ab".to_string());
        assert!(!response1.success);
        
        // Username with invalid characters
        let response2 = mock_register_user(user, "test-user".to_string());
        assert!(!response2.success);
    }
    
    #[test]
    fn test_register_duplicate_username() {
        reset_mock_storage();
        
        let user1 = test_principal();
        let user2 = Principal::from_text("rrkah-fqaaa-aaaaa-aaaaq-cai").unwrap();
        let username = "testuser".to_string();
        
        // Register first user
        let response1 = mock_register_user(user1, username.clone());
        assert!(response1.success);
        
        // Try to register second user with same username
        let response2 = mock_register_user(user2, username);
        assert!(!response2.success);
        assert!(response2.message.contains("Username already taken"));
    }
    
    #[test]
    fn test_get_user() {
        reset_mock_storage();
        
        let user = test_principal();
        let username = "testuser".to_string();
        
        // Register user
        mock_register_user(user, username);
        
        // Get user
        let response = mock_get_user(user);
        
        assert!(response.success);
        assert!(response.user.is_some());
        let profile = response.user.unwrap();
        assert_eq!(profile.username, "testuser");
    }
    
    #[test]
    fn test_get_user_by_username() {
        reset_mock_storage();
        
        let user = test_principal();
        let username = "testuser".to_string();
        
        // Register user
        mock_register_user(user, username.clone());
        
        // Get user by username
        let response = mock_get_user_by_username(username);
        
        assert!(response.success);
        assert!(response.user.is_some());
        let profile = response.user.unwrap();
        assert_eq!(profile.principal, user);
    }
    
    #[test]
    fn test_update_profile() {
        reset_mock_storage();
        
        let user = test_principal();
        let username = "testuser".to_string();
        
        // Register user
        mock_register_user(user, username);
        
        // Update profile
        let update_request = UpdateProfileRequest {
            display_name: Some("Test User".to_string()),
            bio: Some("This is my bio".to_string()),
            avatar_url: Some("https://example.com/avatar.jpg".to_string()),
        };
        
        let response = mock_update_profile(user, update_request);
        
        assert!(response.success);
        assert!(response.user.is_some());
        let profile = response.user.unwrap();
        assert_eq!(profile.display_name.unwrap(), "Test User");
        assert_eq!(profile.bio.unwrap(), "This is my bio");
        assert_eq!(profile.avatar_url.unwrap(), "https://example.com/avatar.jpg");
        assert!(profile.updated_at.is_some());
    }
    
    #[test]
    fn test_award_badge() {
        reset_mock_storage();
        create_test_badges();
        
        let user = test_principal();
        let username = "testuser".to_string();
        
        // Register user
        mock_register_user(user, username);
        
        // Award badge
        let response = mock_award_badge(user, "early_adopter".to_string());
        
        assert!(response.success);
        assert!(response.user.is_some());
        let profile = response.user.unwrap();
        assert_eq!(profile.badges.len(), 1);
        assert_eq!(profile.badges[0], "early_adopter");
    }
    
    #[test]
    fn test_award_duplicate_badge() {
        reset_mock_storage();
        create_test_badges();
        
        let user = test_principal();
        let username = "testuser".to_string();
        
        // Register user
        mock_register_user(user, username);
        
        // Award badge
        mock_award_badge(user, "early_adopter".to_string());
        
        // Try to award the same badge again
        let response = mock_award_badge(user, "early_adopter".to_string());
        
        assert!(!response.success);
        assert!(response.message.contains("User already has this badge"));
    }
    
    #[test]
    fn test_update_user_stats() {
        reset_mock_storage();
        
        let user = test_principal();
        let username = "testuser".to_string();
        
        // Register user
        mock_register_user(user, username);
        
        // Update stats
        let response = mock_update_user_stats(user, 5, 10, 150);
        
        assert!(response.success);
        assert!(response.user.is_some());
        let profile = response.user.unwrap();
        assert_eq!(profile.total_posts, 5);
        assert_eq!(profile.total_comments, 10);
        assert_eq!(profile.total_points, 150);
        assert_eq!(profile.level, 2); // Level should be 1 + (150 / 100) = 2
    }
    
    #[test]
    fn test_get_top_users() {
        reset_mock_storage();
        
        // Register multiple users with different points
        let user1 = test_principal();
        let user2 = Principal::from_text("rrkah-fqaaa-aaaaa-aaaaq-cai").expect("Invalid principal ID");
        let user3 = Principal::from_text("ryjl3-tyaaa-aaaaa-aaaba-cai").expect("Invalid principal ID");
        
        mock_register_user(user1, "user1".to_string());
        mock_register_user(user2, "user2".to_string());
        mock_register_user(user3, "user3".to_string());
        
        // Update points
        mock_update_user_stats(user1, 0, 0, 100);
        mock_update_user_stats(user2, 0, 0, 300);
        mock_update_user_stats(user3, 0, 0, 200);
        
        // Get top 2 users
        let response = mock_get_top_users(2);
        
        assert!(response.success);
        assert_eq!(response.users.len(), 2);
        assert_eq!(response.total, 2);
        
        // Check that the users are sorted by points (highest first)
        let has_user2_with_300_points = response.users.iter().any(|u| u.principal == user2 && u.total_points == 300);
        let has_user3_with_200_points = response.users.iter().any(|u| u.principal == user3 && u.total_points == 200);
        
        assert!(has_user2_with_300_points, "Top users should include user2 with 300 points");
        assert!(has_user3_with_200_points, "Top users should include user3 with 200 points");
        
        // Verify the order - first user should have more points than second
        assert!(response.users[0].total_points > response.users[1].total_points);
    }
    
    #[test]
    fn test_negative_stats_update() {
        reset_mock_storage();
        
        let user = test_principal();
        let username = "testuser".to_string();
        
        // Register user
        mock_register_user(user, username);
        
        // Add some initial stats
        mock_update_user_stats(user, 10, 20, 300);
        
        // Update with negative deltas
        let response = mock_update_user_stats(user, -3, -5, -50);
        
        assert!(response.success);
        assert!(response.user.is_some());
        let profile = response.user.unwrap();
        assert_eq!(profile.total_posts, 7);
        assert_eq!(profile.total_comments, 15);
        assert_eq!(profile.total_points, 250);
        assert_eq!(profile.level, 3); // Level should be 1 + (250 / 100) = 3
    }
}
