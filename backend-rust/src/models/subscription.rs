use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct VendorSubscriptionRequest {
    pub id: Uuid,
    pub vendor_id: Uuid,
    pub requested_tier_id: Uuid,
    pub status: String, // pending, approved, rejected
    pub admin_notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct SubscriptionTier {
    pub id: Uuid,
    pub name: String,
    pub priority_score: i32,
    pub policy_limits: serde_json::Value,
    pub price: rust_decimal::Decimal,
    pub billing_cycle: String,
    pub features: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
