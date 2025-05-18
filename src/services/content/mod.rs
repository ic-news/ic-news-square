use candid::Principal;
use crate::models::content::{ContentType, PaginationParams};
use crate::models::display::{FeedResponse, ContentDetailResponse};
use crate::SquareResult;

// Re-export submodules
pub mod posts;
pub mod comments;
pub mod moderation;
pub mod display;

// Re-export commonly used functions
pub use posts::{
    create_post,
    get_post,
    get_posts,
    update_post,
    delete_post,
};

pub use comments::{
    create_comment,
    get_comment,
    get_comments,
    update_comment,
    delete_comment,
};

pub use moderation::{
    moderate_content,
};

pub use display::{
    get_user_content,
    get_content_detail,
};
