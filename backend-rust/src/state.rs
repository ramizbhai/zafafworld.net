use axum::extract::ws::Message;
use dashmap::DashMap;
use rust_decimal::Decimal;
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::broadcast;
use tokio::sync::mpsc;

pub type Tx = mpsc::UnboundedSender<Message>;

#[derive(Clone, Debug, serde::Serialize)]
pub struct BookingEvent {
    pub booking_number: String,
    pub client_id: uuid::Uuid,
    pub vendor_id: uuid::Uuid,
    pub total_price: Decimal,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Clone, Debug, serde::Serialize)]
#[serde(tag = "type")]
pub enum ChatEvent {
    NewMessage {
        conversation_id: uuid::Uuid,
        message_id: uuid::Uuid,
        sender_id: uuid::Uuid,
        body: String,
        temp_id: Option<String>,
        attachments: Vec<serde_json::Value>,
        participant_ids: Vec<uuid::Uuid>,
        created_at: chrono::DateTime<chrono::Utc>,
    },
    ReadReceipt {
        conversation_id: uuid::Uuid,
        message_id: uuid::Uuid,
        user_id: uuid::Uuid,
        participant_ids: Vec<uuid::Uuid>,
        read_at: chrono::DateTime<chrono::Utc>,
    },
}

#[derive(Clone)]
pub struct WsConn {
    pub id: uuid::Uuid,
    pub tx: Tx,
}

pub struct WsManager {
    /// Outer key: user UUID.
    /// Inner key: connection UUID (WsConn.id).
    /// HashMap inner map gives O(1) deregister instead of O(n) `Vec::retain`.
    pub connections: DashMap<uuid::Uuid, std::collections::HashMap<uuid::Uuid, WsConn>>,
}

impl WsManager {
    pub fn new() -> Self {
        Self {
            connections: DashMap::new(),
        }
    }

    pub fn register(&self, user_id: uuid::Uuid, conn: WsConn) {
        self.connections
            .entry(user_id)
            .or_default()
            .insert(conn.id, conn);
    }

    /// O(1) removal — no scan required.
    pub fn deregister(&self, user_id: uuid::Uuid, conn_id: uuid::Uuid) {
        if let Some(mut conns) = self.connections.get_mut(&user_id) {
            conns.remove(&conn_id);
        }
    }

    pub fn broadcast_to_user(&self, user_id: uuid::Uuid, msg: Message) {
        if let Some(conns) = self.connections.get(&user_id) {
            for conn in conns.values() {
                let _ = conn.tx.send(msg.clone());
            }
        }
    }
}

#[derive(Clone, Debug, serde::Serialize)]
pub struct InquiryEvent {
    pub inquiry_id: uuid::Uuid,
    pub vendor_id: uuid::Uuid,
    pub client_name: Option<String>,
    pub client_phone: Option<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub jwt_secret: String,
    pub frontend_url: String,
    pub email_service: std::sync::Arc<crate::services::email::EmailService>,
    pub whatsapp_service: std::sync::Arc<crate::services::whatsapp::WhatsappService>,
    pub booking_event_tx: broadcast::Sender<BookingEvent>,
    pub chat_event_tx: broadcast::Sender<ChatEvent>,
    #[allow(dead_code)]
    pub inquiry_event_tx: broadcast::Sender<InquiryEvent>,
    pub ws_manager: Arc<WsManager>,
    pub rate_limit_store:
        std::sync::Arc<dashmap::DashMap<String, crate::middleware::rate_limit::TokenBucket>>,
    pub idempotency_store:
        std::sync::Arc<dashmap::DashMap<String, crate::middleware::idempotency::IdempotentState>>,
    pub trusted_proxies: Vec<std::net::IpAddr>,
    /// MinIO client singleton — constructed once at startup from `config` and
    /// stored here so every handler can upload/delete without re-reading env.
    pub minio_client: Arc<crate::services::media::minio_client::MinioClient>,
    pub location_cache: std::sync::Arc<dashmap::DashMap<String, crate::services::location_resolver::CachedLocation>>,
    pub active_location_requests: std::sync::Arc<
        dashmap::DashMap<
            String,
            tokio::sync::broadcast::Sender<Result<crate::services::location_resolver::CachedLocation, String>>,
        >,
    >,
    /// Application configuration. Wrapped in `Arc` so the per-request
    /// `AppState::clone()` is a cheap atomic increment rather than a deep copy
    /// of all config strings.
    pub config: Arc<crate::config::AppConfig>,
}
