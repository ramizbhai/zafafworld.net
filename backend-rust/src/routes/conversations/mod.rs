use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path, Query, State,
    },
    response::IntoResponse,
    routing::{delete, get, patch, post},
    Json, Router,
};
use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use serde_json::{json, Value};
use sqlx::Row;
use uuid::Uuid;

use crate::errors::AppError;
use crate::middleware::auth::{RequireAuth, RlsTx};
use crate::state::{AppState, ChatEvent};
use crate::utils::sanitize::{limits, sanitize_str};

#[derive(Deserialize)]
pub struct CreateConversationInput {
    #[serde(alias = "participantId")]
    pub participant_id: Option<Uuid>,
    #[serde(alias = "productId", alias = "product_id")]
    pub product_id: Option<Uuid>,
    #[serde(alias = "vendorId", alias = "vendor_id")]
    pub vendor_id: Option<Uuid>,
    #[serde(alias = "initialMessage", alias = "initial_message")]
    pub initial_message: Option<String>,
}

pub(crate) async fn create_conversation(
    _auth: RequireAuth,
    mut rls_tx: RlsTx,
    Json(payload): Json<CreateConversationInput>,
) -> Result<Json<Value>, AppError> {
    let current_user_id: String =
        sqlx::query_scalar("SELECT NULLIF(current_setting('app.current_user_id', true), '')")
            .fetch_one(&mut *rls_tx.tx)
            .await?;
    let current_user_uuid = Uuid::parse_str(&current_user_id)
        .map_err(|_| AppError::BadRequest("Malformed user ID in session".to_string()))?;

    // Determine the vendor / other participant ID
    let target_id = payload
        .vendor_id
        .or(payload.participant_id)
        .ok_or_else(|| {
            AppError::BadRequest("vendor_id or participantId is required".to_string())
        })?;

    if current_user_uuid == target_id {
        return Err(AppError::BadRequest(
            "Cannot start a conversation with yourself".to_string(),
        ));
    }

    // Resolve target_user_uuid (must refer to global_users.id)
    let target_user_uuid =
        if sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM global_users WHERE id = $1")
            .bind(target_id)
            .fetch_one(&mut *rls_tx.tx)
            .await?
            > 0
        {
            target_id
        } else {
            // If not found in global_users, it could be a vendor profile ID (vendors.id).
            // Let's resolve it to vendors.user_id.
            let resolved_user_id: Option<Uuid> =
                sqlx::query_scalar("SELECT user_id FROM vendors WHERE id = $1")
                    .bind(target_id)
                    .fetch_optional(&mut *rls_tx.tx)
                    .await?;

            resolved_user_id.ok_or_else(|| {
                AppError::NotFound("Vendor or participant user not found".to_string())
            })?
        };

    if current_user_uuid == target_user_uuid {
        return Err(AppError::BadRequest(
            "Cannot start a conversation with yourself".to_string(),
        ));
    }

    // 1. Check if a conversation (optionally listing-bound) already exists
    let existing_id: Option<Uuid> = if let Some(pid) = payload.product_id {
        sqlx::query_scalar(
            "SELECT cp1.conversation_id FROM conversation_participants cp1
             JOIN conversation_participants cp2 ON cp1.conversation_id = cp2.conversation_id
             JOIN conversations c ON cp1.conversation_id = c.id
             WHERE cp1.user_id = $1 AND cp2.user_id = $2 AND c.product_id = $3",
        )
        .bind(current_user_uuid)
        .bind(target_user_uuid)
        .bind(pid)
        .fetch_optional(&mut *rls_tx.tx)
        .await?
    } else {
        sqlx::query_scalar(
            "SELECT cp1.conversation_id FROM conversation_participants cp1
             JOIN conversation_participants cp2 ON cp1.conversation_id = cp2.conversation_id
             JOIN conversations c ON cp1.conversation_id = c.id
             WHERE cp1.user_id = $1 AND cp2.user_id = $2 AND c.product_id IS NULL",
        )
        .bind(current_user_uuid)
        .bind(target_user_uuid)
        .fetch_optional(&mut *rls_tx.tx)
        .await?
    };

    if let Some(cid) = existing_id {
        // If it exists, retrieve and return it
        rls_tx.tx.commit().await?;
        return Ok(Json(json!({
            "status": "success",
            "conversation_id": cid.to_string(),
            "conversationId": cid.to_string(),
            "message": "Existing conversation retrieved",
            "data": {
                "id": cid.to_string(),
                "conversationId": cid.to_string()
            }
        })));
    }

    // Fetch city_id context if possible
    let city_id: Option<Uuid> = if let Some(pid) = payload.product_id {
        sqlx::query_scalar("SELECT v.city_id FROM vendor_products vp JOIN vendors v ON vp.vendor_id = v.id WHERE vp.id = $1")
            .bind(pid)
            .fetch_optional(&mut *rls_tx.tx)
            .await?
            .flatten()
    } else {
        sqlx::query_scalar("SELECT city_id FROM vendors WHERE user_id = $1")
            .bind(target_user_uuid)
            .fetch_optional(&mut *rls_tx.tx)
            .await?
            .flatten()
    };

    // 2. Create new conversation (with optional product_id context)
    let conv_id: Uuid = if let Some(pid) = payload.product_id {
        sqlx::query_scalar(
            "INSERT INTO conversations (product_id, city_id) VALUES ($1, $2) RETURNING id",
        )
        .bind(pid)
        .bind(city_id)
        .fetch_one(&mut *rls_tx.tx)
        .await?
    } else {
        sqlx::query_scalar("INSERT INTO conversations (city_id) VALUES ($1) RETURNING id")
            .bind(city_id)
            .fetch_one(&mut *rls_tx.tx)
            .await?
    };

    // 3. Add participants
    sqlx::query("INSERT INTO conversation_participants (conversation_id, user_id) VALUES ($1, $2)")
        .bind(conv_id)
        .bind(current_user_uuid)
        .execute(&mut *rls_tx.tx)
        .await?;

    sqlx::query("INSERT INTO conversation_participants (conversation_id, user_id) VALUES ($1, $2)")
        .bind(conv_id)
        .bind(target_user_uuid)
        .execute(&mut *rls_tx.tx)
        .await?;

    // 4. Send initial message if present
    let mut initial_msg_id = None;
    if let Some(ref msg_body) = payload.initial_message {
        let sanitized = sanitize_str(msg_body, limits::MESSAGE);
        if !sanitized.trim().is_empty() {
            let mid = Uuid::new_v4();
            sqlx::query(
                "INSERT INTO messages (id, conversation_id, sender_id, body) VALUES ($1, $2, $3, $4)"
            )
            .bind(mid)
            .bind(conv_id)
            .bind(current_user_uuid)
            .bind(&sanitized)
            .execute(&mut *rls_tx.tx)
            .await?;
            initial_msg_id = Some(mid);
        }
    }

    rls_tx.tx.commit().await?;

    Ok(Json(json!({
        "status": "success",
        "conversation_id": conv_id.to_string(),
        "conversationId": conv_id.to_string(),
        "message": "Conversation created successfully",
        "initial_message_id": initial_msg_id,
        "data": {
            "id": conv_id.to_string(),
            "conversationId": conv_id.to_string()
        }
    })))
}

pub(crate) async fn list_conversations(
    auth: RequireAuth,
    mut rls_tx: RlsTx,
) -> Result<Json<Value>, AppError> {
    let current_user_id: String =
        sqlx::query_scalar("SELECT NULLIF(current_setting('app.current_user_id', true), '')")
            .fetch_one(&mut *rls_tx.tx)
            .await?;
    let current_user_uuid = Uuid::parse_str(&current_user_id)
        .map_err(|_| AppError::BadRequest("Malformed user ID in session".to_string()))?;

    let conversations = if auth.role == crate::models::user::DomainType::Admin {
        // Admin lists ALL conversations on the platform
        sqlx::query(
            "SELECT 
                c.id, c.status, c.created_at, c.updated_at, c.product_id,
                p.title AS product_title,
                (
                    SELECT image_url FROM vendor_gallery 
                    WHERE product_id = c.product_id AND is_cover = TRUE 
                    LIMIT 1
                ) AS product_cover_image,
                p.base_price_sar::float8 AS product_price,
                (
                    SELECT json_agg(json_build_object(
                        'userId', u.id,
                        'email', u.email,
                        'role', u.domain_type,
                        'name', COALESCE(cl.first_name || ' ' || cl.last_name, vp.name_en, u.email)
                    ))
                    FROM conversation_participants cp
                    JOIN global_users u ON cp.user_id = u.id
                    LEFT JOIN client_profiles cl ON u.id = cl.client_id
                    LEFT JOIN vendors vp ON u.id = vp.user_id
                    WHERE cp.conversation_id = c.id
                ) as participants,
                lm.body as last_message_body,
                lm.created_at as last_message_time
             FROM conversations c
             LEFT JOIN vendor_products p ON c.product_id = p.id
             LEFT JOIN LATERAL (
                 SELECT body, created_at FROM messages
                 WHERE conversation_id = c.id
                 ORDER BY created_at DESC
                 LIMIT 1
             ) lm ON true
             ORDER BY COALESCE(lm.created_at, c.updated_at) DESC",
        )
        .fetch_all(&mut *rls_tx.tx)
        .await?
        .into_iter()
        .map(|row| {
            let id: Uuid = row.get("id");
            let status: String = row.get("status");
            let created_at: chrono::DateTime<chrono::Utc> = row.get("created_at");
            let updated_at: chrono::DateTime<chrono::Utc> = row.get("updated_at");
            let product_id: Option<Uuid> = row.get("product_id");
            let product_title: Option<String> = row.get("product_title");
            let product_cover_image: Option<String> = row.get("product_cover_image");
            let product_price: Option<f64> = row.get("product_price");
            let participants: Option<Value> = row.get("participants");
            let last_message_body: Option<String> = row.get("last_message_body");
            let last_message_time: Option<chrono::DateTime<chrono::Utc>> =
                row.get("last_message_time");

            json!({
                "id": id,
                "status": status,
                "created_at": created_at,
                "updated_at": updated_at,
                "productId": product_id,
                "productTitle": product_title,
                "productCoverImage": product_cover_image,
                "productPrice": product_price,
                "participants": participants.unwrap_or(json!([])),
                "lastMessage": {
                    "body": last_message_body,
                    "createdAt": last_message_time
                }
            })
        })
        .collect::<Vec<Value>>()
    } else {
        // Regular user lists only their own conversations
        sqlx::query(
            "SELECT 
                c.id, c.status, c.created_at, c.updated_at, c.product_id,
                p.title AS product_title,
                (
                    SELECT image_url FROM vendor_gallery 
                    WHERE product_id = c.product_id AND is_cover = TRUE 
                    LIMIT 1
                ) AS product_cover_image,
                p.base_price_sar::float8 AS product_price,
                other_u.id as other_user_id, other_u.email as other_email, other_u.domain_type as other_role,
                cl.first_name as client_first_name, cl.last_name as client_last_name,
                vp.name_en as vendor_company_name,
                lm.body as last_message_body, lm.created_at as last_message_time, lm.sender_id as last_message_sender,
                (
                    SELECT COUNT(*)::int FROM messages m
                    WHERE m.conversation_id = c.id
                      AND m.sender_id != $1
                      AND NOT EXISTS (
                          SELECT 1 FROM message_read_receipts r
                          WHERE r.message_id = m.id AND r.user_id = $1
                      )
                ) as unread_count
             FROM conversations c
             JOIN conversation_participants cp1 ON c.id = cp1.conversation_id AND cp1.user_id = $1
             JOIN conversation_participants cp2 ON c.id = cp2.conversation_id AND cp2.user_id != $1
             JOIN global_users other_u ON cp2.user_id = other_u.id
             LEFT JOIN client_profiles cl ON other_u.id = cl.client_id
             LEFT JOIN vendors vp ON other_u.id = vp.user_id
             LEFT JOIN vendor_products p ON c.product_id = p.id
             LEFT JOIN LATERAL (
                 SELECT body, created_at, sender_id FROM messages
                 WHERE conversation_id = c.id
                 ORDER BY created_at DESC
                 LIMIT 1
             ) lm ON true
             WHERE c.status = 'active'
             ORDER BY COALESCE(lm.created_at, c.updated_at) DESC"
        )
        .bind(current_user_uuid)
        .fetch_all(&mut *rls_tx.tx)
        .await?
        .into_iter()
        .map(|row| {
            let id: Uuid = row.get("id");
            let status: String = row.get("status");
            let created_at: chrono::DateTime<chrono::Utc> = row.get("created_at");
            let updated_at: chrono::DateTime<chrono::Utc> = row.get("updated_at");
            let product_id: Option<Uuid> = row.get("product_id");
            let product_title: Option<String> = row.get("product_title");
            let product_cover_image: Option<String> = row.get("product_cover_image");
            let product_price: Option<f64> = row.get("product_price");
            let other_user_id: Uuid = row.get("other_user_id");
            let other_email: String = row.get("other_email");
            let other_role: crate::models::user::DomainType = row.get("other_role");
            let client_first_name: Option<String> = row.get("client_first_name");
            let client_last_name: Option<String> = row.get("client_last_name");
            let vendor_company_name: Option<String> = row.get("vendor_company_name");
            let last_message_body: Option<String> = row.get("last_message_body");
            let last_message_time: Option<chrono::DateTime<chrono::Utc>> = row.get("last_message_time");
            let last_message_sender: Option<Uuid> = row.get("last_message_sender");
            let unread_count: i32 = row.get("unread_count");

            let other_name = match other_role {
                crate::models::user::DomainType::Client => {
                    let first = client_first_name.unwrap_or_default();
                    let last = client_last_name.unwrap_or_default();
                    if first.is_empty() && last.is_empty() {
                        other_email.clone()
                    } else {
                        format!("{} {}", first, last).trim().to_string()
                    }
                }
                crate::models::user::DomainType::Vendor => {
                    vendor_company_name.unwrap_or(other_email.clone())
                }
                crate::models::user::DomainType::Admin => {
                    "Platform Administrator".to_string()
                }
            };

            json!({
                "id": id,
                "status": status,
                "created_at": created_at,
                "updated_at": updated_at,
                "unread_count": unread_count,
                "productId": product_id,
                "productTitle": product_title,
                "productCoverImage": product_cover_image,
                "productPrice": product_price,
                "otherParticipant": {
                    "id": other_user_id,
                    "email": other_email,
                    "role": other_role,
                    "name": other_name
                },
                "lastMessage": {
                    "body": last_message_body,
                    "createdAt": last_message_time,
                    "senderId": last_message_sender
                }
            })
        })
        .collect::<Vec<Value>>()
    };

    rls_tx.tx.commit().await?;

    Ok(Json(json!({
        "status": "success",
        "conversations": conversations,
        "data": conversations
    })))
}

pub(crate) async fn get_conversation(
    _auth: RequireAuth,
    mut rls_tx: RlsTx,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, AppError> {
    let row = sqlx::query(
        "SELECT c.id, c.title, c.status, c.created_at, c.updated_at, c.product_id,
                p.title AS product_title,
                (
                    SELECT image_url FROM vendor_gallery 
                    WHERE product_id = c.product_id AND is_cover = TRUE 
                    LIMIT 1
                ) AS product_cover_image,
                p.base_price_sar::float8 AS product_price
         FROM conversations c
         LEFT JOIN vendor_products p ON c.product_id = p.id
         WHERE c.id = $1",
    )
    .bind(id)
    .fetch_optional(&mut *rls_tx.tx)
    .await?;

    let conv = match row {
        Some(r) => r,
        None => return Err(AppError::NotFound("Conversation not found".to_string())),
    };

    let id: Uuid = conv.get("id");
    let title: Option<String> = conv.get("title");
    let status: String = conv.get("status");
    let created_at: chrono::DateTime<chrono::Utc> = conv.get("created_at");
    let updated_at: chrono::DateTime<chrono::Utc> = conv.get("updated_at");
    let product_id: Option<Uuid> = conv.get("product_id");
    let product_title: Option<String> = conv.get("product_title");
    let product_cover_image: Option<String> = conv.get("product_cover_image");
    let product_price: Option<f64> = conv.get("product_price");

    rls_tx.tx.commit().await?;

    let conv_json = json!({
        "id": id,
        "title": title,
        "status": status,
        "created_at": created_at,
        "updated_at": updated_at,
        "productId": product_id,
        "productTitle": product_title,
        "productCoverImage": product_cover_image,
        "productPrice": product_price
    });

    Ok(Json(json!({
        "status": "success",
        "conversation": conv_json,
        "data": conv_json
    })))
}

#[derive(Deserialize)]
pub struct AttachmentInput {
    #[serde(alias = "fileName", alias = "file_name")]
    pub file_name: String,
    #[serde(alias = "fileUrl", alias = "file_url")]
    pub file_url: String,
    #[serde(alias = "fileType", alias = "file_type")]
    pub file_type: String,
    #[serde(alias = "fileSize", alias = "file_size")]
    pub file_size: i32,
}

#[derive(Deserialize)]
pub struct SendMessageInput {
    pub body: String,
    pub attachments: Option<Vec<AttachmentInput>>,
    #[serde(alias = "tempId", alias = "temp_id")]
    pub temp_id: Option<String>,
}

pub(crate) async fn send_message(
    State(state): State<AppState>,
    _auth: RequireAuth,
    mut rls_tx: RlsTx,
    Path(conversation_id): Path<Uuid>,
    Json(payload): Json<SendMessageInput>,
) -> Result<Json<Value>, AppError> {
    let sanitized_body = sanitize_str(&payload.body, limits::MESSAGE);
    let has_attachments = payload
        .attachments
        .as_ref()
        .map(|a| !a.is_empty())
        .unwrap_or(false);

    if sanitized_body.trim().is_empty() && !has_attachments {
        return Err(AppError::BadRequest(
            "Message body or attachments must be provided".to_string(),
        ));
    }

    let current_user_id: String =
        sqlx::query_scalar("SELECT NULLIF(current_setting('app.current_user_id', true), '')")
            .fetch_one(&mut *rls_tx.tx)
            .await?;
    let current_user_uuid = Uuid::parse_str(&current_user_id)
        .map_err(|_| AppError::BadRequest("Malformed user ID in session".to_string()))?;

    // 1. Double check if conversation is closed
    let conv_status: Option<String> =
        sqlx::query_scalar("SELECT status FROM conversations WHERE id = $1")
            .bind(conversation_id)
            .fetch_optional(&mut *rls_tx.tx)
            .await?;

    match conv_status {
        Some(ref status) if status == "closed" => {
            return Err(AppError::BadRequest(
                "Cannot send messages to a closed conversation".to_string(),
            ));
        }
        None => return Err(AppError::NotFound("Conversation not found".to_string())),
        _ => {}
    }

    // 2. Insert message
    let message_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO messages (id, conversation_id, sender_id, body) VALUES ($1, $2, $3, $4)",
    )
    .bind(message_id)
    .bind(conversation_id)
    .bind(current_user_uuid)
    .bind(if sanitized_body.trim().is_empty() {
        None
    } else {
        Some(&sanitized_body)
    })
    .execute(&mut *rls_tx.tx)
    .await?;

    // 3. Insert attachments if any
    let mut inserted_attachments = Vec::new();
    if let Some(attachments) = payload.attachments {
        for att in attachments {
            let attachment_id = Uuid::new_v4();
            sqlx::query(
                "INSERT INTO message_attachments (id, message_id, file_name, file_url, file_type, file_size)
                 VALUES ($1, $2, $3, $4, $5, $6)"
            )
            .bind(attachment_id)
            .bind(message_id)
            .bind(&att.file_name)
            .bind(&att.file_url)
            .bind(&att.file_type)
            .bind(att.file_size)
            .execute(&mut *rls_tx.tx)
            .await?;

            inserted_attachments.push(json!({
                "id": attachment_id,
                "fileName": att.file_name,
                "fileUrl": att.file_url,
                "fileType": att.file_type,
                "fileSize": att.file_size
            }));
        }
    }

    // 4. Update conversation updated_at
    sqlx::query("UPDATE conversations SET updated_at = NOW() WHERE id = $1")
        .bind(conversation_id)
        .execute(&mut *rls_tx.tx)
        .await?;

    // 5. Retrieve participants of this conversation to broadcast to
    let participants: Vec<Uuid> = sqlx::query_scalar(
        "SELECT user_id FROM conversation_participants WHERE conversation_id = $1",
    )
    .bind(conversation_id)
    .fetch_all(&mut *rls_tx.tx)
    .await?;

    rls_tx.tx.commit().await?;

    // 6. Broadcast onto broadcast channel (handles both WebSocket and SSE streams)
    let mut broadcast_ids = participants.clone();
    if !broadcast_ids.contains(&current_user_uuid) {
        broadcast_ids.push(current_user_uuid);
    }

    let _ = state.chat_event_tx.send(ChatEvent::NewMessage {
        conversation_id,
        message_id,
        sender_id: current_user_uuid,
        body: sanitized_body.clone(),
        temp_id: payload.temp_id,
        attachments: inserted_attachments,
        participant_ids: broadcast_ids,
        created_at: chrono::Utc::now(),
    });

    Ok(Json(json!({
        "status": "success",
        "message_id": message_id,
        "messageId": message_id,
        "message": "Message sent successfully",
        "data": {
            "id": message_id,
            "messageId": message_id
        }
    })))
}

pub(crate) async fn list_messages(
    _auth: RequireAuth,
    mut rls_tx: RlsTx,
    Path(conversation_id): Path<Uuid>,
) -> Result<Json<Value>, AppError> {
    let messages = sqlx::query(
        "SELECT 
            m.id, m.sender_id, COALESCE(m.body, '') AS body, m.created_at,
            (
                SELECT COALESCE(json_agg(json_build_object(
                    'id', a.id,
                    'fileName', a.file_name,
                    'fileUrl', a.file_url,
                    'fileType', a.file_type,
                    'fileSize', a.file_size
                )), '[]'::json)
                FROM message_attachments a WHERE a.message_id = m.id
            ) as attachments,
            (
                SELECT COALESCE(json_agg(json_build_object(
                    'userId', r.user_id,
                    'readAt', r.read_at
                )), '[]'::json)
                FROM message_read_receipts r WHERE r.message_id = m.id
            ) as read_receipts
         FROM messages m
         WHERE m.conversation_id = $1
         ORDER BY m.created_at ASC",
    )
    .bind(conversation_id)
    .fetch_all(&mut *rls_tx.tx)
    .await?
    .into_iter()
    .map(|row| {
        let id: Uuid = row.get("id");
        let sender_id: Uuid = row.get("sender_id");
        let body: String = row.get("body");
        let created_at: chrono::DateTime<chrono::Utc> = row.get("created_at");
        let attachments: Value = row.get("attachments");
        let read_receipts: Value = row.get("read_receipts");

        json!({
            "id": id,
            "senderId": sender_id,
            "body": body,
            "createdAt": created_at,
            "attachments": attachments,
            "readReceipts": read_receipts
        })
    })
    .collect::<Vec<Value>>();

    rls_tx.tx.commit().await?;

    Ok(Json(json!({
        "status": "success",
        "messages": messages,
        "data": messages
    })))
}

pub(crate) async fn mark_message_read(
    _auth: RequireAuth,
    State(state): State<AppState>,
    mut rls_tx: RlsTx,
    Path(message_id): Path<Uuid>,
) -> Result<Json<Value>, AppError> {
    let current_user_id: String =
        sqlx::query_scalar("SELECT NULLIF(current_setting('app.current_user_id', true), '')")
            .fetch_one(&mut *rls_tx.tx)
            .await?;
    let current_user_uuid = Uuid::parse_str(&current_user_id)
        .map_err(|_| AppError::BadRequest("Malformed user ID in session".to_string()))?;

    // Get conversation_id
    let conversation_id: Option<Uuid> =
        sqlx::query_scalar("SELECT conversation_id FROM messages WHERE id = $1")
            .bind(message_id)
            .fetch_optional(&mut *rls_tx.tx)
            .await?;

    let conversation_id = match conversation_id {
        Some(id) => id,
        None => return Err(AppError::NotFound("Message not found".to_string())),
    };

    // Mark preceding messages in the conversation as read for the current user
    sqlx::query(
        "INSERT INTO message_read_receipts (message_id, user_id)
         SELECT m.id, $2
         FROM messages m
         WHERE m.conversation_id = $1
           AND m.created_at <= (SELECT created_at FROM messages WHERE id = $3)
           AND m.sender_id != $2
         ON CONFLICT (message_id, user_id) DO NOTHING",
    )
    .bind(conversation_id)
    .bind(current_user_uuid)
    .bind(message_id)
    .execute(&mut *rls_tx.tx)
    .await?;

    // Fetch participants of the conversation
    let participants: Vec<Uuid> = sqlx::query_scalar(
        "SELECT user_id FROM conversation_participants WHERE conversation_id = $1",
    )
    .bind(conversation_id)
    .fetch_all(&mut *rls_tx.tx)
    .await?;

    rls_tx.tx.commit().await?;

    // Broadcast onto event bus (updates active WebSockets and SSE streams)
    let _ = state
        .chat_event_tx
        .send(crate::state::ChatEvent::ReadReceipt {
            conversation_id,
            message_id,
            user_id: current_user_uuid,
            participant_ids: participants,
            read_at: chrono::Utc::now(),
        });

    Ok(Json(json!({
        "status": "success",
        "message": "Messages marked as read"
    })))
}

pub(crate) async fn upload_attachment(
    axum::extract::State(state): axum::extract::State<crate::state::AppState>,
    _auth: RequireAuth,
    mut multipart: axum::extract::Multipart,
) -> Result<Json<Value>, AppError> {
    let mut file_name = String::new();
    let mut file_url = String::new();
    let mut file_type = String::new();
    let mut file_size = 0;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|err| AppError::BadRequest(err.to_string()))?
    {
        let name = field.name().unwrap_or("").to_string();
        if name == "file" {
            let original_name = field.file_name().unwrap_or("upload.png").to_string();
            let target_dir = "assets/uploads/attachments/";
            let url_prefix = "/assets/uploads/attachments/";
            let max_bytes = 10 * 1024 * 1024; // 10 MB limit for chat
            let max_dimension = 1280;

            let processed = crate::services::media::process_and_save_upload(
                field,
                &original_name,
                target_dir,
                url_prefix,
                max_bytes,
                max_dimension,
                &state.minio_client,
            )
            .await?;

            file_name = processed.file_name;
            file_url = processed.file_url;
            file_type = processed.mime_type;
            file_size = processed.file_size as i32;
        }
    }

    if file_url.is_empty() {
        return Err(AppError::BadRequest(
            "Attachment file is required".to_string(),
        ));
    }

    Ok(Json(json!({
        "status": "success",
        "file_name": file_name,
        "file_url": file_url,
        "file_type": file_type,
        "file_size": file_size,
        "data": {
            "fileName": file_name,
            "fileUrl": file_url,
            "fileType": file_type,
            "fileSize": file_size
        }
    })))
}

#[derive(serde::Deserialize)]
pub struct UpdateConversationInput {
    pub status: String,
}

pub(crate) async fn update_conversation(
    _auth: RequireAuth,
    mut rls_tx: RlsTx,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateConversationInput>,
) -> Result<Json<Value>, AppError> {
    if payload.status != "active" && payload.status != "archived" && payload.status != "closed" {
        return Err(AppError::BadRequest(
            "Invalid conversation status".to_string(),
        ));
    }

    let rows_updated =
        sqlx::query("UPDATE conversations SET status = $1, updated_at = NOW() WHERE id = $2")
            .bind(&payload.status)
            .bind(id)
            .execute(&mut *rls_tx.tx)
            .await?
            .rows_affected();

    if rows_updated == 0 {
        return Err(AppError::NotFound(
            "Conversation not found or unauthorized".to_string(),
        ));
    }

    rls_tx.tx.commit().await?;

    Ok(Json(json!({
        "status": "success",
        "message": "Conversation updated successfully"
    })))
}

pub(crate) async fn moderate_message(
    auth: RequireAuth,
    mut rls_tx: RlsTx,
    Path(message_id): Path<Uuid>,
) -> Result<Json<Value>, AppError> {
    if auth.role != crate::models::user::DomainType::Admin {
        return Err(AppError::Forbidden(
            "Only administrators can moderate messages".to_string(),
        ));
    }

    let rows_updated = sqlx::query(
        "UPDATE messages SET body = '[This message has been redacted by the moderator due to community guidelines violations]', updated_at = NOW() WHERE id = $1"
    )
    .bind(message_id)
    .execute(&mut *rls_tx.tx)
    .await?
    .rows_affected();

    if rows_updated == 0 {
        return Err(AppError::NotFound("Message not found".to_string()));
    }

    rls_tx.tx.commit().await?;

    Ok(Json(json!({
        "status": "success",
        "message": "Message successfully moderated and redacted"
    })))
}

#[derive(serde::Deserialize)]
pub struct WsQuery {
    pub token: String,
}

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    headers: axum::http::HeaderMap,
    query: Option<Query<WsQuery>>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let mut token = None;

    // 1. Primary: Extract from Cookie header (identical to REST middleware)
    // We pass a dummy admin path to force it to look for `zafaf_admin_session` first.
    if let Some(t) = crate::middleware::auth::extract_session_token(&headers, "/api/v1/admin/") {
        token = Some(t);
    }

    // 2. Fallback: Query string ?token=... (keep for backward compatibility)
    if token.is_none() {
        if let Some(Query(q)) = query {
            token = Some(q.token);
        }
    }

    ws.on_upgrade(move |mut socket| async move {
        let token_str = match token {
            Some(t) => t,
            None => {
                let _ = socket.send(axum::extract::ws::Message::Close(Some(axum::extract::ws::CloseFrame {
                    code: 4401,
                    reason: std::borrow::Cow::Borrowed("Unauthorized: Missing token"),
                }))).await;
                return;
            }
        };

        match crate::middleware::auth::verify_token(&token_str, &state.jwt_secret, &state.db).await {
            Ok(claims) => {
                if claims.role != crate::models::user::DomainType::Admin {
                    let _ = socket.send(axum::extract::ws::Message::Close(Some(axum::extract::ws::CloseFrame {
                        code: 4403,
                        reason: std::borrow::Cow::Borrowed("Forbidden: Requires Admin role"),
                    }))).await;
                    return;
                }

                match Uuid::parse_str(&claims.sub) {
                    Ok(user_uuid) => handle_socket(socket, user_uuid, state).await,
                    Err(_) => {
                        let _ = socket.send(axum::extract::ws::Message::Close(Some(axum::extract::ws::CloseFrame {
                            code: 4401,
                            reason: std::borrow::Cow::Borrowed("Unauthorized: Invalid user ID"),
                        }))).await;
                    }
                }
            },
            Err(err) => {
                tracing::warn!("WebSocket auth rejection: {:?}", err);
                let _ = socket.send(axum::extract::ws::Message::Close(Some(axum::extract::ws::CloseFrame {
                    code: 4401,
                    reason: std::borrow::Cow::Borrowed("Unauthorized: Invalid token signature"),
                }))).await;
            }
        }
    })
}

async fn handle_ws_read_receipt(
    user_id: Uuid,
    message_id: Uuid,
    state: &AppState,
) -> Result<(), AppError> {
    let mut tx = state
        .db
        .begin()
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;

    // Set config for RLS (required because message tables are guarded by RLS policies!)
    sqlx::query("SELECT set_config('app.current_user_id', $1, true)")
        .bind(user_id.to_string())
        .execute(&mut *tx)
        .await?;

    // Get conversation_id
    let conversation_id: Option<Uuid> =
        sqlx::query_scalar("SELECT conversation_id FROM messages WHERE id = $1")
            .bind(message_id)
            .fetch_optional(&mut *tx)
            .await?;

    let conversation_id = match conversation_id {
        Some(id) => id,
        None => return Ok(()),
    };

    // Mark preceding messages in the conversation as read for the current user
    sqlx::query(
        "INSERT INTO message_read_receipts (message_id, user_id)
         SELECT m.id, $2
         FROM messages m
         WHERE m.conversation_id = $1
           AND m.created_at <= (SELECT created_at FROM messages WHERE id = $3)
           AND m.sender_id != $2
         ON CONFLICT (message_id, user_id) DO NOTHING",
    )
    .bind(conversation_id)
    .bind(user_id)
    .bind(message_id)
    .execute(&mut *tx)
    .await?;

    // Fetch participants of the conversation
    let participants: Vec<Uuid> = sqlx::query_scalar(
        "SELECT user_id FROM conversation_participants WHERE conversation_id = $1",
    )
    .bind(conversation_id)
    .fetch_all(&mut *tx)
    .await?;

    tx.commit()
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;

    // Broadcast onto event bus (updates active WebSockets and SSE streams)
    let _ = state
        .chat_event_tx
        .send(crate::state::ChatEvent::ReadReceipt {
            conversation_id,
            message_id,
            user_id,
            participant_ids: participants,
            read_at: chrono::Utc::now(),
        });

    Ok(())
}

async fn handle_socket(socket: WebSocket, user_id: Uuid, state: AppState) {
    let (mut sender, mut receiver) = socket.split();
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    let conn_id = Uuid::new_v4();

    state
        .ws_manager
        .register(user_id, crate::state::WsConn { id: conn_id, tx });

    let mut send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if sender.send(msg).await.is_err() {
                break;
            }
        }
    });

    let ws_manager_clone = state.ws_manager.clone();
    let state_clone = state.clone();
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            match msg {
                Message::Ping(ping) => {
                    ws_manager_clone.broadcast_to_user(user_id, Message::Pong(ping));
                }
                Message::Text(text) => {
                    tracing::info!("Received WS text message from user {}: {}", user_id, text);
                    if let Ok(value) = serde_json::from_str::<serde_json::Value>(&text) {
                        if value.get("type").and_then(|t| t.as_str()) == Some("READ_RECEIPT") {
                            if let Some(msg_id_str) =
                                value.get("messageId").and_then(|m| m.as_str())
                            {
                                if let Ok(message_id) = Uuid::parse_str(msg_id_str) {
                                    if let Err(err) =
                                        handle_ws_read_receipt(user_id, message_id, &state_clone)
                                            .await
                                    {
                                        tracing::error!(
                                            "Error handling WS read receipt: {:?}",
                                            err
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    });

    tokio::select! {
        _ = (&mut send_task) => {
            recv_task.abort();
        }
        _ = (&mut recv_task) => {
            send_task.abort();
        }
    }

    state.ws_manager.deregister(user_id, conn_id);
}

pub fn router(_state: AppState) -> Router<AppState> {
    Router::new()
        .route(
            "/conversations",
            get(list_conversations).post(create_conversation),
        )
        .route(
            "/conversations/:id",
            get(get_conversation).patch(update_conversation),
        )
        .route(
            "/conversations/:id/messages",
            get(list_messages).post(send_message),
        )
        .route("/messages/:id/read", patch(mark_message_read))
        .route("/attachments/upload", post(upload_attachment))
        .route("/messages/:id", delete(moderate_message))
        .route("/conversations/ws", get(ws_handler))
}
