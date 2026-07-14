use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct VerifiedReview {
    pub id: Uuid,
    pub client_id: Uuid,
    pub vendor_id: Uuid,
    pub rating: i32,
    pub review_text: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct ReviewAttachment {
    pub id: Uuid,
    pub review_id: Uuid,
    pub file_path: String,
    pub created_at: DateTime<Utc>,
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct VerifiedReviewDto {
    pub id: Uuid,
    pub client_id: Uuid,
    pub vendor_id: Uuid,
    pub rating: i32,
    pub review_text: String,
    pub status: String,
    pub created_at: DateTime<Utc>,

    // Optional joined fields for client representation
    pub client_first_name: Option<String>,
    pub client_last_name: Option<String>,
    pub client_avatar: Option<String>,
}
