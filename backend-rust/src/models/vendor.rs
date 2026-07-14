use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct Vendor {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub name_ar: String,
    pub name_en: String,
    pub slug: String,
    pub category: String,
    pub city_id: Option<Uuid>,
    pub status: String, // 'pending' | 'approved'
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
