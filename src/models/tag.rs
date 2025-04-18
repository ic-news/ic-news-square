use candid::{CandidType, Deserialize};

#[derive(CandidType, Deserialize, Clone, PartialEq, Debug)]
pub enum TagType {
    Topic,
    Category,
    Location,
    Custom
}

#[derive(CandidType, Deserialize, Clone)]
pub struct TagStats {
    pub tag: String,
    pub tag_type: TagType,
    pub post_count: u64,
    pub last_used: u64,
}

#[derive(CandidType, Deserialize, Clone)]
pub struct GetHotTagsRequest {
    pub tag_type: Option<TagType>,
    pub limit: Option<usize>,  // Default to 20 if not specified
}

#[derive(CandidType, Deserialize, Clone)]
pub struct HotTagsResponse {
    pub tags: Vec<TagStats>,
    pub updated_at: u64,
}
