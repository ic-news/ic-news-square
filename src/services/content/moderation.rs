use candid::Principal;
use ic_cdk::api::time;

use crate::auth::is_admin;
use crate::models::content::{ContentModerationRequest, ContentStatus, ContentType};
use crate::models::storage::Storage;
use crate::{SquareError, SquareResult};
use crate::storage::STORAGE;
use crate::utils::error_handler::*;

pub fn moderate_content(request: ContentModerationRequest) -> SquareResult<()> {
    const MODULE: &str = "services::content::moderation";
    const FUNCTION: &str = "moderate_content";
    
    // Only admin can moderate content
    if is_admin().is_err() {
        return log_and_return(unauthorized_error(
            "Only admin can moderate content",
            MODULE,
            FUNCTION
        ));
    }
    
    STORAGE.with(|storage| {
        let mut store = storage.borrow_mut();
        
        match request.content_type {
            ContentType::Post => {
                let post = store.posts.get_mut(&request.content_id).ok_or_else(|| {
                    not_found_error("Post", &request.content_id, MODULE, FUNCTION)
                })?;
                
                post.status = request.status;
                post.updated_at = time() / 1_000_000;
            }
            ContentType::Comment => {
                let comment = store.comments.get_mut(&request.content_id).ok_or_else(|| {
                    not_found_error("Comment", &request.content_id, MODULE, FUNCTION)
                })?;
                
                comment.status = request.status;
                comment.updated_at = time() / 1_000_000;
            }
        }
        
        Ok(())
    })
}
