use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "user_domain_enum")]
pub enum DomainType {
    Client,
    Vendor,
    Admin,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AdminScope {
    SuperAdmin,
    SupportAdmin,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum VendorScope {
    Owner,
    Staff,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(untagged)]
pub enum UserScope {
    Admin(AdminScope),
    Vendor(VendorScope),
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub password_hash: String,
    pub domain_type: DomainType,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct Profile {
    pub client_id: Uuid,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub phone: Option<String>,
    pub wedding_date: Option<NaiveDate>,
    pub city_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct VendorProfile {
    pub vendor_id: Uuid,
    pub company_name_ar: Option<String>,
    pub company_name_en: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String, // User ID (stringified UUID)
    pub email: String,
    pub role: DomainType,
    pub scopes: Vec<UserScope>,
    pub exp: i64, // Expiration timestamp
    pub iat: i64, // Issued At timestamp
}
