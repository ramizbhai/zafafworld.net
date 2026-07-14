use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct Inquiry {
    pub id: Uuid,
    pub vendor_id: Uuid,
    pub customer_name: String,
    pub phone: String,
    pub wedding_date: NaiveDate,
    pub message: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub resolution_note: Option<String>,
}


#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct VendorReview {
    pub id: Uuid,
    pub vendor_id: Uuid,
    pub couple_name: String,
    pub rating: i32,
    pub comment: Option<String>,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct VendorTask {
    pub id: Uuid,
    pub vendor_id: Uuid,
    pub title_ar: String,
    pub title_en: String,
    pub is_completed: bool,
    pub due_date: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct VendorWhatsappTemplate {
    pub id: Uuid,
    pub vendor_id: Uuid,
    pub template_name: String,
    pub body_text_ar: Option<String>,
    pub body_text_en: Option<String>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct VendorStaff {
    pub id: Uuid,
    pub vendor_id: Uuid,
    pub name: String,
    pub email: String,
    pub role: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
#[allow(dead_code)]
pub struct VendorInquiry {
    pub id: Uuid,
    pub client_id: Uuid,
    pub vendor_id: Uuid,
    pub event_date: NaiveDate,
    pub guest_count: i32,
    pub message: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Rich data-projection mapping client details to render within the vendor portal
#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct VendorInquiryDto {
    pub id: Uuid,
    pub client_id: Option<Uuid>,
    pub vendor_id: Uuid,
    pub event_date: NaiveDate,
    pub guest_count: i32,
    pub message: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,

    // Joined client data (Leverages profiles/users data contexts safely)
    pub client_first_name: Option<String>,
    pub client_last_name: Option<String>,
    pub client_phone: Option<String>,
    pub client_email: Option<String>,

    // Guest details fallback
    pub name: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
}
