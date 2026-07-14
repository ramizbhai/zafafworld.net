use crate::errors::AppError;
use crate::middleware::auth::{RequireAdmin, RequireSuperAdmin, RlsTx};
use crate::state::AppState;
use axum::{
    extract::{Path, Query, State},
    routing::{get, patch, post, delete},
    Json, Router,
};
use serde_json::{json, Value};
use uuid::Uuid;
use sqlx::Row;
use serde::Deserialize;

pub fn router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/assistant/inquiries", get(list_assistant_inquiries))
        .route(
            "/assistant/inquiries/:id/status",
            patch(update_assistant_inquiry_status),
        )
        .route("/afrah/inquiries", get(list_afrah_inquiries))
        .route(
            "/afrah/inquiries/:id/status",
            patch(update_afrah_inquiry_status),
        )
        .route("/inquiries", get(list_all_inquiries))
        .route("/inquiries/metrics", get(get_inquiries_metrics))
        .route("/inquiries/:id", get(get_inquiry_detail))
        .route("/inquiries/:id/status", patch(update_inquiry_status))
        .route("/inquiries/:id/notes", post(create_inquiry_note))
        .route(
            "/inquiries/:id/notes/:note_id",
            patch(update_inquiry_note).delete(delete_inquiry_note),
        )
        .route(
            "/inquiries/:id/management",
            patch(update_inquiry_management),
        )
}

async fn list_assistant_inquiries(
    mut rls_tx: RlsTx,
    _auth: RequireAdmin,
) -> Result<Json<Value>, AppError> {
    tracing::info!("Querying database for all assistant inquiries...");
    let rows = sqlx::query(
        "SELECT 
            ai.id,
            ai.client_id,
            ai.message,
            ai.status,
            ai.created_at,
            gu.email AS client_email
         FROM assistant_inquiries ai
         LEFT JOIN global_users gu ON ai.client_id = gu.id
         ORDER BY ai.created_at DESC",
    )
    .fetch_all(&mut *rls_tx.tx)
    .await?;

    let mut inquiries = Vec::new();
    for row in rows {
        let id: Uuid = row.get("id");
        let client_id: Uuid = row.get("client_id");
        let message: String = row.get("message");
        let status: String = row.get("status");
        let created_at: chrono::DateTime<chrono::Utc> = row.get("created_at");
        let client_email: Option<String> = row.get("client_email");

        inquiries.push(json!({
            "id": id.to_string(),
            "client_id": client_id.to_string(),
            "client_email": client_email.unwrap_or_else(|| "unknown@zafaf.net".to_string()),
            "message": message,
            "status": status,
            "created_at": created_at.to_rfc3339(),
        }));
    }

    rls_tx.tx.commit().await?;

    Ok(Json(json!({
        "status": "success",
        "inquiries": inquiries
    })))
}

#[derive(serde::Deserialize)]
pub struct UpdateAssistantInquiryStatusInput {
    pub status: String,
}

async fn update_assistant_inquiry_status(
    mut rls_tx: RlsTx,
    _auth: RequireAdmin,
    Path(id): Path<String>,
    Json(input): Json<UpdateAssistantInquiryStatusInput>,
) -> Result<Json<Value>, AppError> {
    tracing::info!(
        "Updating assistant inquiry {} status to: {}",
        id,
        input.status
    );

    let inquiry_uuid = Uuid::parse_str(&id)
        .map_err(|_| AppError::BadRequest("Invalid inquiry UUID format".to_string()))?;

    if input.status != "pending" && input.status != "resolved" {
        return Err(AppError::BadRequest(
            "Status must be either 'pending' or 'resolved'".to_string(),
        ));
    }

    sqlx::query(
        "UPDATE assistant_inquiries SET status = $1, updated_at = CURRENT_TIMESTAMP WHERE id = $2",
    )
    .bind(&input.status)
    .bind(inquiry_uuid)
    .execute(&mut *rls_tx.tx)
    .await?;

    rls_tx.tx.commit().await?;

    Ok(Json(json!({
        "status": "success",
        "message": "Assistant inquiry status updated successfully"
    })))
}

// ─── ADMIN AFRAH INQUIRIES ───────────────────────────────────────────────────

async fn list_afrah_inquiries(
    State(state): State<AppState>,
    _auth: RequireAdmin,
) -> Result<Json<Value>, AppError> {
    let rows = sqlx::query(
        "SELECT id, name, phone, is_whatsapp, event_date, message, email, status, ip_address, created_at, updated_at
         FROM afrah_inquiries
         ORDER BY created_at DESC",
    )
    .fetch_all(&state.db)
    .await?;

    let mut inquiries = Vec::new();
    for row in rows {
        let id: Uuid = row.get("id");
        let name: String = row.get("name");
        let phone: String = row.get::<Option<String>, _>("phone").unwrap_or_default();
        let is_whatsapp: bool = row.get("is_whatsapp");
        let event_date: chrono::NaiveDate = row.get("event_date");
        let message: String = row.get("message");
        let email: Option<String> = row.get("email");
        let status: String = row.get("status");
        let ip_address: Option<String> = row.get("ip_address");
        let created_at: chrono::DateTime<chrono::Utc> = row.get("created_at");
        let updated_at: chrono::DateTime<chrono::Utc> = row.get("updated_at");

        inquiries.push(json!({
            "id": id.to_string(),
            "name": name,
            "phone": phone,
            "isWhatsapp": is_whatsapp,
            "eventDate": event_date.to_string(),
            "message": message,
            "email": email.unwrap_or_default(),
            "status": status,
            "ipAddress": ip_address.unwrap_or_default(),
            "createdAt": created_at.to_rfc3339(),
            "updatedAt": updated_at.to_rfc3339(),
        }));
    }

    Ok(Json(json!({
        "status": "success",
        "inquiries": inquiries
    })))
}

#[derive(serde::Deserialize)]
pub struct UpdateAfrahInquiryStatusInput {
    pub status: String,
}

async fn update_afrah_inquiry_status(
    State(state): State<AppState>,
    _auth: RequireAdmin,
    Path(id): Path<String>,
    Json(input): Json<UpdateAfrahInquiryStatusInput>,
) -> Result<Json<Value>, AppError> {
    let inquiry_uuid = Uuid::parse_str(&id)
        .map_err(|_| AppError::BadRequest("Invalid inquiry UUID format".to_string()))?;

    if !matches!(input.status.as_str(), "pending" | "contacted" | "resolved") {
        return Err(AppError::BadRequest(
            "Status must be 'pending', 'contacted', or 'resolved'".to_string(),
        ));
    }

    sqlx::query(
        "UPDATE afrah_inquiries SET status = $1, updated_at = CURRENT_TIMESTAMP WHERE id = $2",
    )
    .bind(&input.status)
    .bind(inquiry_uuid)
    .execute(&state.db)
    .await?;

    Ok(Json(json!({
        "status": "success",
        "message": "Afrah inquiry status updated successfully"
    })))
}

#[derive(Deserialize)]
pub struct AdminInquiriesQuery {
    pub page: Option<i64>,
    pub limit: Option<i64>,
    pub q: Option<String>,
    pub status: Option<String>,
    pub vendor_id: Option<Uuid>,
    pub product_id: Option<Uuid>,
    pub inquiry_type: Option<String>,
    pub city_id: Option<Uuid>,
    pub event_date: Option<String>,
    pub date_from: Option<String>,
    pub date_to: Option<String>,
    pub priority: Option<String>,
    pub escalation_status: Option<String>,
    pub resolution_status: Option<String>,
    pub sort_by: Option<String>,
    pub order: Option<String>,
}

/// GET /api/v1/admin/inquiries
async fn list_all_inquiries(
    State(state): State<AppState>,
    _auth: RequireAdmin,
    axum::extract::Query(query): axum::extract::Query<AdminInquiriesQuery>,
) -> Result<Json<Value>, AppError> {
    let page = query.page.unwrap_or(1).max(1);
    let limit = query.limit.unwrap_or(20).clamp(1, 100);
    let offset = (page - 1) * limit;

    let mut where_clauses: Vec<String> = Vec::new();
    let mut param_idx = 1;

    enum BindVal {
        String(String),
        Uuid(Uuid),
        Date(chrono::NaiveDate),
    }
    let mut binds: Vec<BindVal> = Vec::new();

    if let Some(ref q) = query.q {
        let clean_q = q.trim();
        if !clean_q.is_empty() {
            let search_pattern = format!("%{}%", clean_q);
            if let Ok(u) = Uuid::parse_str(clean_q) {
                where_clauses.push(format!("(vi.id = ${0} OR vi.name ILIKE ${1} OR vi.phone ILIKE ${1} OR vi.email ILIKE ${1} OR v.name_en ILIKE ${1} OR v.name_ar ILIKE ${1} OR vp.title_en ILIKE ${1} OR vp.title_ar ILIKE ${1})", param_idx, param_idx + 1));
                binds.push(BindVal::Uuid(u));
                binds.push(BindVal::String(search_pattern));
                param_idx += 2;
            } else {
                where_clauses.push(format!("(vi.name ILIKE ${0} OR vi.phone ILIKE ${0} OR vi.email ILIKE ${0} OR v.name_en ILIKE ${0} OR v.name_ar ILIKE ${0} OR vp.title_en ILIKE ${0} OR vp.title_ar ILIKE ${0})", param_idx));
                binds.push(BindVal::String(search_pattern));
                param_idx += 1;
            }
        }
    }

    if let Some(ref st) = query.status {
        if !st.trim().is_empty() {
            where_clauses.push(format!("vi.status = ${}", param_idx));
            binds.push(BindVal::String(st.trim().to_string()));
            param_idx += 1;
        }
    }

    if let Some(vid) = query.vendor_id {
        where_clauses.push(format!("vi.vendor_id = ${}", param_idx));
        binds.push(BindVal::Uuid(vid));
        param_idx += 1;
    }

    if let Some(pid) = query.product_id {
        where_clauses.push(format!("vi.product_id = ${}", param_idx));
        binds.push(BindVal::Uuid(pid));
        param_idx += 1;
    }

    if let Some(cid) = query.city_id {
        where_clauses.push(format!("vi.city_id = ${}", param_idx));
        binds.push(BindVal::Uuid(cid));
        param_idx += 1;
    }

    if let Some(ref inq_type) = query.inquiry_type {
        if inq_type == "guest" {
            where_clauses.push("vi.client_id IS NULL".to_string());
        } else if inq_type == "auth" {
            where_clauses.push("vi.client_id IS NOT NULL".to_string());
        }
    }

    if let Some(ref ed) = query.event_date {
        if let Ok(d) = chrono::NaiveDate::parse_from_str(ed, "%Y-%m-%d") {
            where_clauses.push(format!("vi.event_date = ${}", param_idx));
            binds.push(BindVal::Date(d));
            param_idx += 1;
        }
    }

    if let Some(ref df) = query.date_from {
        if let Ok(d) = chrono::NaiveDate::parse_from_str(df, "%Y-%m-%d") {
            where_clauses.push(format!("vi.created_at >= ${}", param_idx));
            binds.push(BindVal::Date(d));
            param_idx += 1;
        }
    }

    if let Some(ref dt) = query.date_to {
        if let Ok(d) = chrono::NaiveDate::parse_from_str(dt, "%Y-%m-%d") {
            where_clauses.push(format!("vi.created_at <= ${}", param_idx));
            binds.push(BindVal::Date(d));
            param_idx += 1;
        }
    }

    if let Some(ref pr) = query.priority {
        if !pr.trim().is_empty() {
            where_clauses.push(format!("COALESCE(vim.priority, 'medium') = ${}", param_idx));
            binds.push(BindVal::String(pr.trim().to_string()));
            param_idx += 1;
        }
    }

    if let Some(ref esc) = query.escalation_status {
        if !esc.trim().is_empty() {
            where_clauses.push(format!(
                "COALESCE(vim.escalation_status, 'none') = ${}",
                param_idx
            ));
            binds.push(BindVal::String(esc.trim().to_string()));
            param_idx += 1;
        }
    }

    if let Some(ref res) = query.resolution_status {
        if !res.trim().is_empty() {
            where_clauses.push(format!(
                "COALESCE(vim.resolution_status, 'unresolved') = ${}",
                param_idx
            ));
            binds.push(BindVal::String(res.trim().to_string()));
        }
    }

    let where_sql = if where_clauses.is_empty() {
        "".to_string()
    } else {
        format!("WHERE {}", where_clauses.join(" AND "))
    };

    let sort_col = match query.sort_by.as_deref() {
        Some("event_date") => "vi.event_date",
        Some("status") => "vi.status",
        Some("priority") => "COALESCE(vim.priority, 'medium')",
        _ => "vi.created_at",
    };
    let sort_ord = match query.order.as_deref() {
        Some("asc") | Some("ASC") => "ASC",
        _ => "DESC",
    };

    let count_sql = format!(
        "SELECT COUNT(*)::bigint FROM vendor_inquiries vi
         JOIN vendors v ON vi.vendor_id = v.id
         LEFT JOIN vendor_products vp ON vi.product_id = vp.id
         LEFT JOIN global_users gu ON vi.client_id = gu.id
         LEFT JOIN cities c ON vi.city_id = c.id
         LEFT JOIN vendor_inquiry_management vim ON vi.id = vim.inquiry_id {}",
        where_sql
    );

    let select_sql = format!(
        "SELECT vi.id, vi.client_id, vi.vendor_id, vi.product_id, vi.event_date, vi.guest_count, vi.message, vi.status, vi.name, vi.phone, vi.email, vi.city_id, vi.created_at, vi.updated_at,
                v.name_en as vendor_name_en, v.name_ar as vendor_name_ar,
                vp.title_en as listing_title_en, vp.title_ar as listing_title_ar,
                c.name_en as city_name_en, c.name_ar as city_name_ar,
                gu.first_name as client_first_name, gu.last_name as client_last_name, gu.email as client_user_email, gu.phone as client_user_phone,
                COALESCE(vim.priority, 'medium') as priority,
                COALESCE(vim.escalation_status, 'none') as escalation_status,
                COALESCE(vim.resolution_status, 'unresolved') as resolution_status,
                vim.assigned_admin_id,
                admin_u.first_name as admin_first_name, admin_u.last_name as admin_last_name
         FROM vendor_inquiries vi
         JOIN vendors v ON vi.vendor_id = v.id
         LEFT JOIN vendor_products vp ON vi.product_id = vp.id
         LEFT JOIN global_users gu ON vi.client_id = gu.id
         LEFT JOIN cities c ON vi.city_id = c.id
         LEFT JOIN vendor_inquiry_management vim ON vi.id = vim.inquiry_id
         LEFT JOIN global_users admin_u ON vim.assigned_admin_id = admin_u.id
         {} ORDER BY {} {} LIMIT {} OFFSET {}",
        where_sql, sort_col, sort_ord, limit, offset
    );

    let mut count_q = sqlx::query_scalar::<_, i64>(&count_sql);
    for b in &binds {
        match b {
            BindVal::String(s) => count_q = count_q.bind(s),
            BindVal::Uuid(u) => count_q = count_q.bind(u),
            BindVal::Date(d) => count_q = count_q.bind(d),
        }
    }
    let total_count = count_q.fetch_one(&state.db).await?;

    let mut select_q = sqlx::query(&select_sql);
    for b in &binds {
        match b {
            BindVal::String(s) => select_q = select_q.bind(s),
            BindVal::Uuid(u) => select_q = select_q.bind(u),
            BindVal::Date(d) => select_q = select_q.bind(d),
        }
    }
    let rows = select_q.fetch_all(&state.db).await?;

    let items: Vec<Value> = rows
        .into_iter()
        .map(|row| {
            let id: Uuid = row.get("id");
            let client_id: Option<Uuid> = row.get("client_id");
            let vendor_id: Uuid = row.get("vendor_id");
            let product_id: Option<Uuid> = row.get("product_id");
            let event_date: chrono::NaiveDate = row.get("event_date");
            let guest_count: i32 = row.get("guest_count");
            let message: String = row.get("message");
            let status: String = row.get("status");
            let name: Option<String> = row.get("name");
            let phone: Option<String> = row.get("phone");
            let email: Option<String> = row.get("email");
            let city_id: Option<Uuid> = row.get("city_id");
            let created_at: chrono::DateTime<chrono::Utc> = row.get("created_at");
            let updated_at: chrono::DateTime<chrono::Utc> = row.get("updated_at");

            let vendor_name_en: String = row.get("vendor_name_en");
            let vendor_name_ar: String = row.get("vendor_name_ar");
            let listing_title_en: Option<String> = row.get("listing_title_en");
            let listing_title_ar: Option<String> = row.get("listing_title_ar");
            let city_name_en: Option<String> = row.get("city_name_en");
            let city_name_ar: Option<String> = row.get("city_name_ar");

            let client_first: Option<String> = row.get("client_first_name");
            let client_last: Option<String> = row.get("client_last_name");
            let client_user_email: Option<String> = row.get("client_user_email");
            let client_user_phone: Option<String> = row.get("client_user_phone");

            let priority: String = row.get("priority");
            let escalation_status: String = row.get("escalation_status");
            let resolution_status: String = row.get("resolution_status");
            let assigned_admin_id: Option<Uuid> = row.get("assigned_admin_id");
            let admin_first: Option<String> = row.get("admin_first_name");
            let admin_last: Option<String> = row.get("admin_last_name");

            let display_client_name = name
                .clone()
                .or_else(|| match (&client_first, &client_last) {
                    (Some(f), Some(l)) => Some(format!("{} {}", f, l)),
                    (Some(f), None) => Some(f.clone()),
                    _ => None,
                })
                .unwrap_or_else(|| "Guest User".to_string());

            let display_phone = phone.clone().or(client_user_phone).unwrap_or_default();
            let display_email = email.clone().or(client_user_email).unwrap_or_default();
            let assigned_admin_name = match (&admin_first, &admin_last) {
                (Some(f), Some(l)) => Some(format!("{} {}", f, l)),
                (Some(f), None) => Some(f.clone()),
                _ => None,
            };

            json!({
                "id": id.to_string(),
                "status": status,
                "eventDate": event_date.to_string(),
                "guestCount": guest_count,
                "message": message,
                "createdAt": created_at,
                "updatedAt": updated_at,
                "isGuest": client_id.is_none(),
                "client": {
                    "id": client_id.map(|u| u.to_string()),
                    "name": display_client_name,
                    "phone": display_phone,
                    "email": display_email
                },
                "vendor": {
                    "id": vendor_id.to_string(),
                    "nameEn": vendor_name_en,
                    "nameAr": vendor_name_ar
                },
                "listing": product_id.map(|pid| json!({
                    "id": pid.to_string(),
                    "titleEn": listing_title_en.unwrap_or_default(),
                    "titleAr": listing_title_ar.unwrap_or_default()
                })),
                "city": city_id.map(|cid| json!({
                    "id": cid.to_string(),
                    "nameEn": city_name_en.unwrap_or_default(),
                    "nameAr": city_name_ar.unwrap_or_default()
                })),
                "management": {
                    "priority": priority,
                    "escalationStatus": escalation_status,
                    "resolutionStatus": resolution_status,
                    "assignedAdminId": assigned_admin_id.map(|u| u.to_string()),
                    "assignedAdminName": assigned_admin_name
                }
            })
        })
        .collect();

    let total_pages = (total_count as f64 / limit as f64).ceil() as i64;

    Ok(Json(json!({
        "status": "success",
        "inquiries": items,
        "pagination": {
            "page": page,
            "limit": limit,
            "totalItems": total_count,
            "totalPages": total_pages
        }
    })))
}

/// GET /api/v1/admin/inquiries/metrics
async fn get_inquiries_metrics(
    State(state): State<AppState>,
    _auth: RequireAdmin,
) -> Result<Json<Value>, AppError> {
    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM vendor_inquiries")
        .fetch_one(&state.db)
        .await?;

    let unread: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM vendor_inquiries WHERE status = 'unread'")
            .fetch_one(&state.db)
            .await?;

    let waiting_vendor: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM vendor_inquiries WHERE status IN ('unread', 'viewed', 'pending')",
    )
    .fetch_one(&state.db)
    .await?;

    let escalated: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM vendor_inquiry_management WHERE escalation_status = 'escalated'",
    )
    .fetch_one(&state.db)
    .await?;

    let resolved_today: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM vendor_inquiry_management WHERE resolution_status = 'resolved' AND resolved_at >= CURRENT_DATE")
        .fetch_one(&state.db)
        .await?;

    let high_priority: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM vendor_inquiry_management WHERE priority IN ('high', 'critical')",
    )
    .fetch_one(&state.db)
    .await?;

    Ok(Json(json!({
        "status": "success",
        "metrics": {
            "total": total,
            "unread": unread,
            "waitingVendor": waiting_vendor,
            "escalated": escalated,
            "resolvedToday": resolved_today,
            "highPriority": high_priority
        }
    })))
}

/// GET /api/v1/admin/inquiries/:id
async fn get_inquiry_detail(
    State(state): State<AppState>,
    _auth: RequireAdmin,
    Path(id): Path<String>,
) -> Result<Json<Value>, AppError> {
    let inq_uuid =
        Uuid::parse_str(&id).map_err(|_| AppError::BadRequest("Invalid inquiry ID".to_string()))?;

    let row = sqlx::query(
        "SELECT vi.id, vi.client_id, vi.vendor_id, vi.product_id, vi.event_date, vi.guest_count, vi.message, vi.status, vi.name, vi.phone, vi.email, vi.conversation_id, vi.city_id, vi.created_at, vi.updated_at,
                v.name_en as vendor_name_en, v.name_ar as vendor_name_ar, v.email as vendor_email, v.phone as vendor_phone,
                vp.title_en as listing_title_en, vp.title_ar as listing_title_ar,
                c.name_en as city_name_en, c.name_ar as city_name_ar,
                gu.first_name as client_first_name, gu.last_name as client_last_name, gu.email as client_user_email, gu.phone as client_user_phone,
                COALESCE(vim.priority, 'medium') as priority,
                COALESCE(vim.escalation_status, 'none') as escalation_status,
                COALESCE(vim.resolution_status, 'unresolved') as resolution_status,
                vim.assigned_admin_id, vim.assigned_at, vim.escalated_at, vim.resolved_at,
                admin_u.first_name as admin_first_name, admin_u.last_name as admin_last_name
         FROM vendor_inquiries vi
         JOIN vendors v ON vi.vendor_id = v.id
         LEFT JOIN vendor_products vp ON vi.product_id = vp.id
         LEFT JOIN global_users gu ON vi.client_id = gu.id
         LEFT JOIN cities c ON vi.city_id = c.id
         LEFT JOIN vendor_inquiry_management vim ON vi.id = vim.inquiry_id
         LEFT JOIN global_users admin_u ON vim.assigned_admin_id = admin_u.id
         WHERE vi.id = $1"
    )
    .bind(inq_uuid)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Inquiry not found".to_string()))?;

    let conversation_id: Option<Uuid> = row.get("conversation_id");

    let messages: Vec<Value> = if let Some(cid) = conversation_id {
        let msg_rows = sqlx::query(
            "SELECT m.id, m.sender_id, m.body, m.created_at, gu.first_name, gu.last_name, gu.domain_type
             FROM messages m
             LEFT JOIN global_users gu ON m.sender_id = gu.id
             WHERE m.conversation_id = $1
             ORDER BY m.created_at ASC"
        )
        .bind(cid)
        .fetch_all(&state.db)
        .await?;

        msg_rows
            .into_iter()
            .map(|mr| {
                let mid: Uuid = mr.get("id");
                let sender_id: Uuid = mr.get("sender_id");
                let body: String = mr.get("body");
                let created_at: chrono::DateTime<chrono::Utc> = mr.get("created_at");
                let fn_opt: Option<String> = mr.get("first_name");
                let ln_opt: Option<String> = mr.get("last_name");
                let domain: Option<String> = mr.get("domain_type");
                let sender_name = match (fn_opt, ln_opt) {
                    (Some(f), Some(l)) => format!("{} {}", f, l),
                    (Some(f), None) => f,
                    _ => "User".to_string(),
                };
                json!({
                    "id": mid.to_string(),
                    "senderId": sender_id.to_string(),
                    "senderName": sender_name,
                    "senderRole": domain.unwrap_or_else(|| "client".to_string()),
                    "body": body,
                    "createdAt": created_at
                })
            })
            .collect()
    } else {
        vec![]
    };

    let note_rows = sqlx::query(
        "SELECT n.id, n.admin_id, n.note, n.note_type, n.is_internal, n.created_at, n.updated_at,
                gu.first_name, gu.last_name
         FROM vendor_inquiry_admin_notes n
         JOIN global_users gu ON n.admin_id = gu.id
         WHERE n.inquiry_id = $1
         ORDER BY n.created_at ASC",
    )
    .bind(inq_uuid)
    .fetch_all(&state.db)
    .await?;

    let admin_notes: Vec<Value> = note_rows
        .into_iter()
        .map(|nr| {
            let nid: Uuid = nr.get("id");
            let admin_id: Uuid = nr.get("admin_id");
            let note: String = nr.get("note");
            let note_type: String = nr.get("note_type");
            let is_internal: bool = nr.get("is_internal");
            let created_at: chrono::DateTime<chrono::Utc> = nr.get("created_at");
            let updated_at: chrono::DateTime<chrono::Utc> = nr.get("updated_at");
            let fn_opt: Option<String> = nr.get("first_name");
            let ln_opt: Option<String> = nr.get("last_name");
            let admin_name = match (fn_opt, ln_opt) {
                (Some(f), Some(l)) => format!("{} {}", f, l),
                (Some(f), None) => f,
                _ => "Admin".to_string(),
            };
            json!({
                "id": nid.to_string(),
                "adminId": admin_id.to_string(),
                "adminName": admin_name,
                "note": note,
                "noteType": note_type,
                "isInternal": is_internal,
                "createdAt": created_at,
                "updatedAt": updated_at
            })
        })
        .collect();

    let client_id: Option<Uuid> = row.get("client_id");
    let vendor_id: Uuid = row.get("vendor_id");
    let product_id: Option<Uuid> = row.get("product_id");
    let event_date: chrono::NaiveDate = row.get("event_date");
    let guest_count: i32 = row.get("guest_count");
    let message: String = row.get("message");
    let status: String = row.get("status");
    let name: Option<String> = row.get("name");
    let phone: Option<String> = row.get("phone");
    let email: Option<String> = row.get("email");
    let city_id: Option<Uuid> = row.get("city_id");
    let created_at: chrono::DateTime<chrono::Utc> = row.get("created_at");
    let updated_at: chrono::DateTime<chrono::Utc> = row.get("updated_at");

    let vendor_name_en: String = row.get("vendor_name_en");
    let vendor_name_ar: String = row.get("vendor_name_ar");
    let vendor_email: Option<String> = row.get("vendor_email");
    let vendor_phone: Option<String> = row.get("vendor_phone");

    let listing_title_en: Option<String> = row.get("listing_title_en");
    let listing_title_ar: Option<String> = row.get("listing_title_ar");
    let city_name_en: Option<String> = row.get("city_name_en");
    let city_name_ar: Option<String> = row.get("city_name_ar");

    let client_first: Option<String> = row.get("client_first_name");
    let client_last: Option<String> = row.get("client_last_name");
    let client_user_email: Option<String> = row.get("client_user_email");
    let client_user_phone: Option<String> = row.get("client_user_phone");

    let priority: String = row.get("priority");
    let escalation_status: String = row.get("escalation_status");
    let resolution_status: String = row.get("resolution_status");
    let assigned_admin_id: Option<Uuid> = row.get("assigned_admin_id");
    let assigned_at: Option<chrono::DateTime<chrono::Utc>> = row.get("assigned_at");
    let escalated_at: Option<chrono::DateTime<chrono::Utc>> = row.get("escalated_at");
    let resolved_at: Option<chrono::DateTime<chrono::Utc>> = row.get("resolved_at");
    let admin_first: Option<String> = row.get("admin_first_name");
    let admin_last: Option<String> = row.get("admin_last_name");

    let display_client_name = name
        .clone()
        .or_else(|| match (&client_first, &client_last) {
            (Some(f), Some(l)) => Some(format!("{} {}", f, l)),
            (Some(f), None) => Some(f.clone()),
            _ => None,
        })
        .unwrap_or_else(|| "Guest User".to_string());

    let display_phone = phone.clone().or(client_user_phone).unwrap_or_default();
    let display_email = email.clone().or(client_user_email).unwrap_or_default();
    let assigned_admin_name = match (&admin_first, &admin_last) {
        (Some(f), Some(l)) => Some(format!("{} {}", f, l)),
        (Some(f), None) => Some(f.clone()),
        _ => None,
    };

    let admin_uuid = Uuid::parse_str(&_auth.user_id).unwrap_or_default();
    let _ = sqlx::query(
        "INSERT INTO system_events (user_id, target_vendor_id, event_type, message_ar, message_en)
         VALUES ($1, $2, 'admin_inquiry_viewed', $3, $4)",
    )
    .bind(admin_uuid)
    .bind(vendor_id)
    .bind(format!("قام المسؤول بتفحص تفاصيل الاستفسار «{}»", inq_uuid))
    .bind(format!("Admin viewed inquiry detail '{}'", inq_uuid))
    .execute(&state.db)
    .await;

    Ok(Json(json!({
        "status": "success",
        "inquiry": {
            "id": inq_uuid.to_string(),
            "status": status,
            "eventDate": event_date.to_string(),
            "guestCount": guest_count,
            "message": message,
            "createdAt": created_at,
            "updatedAt": updated_at,
            "conversationId": conversation_id.map(|u| u.to_string()),
            "isGuest": client_id.is_none()
        },
        "client": {
            "id": client_id.map(|u| u.to_string()),
            "name": display_client_name,
            "phone": display_phone,
            "email": display_email
        },
        "vendor": {
            "id": vendor_id.to_string(),
            "nameEn": vendor_name_en,
            "nameAr": vendor_name_ar,
            "email": vendor_email.unwrap_or_default(),
            "phone": vendor_phone.unwrap_or_default()
        },
        "listing": product_id.map(|pid| json!({
            "id": pid.to_string(),
            "titleEn": listing_title_en.unwrap_or_default(),
            "titleAr": listing_title_ar.unwrap_or_default()
        })),
        "city": city_id.map(|cid| json!({
            "id": cid.to_string(),
            "nameEn": city_name_en.unwrap_or_default(),
            "nameAr": city_name_ar.unwrap_or_default()
        })),
        "conversation": messages,
        "adminNotes": admin_notes,
        "management": {
            "priority": priority,
            "escalationStatus": escalation_status,
            "resolutionStatus": resolution_status,
            "assignedAdminId": assigned_admin_id.map(|u| u.to_string()),
            "assignedAdminName": assigned_admin_name,
            "assignedAt": assigned_at,
            "escalatedAt": escalated_at,
            "resolvedAt": resolved_at
        }
    })))
}

#[derive(Deserialize)]
pub struct UpdateInquiryStatusInput {
    pub status: String,
}

/// PATCH /api/v1/admin/inquiries/:id/status
async fn update_inquiry_status(
    State(state): State<AppState>,
    auth: RequireAdmin,
    Path(id): Path<String>,
    Json(input): Json<UpdateInquiryStatusInput>,
) -> Result<Json<Value>, AppError> {
    let inq_uuid =
        Uuid::parse_str(&id).map_err(|_| AppError::BadRequest("Invalid inquiry ID".to_string()))?;
    let admin_uuid = Uuid::parse_str(&auth.user_id).unwrap_or_default();

    let valid_statuses = [
        "unread", "viewed", "pending", "replied", "closed", "declined",
    ];
    if !valid_statuses.contains(&input.status.as_str()) {
        return Err(AppError::BadRequest("Invalid status value".to_string()));
    }

    let res =
        sqlx::query("UPDATE vendor_inquiries SET status = $1, updated_at = NOW() WHERE id = $2")
            .bind(&input.status)
            .bind(inq_uuid)
            .execute(&state.db)
            .await?;

    if res.rows_affected() == 0 {
        return Err(AppError::NotFound("Inquiry not found".to_string()));
    }

    let _ = sqlx::query(
        "INSERT INTO system_events (user_id, event_type, message_ar, message_en)
         VALUES ($1, 'admin_inquiry_status_updated', $2, $3)",
    )
    .bind(admin_uuid)
    .bind(format!(
        "تحديث حالة الاستفسار «{}» إلى {}",
        inq_uuid, input.status
    ))
    .bind(format!(
        "Admin updated inquiry '{}' status to {}",
        inq_uuid, input.status
    ))
    .execute(&state.db)
    .await;

    Ok(Json(json!({
        "status": "success",
        "message": "Inquiry status updated successfully"
    })))
}

#[derive(Deserialize)]
pub struct CreateInquiryNoteInput {
    pub note: String,
    pub note_type: Option<String>,
    pub is_internal: Option<bool>,
}

/// POST /api/v1/admin/inquiries/:id/notes
async fn create_inquiry_note(
    State(state): State<AppState>,
    auth: RequireAdmin,
    Path(id): Path<String>,
    Json(input): Json<CreateInquiryNoteInput>,
) -> Result<Json<Value>, AppError> {
    let inq_uuid =
        Uuid::parse_str(&id).map_err(|_| AppError::BadRequest("Invalid inquiry ID".to_string()))?;
    let admin_uuid = Uuid::parse_str(&auth.user_id).unwrap_or_default();

    if input.note.trim().is_empty() {
        return Err(AppError::BadRequest("Note cannot be empty".to_string()));
    }

    let note_type = input.note_type.unwrap_or_else(|| "internal".to_string());
    let is_internal = input.is_internal.unwrap_or(true);
    let note_id = Uuid::new_v4();

    sqlx::query(
        "INSERT INTO vendor_inquiry_admin_notes (id, inquiry_id, admin_id, note, note_type, is_internal)
         VALUES ($1, $2, $3, $4, $5, $6)"
    )
    .bind(note_id)
    .bind(inq_uuid)
    .bind(admin_uuid)
    .bind(input.note.trim())
    .bind(&note_type)
    .bind(is_internal)
    .execute(&state.db)
    .await?;

    let _ = sqlx::query(
        "INSERT INTO system_events (user_id, event_type, message_ar, message_en)
         VALUES ($1, 'admin_inquiry_note_added', $2, $3)",
    )
    .bind(admin_uuid)
    .bind(format!("إضافة ملاحظة إدارية للاستفسار «{}»", inq_uuid))
    .bind(format!("Admin added note to inquiry '{}'", inq_uuid))
    .execute(&state.db)
    .await;

    Ok(Json(json!({
        "status": "success",
        "message": "Admin note created successfully",
        "id": note_id.to_string()
    })))
}

#[derive(Deserialize)]
pub struct UpdateInquiryNoteInput {
    pub note: Option<String>,
    pub note_type: Option<String>,
    pub is_internal: Option<bool>,
}

/// PATCH /api/v1/admin/inquiries/:id/notes/:note_id
async fn update_inquiry_note(
    State(state): State<AppState>,
    auth: RequireAdmin,
    Path((id, note_id)): Path<(String, String)>,
    Json(input): Json<UpdateInquiryNoteInput>,
) -> Result<Json<Value>, AppError> {
    let inq_uuid =
        Uuid::parse_str(&id).map_err(|_| AppError::BadRequest("Invalid inquiry ID".to_string()))?;
    let note_uuid = Uuid::parse_str(&note_id)
        .map_err(|_| AppError::BadRequest("Invalid note ID".to_string()))?;
    let admin_uuid = Uuid::parse_str(&auth.user_id).unwrap_or_default();

    let res = sqlx::query(
        "UPDATE vendor_inquiry_admin_notes
         SET note = COALESCE($1, note),
             note_type = COALESCE($2, note_type),
             is_internal = COALESCE($3, is_internal),
             updated_at = NOW()
         WHERE id = $4 AND inquiry_id = $5",
    )
    .bind(input.note.as_deref().map(|s| s.trim()))
    .bind(input.note_type)
    .bind(input.is_internal)
    .bind(note_uuid)
    .bind(inq_uuid)
    .execute(&state.db)
    .await?;

    if res.rows_affected() == 0 {
        return Err(AppError::NotFound("Note not found".to_string()));
    }

    let _ = sqlx::query(
        "INSERT INTO system_events (user_id, event_type, message_ar, message_en)
         VALUES ($1, 'admin_inquiry_note_edited', $2, $3)",
    )
    .bind(admin_uuid)
    .bind(format!("تعديل ملاحظة إدارية للاستفسار «{}»", inq_uuid))
    .bind(format!("Admin edited note on inquiry '{}'", inq_uuid))
    .execute(&state.db)
    .await;

    Ok(Json(json!({
        "status": "success",
        "message": "Admin note updated successfully"
    })))
}

/// DELETE /api/v1/admin/inquiries/:id/notes/:note_id
async fn delete_inquiry_note(
    State(state): State<AppState>,
    auth: RequireAdmin,
    Path((id, note_id)): Path<(String, String)>,
) -> Result<Json<Value>, AppError> {
    let inq_uuid =
        Uuid::parse_str(&id).map_err(|_| AppError::BadRequest("Invalid inquiry ID".to_string()))?;
    let note_uuid = Uuid::parse_str(&note_id)
        .map_err(|_| AppError::BadRequest("Invalid note ID".to_string()))?;
    let admin_uuid = Uuid::parse_str(&auth.user_id).unwrap_or_default();

    let res =
        sqlx::query("DELETE FROM vendor_inquiry_admin_notes WHERE id = $1 AND inquiry_id = $2")
            .bind(note_uuid)
            .bind(inq_uuid)
            .execute(&state.db)
            .await?;

    if res.rows_affected() == 0 {
        return Err(AppError::NotFound("Note not found".to_string()));
    }

    let _ = sqlx::query(
        "INSERT INTO system_events (user_id, event_type, message_ar, message_en)
         VALUES ($1, 'admin_inquiry_note_deleted', $2, $3)",
    )
    .bind(admin_uuid)
    .bind(format!("حذف ملاحظة إدارية للاستفسار «{}»", inq_uuid))
    .bind(format!("Admin deleted note from inquiry '{}'", inq_uuid))
    .execute(&state.db)
    .await;

    Ok(Json(json!({
        "status": "success",
        "message": "Admin note deleted successfully"
    })))
}

#[derive(Deserialize)]
pub struct UpdateInquiryManagementInput {
    pub priority: Option<String>,
    pub escalation_status: Option<String>,
    pub resolution_status: Option<String>,
    pub assigned_admin_id: Option<Uuid>,
}

/// PATCH /api/v1/admin/inquiries/:id/management
#[axum::debug_handler]
async fn update_inquiry_management(
    State(state): State<AppState>,
    auth: RequireSuperAdmin,
    Path(id): Path<String>,
    Json(input): Json<UpdateInquiryManagementInput>,
) -> Result<Json<Value>, AppError> {
    let inq_uuid =
        Uuid::parse_str(&id).map_err(|_| AppError::BadRequest("Invalid inquiry ID".to_string()))?;
    let super_admin_uuid = Uuid::parse_str(&auth.user_id).unwrap_or_default();

    let exists: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM vendor_inquiries WHERE id = $1)")
            .bind(inq_uuid)
            .fetch_one(&state.db)
            .await?;

    if !exists {
        return Err(AppError::NotFound("Inquiry not found".to_string()));
    }

    let now = chrono::Utc::now();
    let assigned_at = if input.assigned_admin_id.is_some() {
        Some(now)
    } else {
        None
    };
    let escalated_at = if input.escalation_status.as_deref() == Some("escalated") {
        Some(now)
    } else {
        None
    };
    let resolved_at = if input.resolution_status.as_deref() == Some("resolved") {
        Some(now)
    } else {
        None
    };

    sqlx::query(
        "INSERT INTO vendor_inquiry_management (
            inquiry_id, assigned_admin_id, escalation_status, resolution_status, priority, assigned_at, escalated_at, resolved_at, updated_at
         ) VALUES ($1, $2, COALESCE($3, 'none'), COALESCE($4, 'unresolved'), COALESCE($5, 'medium'), $6, $7, $8, NOW())
         ON CONFLICT (inquiry_id) DO UPDATE SET
            assigned_admin_id = COALESCE(EXCLUDED.assigned_admin_id, vendor_inquiry_management.assigned_admin_id),
            escalation_status = COALESCE($3, vendor_inquiry_management.escalation_status),
            resolution_status = COALESCE($4, vendor_inquiry_management.resolution_status),
            priority = COALESCE($5, vendor_inquiry_management.priority),
            assigned_at = COALESCE(EXCLUDED.assigned_at, vendor_inquiry_management.assigned_at),
            escalated_at = COALESCE(EXCLUDED.escalated_at, vendor_inquiry_management.escalated_at),
            resolved_at = COALESCE(EXCLUDED.resolved_at, vendor_inquiry_management.resolved_at),
            updated_at = NOW()"
    )
    .bind(inq_uuid)
    .bind(input.assigned_admin_id)
    .bind(input.escalation_status)
    .bind(input.resolution_status)
    .bind(input.priority)
    .bind(assigned_at)
    .bind(escalated_at)
    .bind(resolved_at)
    .execute(&state.db)
    .await?;

    let _ = sqlx::query(
        "INSERT INTO system_events (user_id, event_type, message_ar, message_en)
         VALUES ($1, 'admin_inquiry_management_updated', $2, $3)",
    )
    .bind(super_admin_uuid)
    .bind(format!("تحديث إدارة الاستفسار «{}»", inq_uuid))
    .bind(format!(
        "Admin updated CRM management state for inquiry '{}'",
        inq_uuid
    ))
    .execute(&state.db)
    .await;

    Ok(Json(json!({
        "status": "success",
        "message": "Inquiry CRM management state updated successfully"
    })))
}
