use candid::Principal;
use crate::models::discovery::*;
use crate::models::display::FeedResponse;
use crate::models::error::SquareResult;

// Re-export submodules
pub mod trending;
pub mod search;
pub mod recommendations;

// Re-export commonly used functions
pub use trending::{
    init_trending_topics,
    get_trending_topics,
    get_hot_tags,
    update_trending_content,
};

pub use search::{
    discover_content,
    search_content,
};

pub use recommendations::{
    get_personalized_recommendations,
    get_collaborative_recommendations,
};
