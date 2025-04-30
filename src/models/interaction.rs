use candid::{CandidType, Deserialize, Principal};
use crate::models::content::ContentType;

// Request DTOs
#[derive(CandidType, Deserialize, Clone)]
pub struct LikeContentRequest {
    pub content_id: String,
    pub content_type: ContentType,
}

#[derive(CandidType, Deserialize, Clone)]
pub struct ReportContentRequest {
    pub content_id: String,
    pub content_type: ContentType,
    pub reason: ReportReason,
    pub description: Option<String>,
}

// Response DTOs
#[derive(CandidType, Deserialize, Clone)]
pub struct InteractionResponse {
    pub success: bool,
    pub message: String,
}

#[derive(CandidType, Deserialize, Clone)]
pub struct InteractionCountsResponse {
    pub likes: u64,
    pub comments: u64,
    pub is_liked_by_caller: bool,
}

#[derive(CandidType, Deserialize, Clone)]
pub struct UserLikeInfo {
    pub principal: Principal,
    pub username: String,
    pub timestamp: u64,
}

#[derive(CandidType, Deserialize, Clone)]
pub struct LikesResponse {
    pub content_id: String,
    pub content_type: ContentType,
    pub likes: Vec<UserLikeInfo>,
    pub total: u64,
}

// Enums
#[derive(CandidType, Deserialize, Clone, PartialEq, Debug)]
pub enum SharingPlatform {
    Telegram,
    Twitter,
    Facebook,
    LinkedIn,
    Email,
    Other,
}

#[derive(CandidType, Deserialize, Clone, PartialEq, Debug)]
pub enum ReportReason {
    Spam,
    Harassment,
    FalseInformation,
    Violence,
    Scam,
    IllegalContent,
    Other,
}

// Content view tracking
#[derive(CandidType, Deserialize, Clone)]
pub struct ViewContentRequest {
    pub content_id: String,
    pub content_type: ContentType,
}

// Content report management
#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct ContentReport {
    pub id: String,
    pub content_id: String,
    pub content_type: ContentType,
    pub reporter: Principal,
    pub reason: ReportReason,
    pub description: Option<String>,
    pub status: ReportStatus,
    pub created_at: u64,
    pub resolved_at: Option<u64>,
    pub resolver: Option<Principal>,
    pub resolution_notes: Option<String>,
}

#[derive(CandidType, Deserialize, Clone, PartialEq, Debug)]
pub enum ReportStatus {
    Pending,
    Resolved,
    Rejected,
}

#[derive(CandidType, Deserialize, Clone)]
pub struct ResolveReportRequest {
    pub report_id: String,
    pub status: ReportStatus,
    pub notes: Option<String>,
}
