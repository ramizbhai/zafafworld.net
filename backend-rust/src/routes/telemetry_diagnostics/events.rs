use axum::{
    extract::State,
    response::sse::{Event, KeepAlive, Sse},
    routing::get,
    Router,
};
use serde_json::json;
use std::convert::Infallible;
use uuid::Uuid;

use crate::errors::AppError;
use crate::middleware::auth::RequireAuth;
use crate::models::user::DomainType;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new().route("/stream", get(events_stream))
}

async fn events_stream(
    auth: RequireAuth,
    State(state): State<AppState>,
) -> Result<Sse<impl tokio_stream::Stream<Item = Result<Event, Infallible>>>, AppError> {
    let user_uuid = Uuid::parse_str(&auth.user_id)
        .map_err(|_| AppError::BadRequest("Invalid user ID format in token".to_string()))?;

    let mut rx = state.booking_event_tx.subscribe();
    let mut chat_rx = state.chat_event_tx.subscribe();

    let stream = async_stream::stream! {
        yield Ok(Event::default().event("connected").data("connection established"));

        loop {
            tokio::select! {
                res = rx.recv() => {
                    match res {
                        Ok(event) => {
                            match auth.role {
                                DomainType::Vendor if event.vendor_id == user_uuid => {
                                    let payload = json!({
                                        "bookingNumber": event.booking_number,
                                        "clientId": event.client_id,
                                        "totalPrice": event.total_price,
                                        "timestamp": event.timestamp,
                                    });
                                    if let Ok(event_obj) = Event::default().event("NEW_LEAD_RECEIVED").json_data(&payload) {
                                        yield Ok(event_obj);
                                    }
                                }
                                DomainType::Admin => {
                                    let payload = json!({
                                        "bookingNumber": event.booking_number,
                                        "clientId": event.client_id,
                                        "vendorId": event.vendor_id,
                                        "totalPrice": event.total_price,
                                        "timestamp": event.timestamp,
                                    });
                                    if let Ok(event_obj) = Event::default().event("SYSTEM_TRANSACTION_LOGGED").json_data(&payload) {
                                        yield Ok(event_obj);
                                    }
                                }
                                _ => {}
                            }
                        }
                        Err(tokio::sync::broadcast::error::RecvError::Lagged(skipped)) => {
                            tracing::warn!("Event stream receiver lagged, skipped {} messages", skipped);
                        }
                        Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                            tracing::info!("Event stream channel closed");
                            break;
                        }
                    }
                }
                res = chat_rx.recv() => {
                    match res {
                        Ok(event) => {
                            match event {
                                crate::state::ChatEvent::NewMessage {
                                    conversation_id,
                                    message_id,
                                    sender_id,
                                    body,
                                    temp_id: _,
                                    attachments,
                                    participant_ids,
                                    created_at,
                                } => {
                                    if participant_ids.contains(&user_uuid) {
                                        let payload = json!({
                                            "conversationId": conversation_id,
                                            "messageId": message_id,
                                            "senderId": sender_id,
                                            "body": body,
                                            "createdAt": created_at,
                                            "attachments": attachments,
                                        });
                                        if let Ok(event_obj) = Event::default().event("NEW_MESSAGE_RECEIVED").json_data(&payload) {
                                            yield Ok(event_obj);
                                        }
                                    }
                                }
                                crate::state::ChatEvent::ReadReceipt {
                                    conversation_id,
                                    message_id,
                                    user_id,
                                    participant_ids,
                                    read_at,
                                } => {
                                    if participant_ids.contains(&user_uuid) {
                                        let payload = json!({
                                            "conversationId": conversation_id,
                                            "messageId": message_id,
                                            "userId": user_id,
                                            "readAt": read_at,
                                        });
                                        if let Ok(event_obj) = Event::default().event("READ_RECEIPT_RECEIVED").json_data(&payload) {
                                            yield Ok(event_obj);
                                        }
                                    }
                                }
                            }
                        }
                        Err(tokio::sync::broadcast::error::RecvError::Lagged(skipped)) => {
                            tracing::warn!("Chat event stream receiver lagged, skipped {} messages", skipped);
                        }
                        Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                            tracing::info!("Chat event stream channel closed");
                            break;
                        }
                    }
                }
            }
        }
    };

    Ok(Sse::new(stream).keep_alive(KeepAlive::default()))
}

pub async fn publish_telemetry_event(
    _db: &sqlx::PgPool,
    event_type: &str,
    payload: serde_json::Value,
) -> Result<(), AppError> {
    tracing::info!(event_type = %event_type, payload = ?payload, "Telemetry event published");
    Ok(())
}
