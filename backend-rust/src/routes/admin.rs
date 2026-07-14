use crate::errors::AppError;
use crate::middleware::auth::{RequireAdmin, RequireAuth, RequireSuperAdmin, RlsTx};
use crate::models::user::DomainType;
use crate::state::AppState;
use crate::utils::crypto::hash_password;
use axum::{
    extract::{Path, State},
    routing::{delete, get, patch, post},
    Json, Router,
};
use serde::Deserialize;
use serde_json::{json, Value};
use sqlx::Row;
use uuid::Uuid;
use validator::Validate;

pub fn router() -> Router<AppState> {
    Router::new()

        .route(
            "/vendors/:id/chat/messages",
            get(get_admin_vendor_chat_messages),
        )
        .route(
            "/vendors/:id/chat/reply",
            post(post_admin_vendor_chat_reply),
        )

        .route("/analytics/summary", get(get_global_analytics))
        .route("/analytics/dashboard", get(get_dashboard_analytics))
        .route("/dashboard/stream", get(get_admin_dashboard_stream))




        .route("/audit/logs", get(list_audit_logs))
        .route("/audit/performance", get(get_audit_performance))
        .route(
            "/settings",
            get(get_admin_settings).put(update_admin_settings),
        )
        .route(
            "/notifications",
            get(list_admin_notifications).post(create_admin_notification),
        )
        .route("/notifications/recent", get(list_recent_notifications))
        .route(
            "/notifications/unread-count",
            get(get_unread_notifications_count),
        )
        .route("/notifications/mark-read", post(mark_notifications_read))
        .route("/conversations", get(list_conversations))
        .route(
            "/conversations/:id/messages",
            get(get_conversation_messages),
        )
        .route(
            "/afrah/conversations/:id/messages",
            post(send_afrah_message),
        )
        .route(
            "/conversations/:id/status",
            patch(update_conversation_status),
        )
        .route("/messages/:id", delete(delete_admin_message))
        .route(
            "/messages/unread-count",
            get(get_unread_messages_count),
        )



        .route("/outbox", get(list_outbox_events))
        .route("/outbox/metrics", get(get_outbox_metrics))
        .route(
            "/diagnostics/notifications",
            get(get_notification_diagnostics),
        )
}

async fn get_notification_diagnostics(
    State(state): State<AppState>,
    _auth: RequireAdmin,
) -> Result<Json<Value>, AppError> {
    let email_diag = state.email_service.get_diagnostics();
    let whatsapp_diag = state.whatsapp_service.get_diagnostics();

    Ok(Json(json!({
        "status": "success",
        "email": email_diag,
        "whatsapp": whatsapp_diag,
    })))
}

// ─── VENDOR STATUS MANAGEMENT ────────────────────────────────────────────────

/// GET /admin/vendors/:id — Full vendor detail with halls for admin review
// ─── REVIEW MODERATION ───────────────────────────────────────────────────────



#[derive(serde::Serialize, Debug)]
pub struct AdminDashboardAnalyticsResponse {
    pub total_active_vendors: i64,
    pub pending_reviews_count: i64,
    pub total_inquiries_count: i64,
    pub active_subscriptions_count: i64,
    pub pending_approvals_count: i64,
}

async fn get_dashboard_analytics(
    _auth: RequireAdmin,
    mut rls_tx: RlsTx,
) -> Result<Json<AdminDashboardAnalyticsResponse>, AppError> {
    tracing::info!("Aggregating dashboard analytics core telemetry...");

    let row = sqlx::query(
        "SELECT
            (SELECT COUNT(*) FROM vendors WHERE status = 'active')::bigint AS total_active_vendors,
            (SELECT COUNT(*) FROM vendor_reviews WHERE status = 'pending_approval')::bigint AS pending_reviews_count,
            (SELECT COUNT(*) FROM vendor_inquiries)::bigint AS total_inquiries_count,
            (SELECT COUNT(*) FROM vendors WHERE subscription_status IN ('trial', 'active'))::bigint AS active_subscriptions_count,
            (SELECT COUNT(*) FROM vendor_products WHERE status = 'pending_approval')::bigint AS pending_approvals_count"
    )
    .fetch_one(&mut *rls_tx.tx)
    .await?;

    let total_active_vendors: i64 = row.get("total_active_vendors");
    let pending_reviews_count: i64 = row.get("pending_reviews_count");
    let total_inquiries_count: i64 = row.get("total_inquiries_count");
    let active_subscriptions_count: i64 = row.get("active_subscriptions_count");
    let pending_approvals_count: i64 = row.get("pending_approvals_count");

    rls_tx
        .tx
        .commit()
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;

    Ok(Json(AdminDashboardAnalyticsResponse {
        total_active_vendors,
        pending_reviews_count,
        total_inquiries_count,
        active_subscriptions_count,
        pending_approvals_count,
    }))
}

// ─── PLATFORM ANALYTICS ──────────────────────────────────────────────────────

#[derive(serde::Deserialize)]
pub struct AnalyticsQueryParams {
    pub range: Option<String>,
}

async fn get_global_analytics(
    _auth: RequireSuperAdmin,
    axum::extract::Query(params): axum::extract::Query<AnalyticsQueryParams>,
    mut rls_tx: RlsTx,
) -> Result<Json<Value>, AppError> {
    tracing::info!(
        "Aggregating global platform telemetry metrics... range: {:?}",
        params.range
    );

    // Fetch platform commission rate from settings (bubble up errors if not found or invalid)
    let commission_rate_str = sqlx::query_scalar::<_, String>(
        "SELECT value FROM admin_settings WHERE key = 'platform_commission_rate'",
    )
    .fetch_optional(&mut *rls_tx.tx)
    .await?
    .ok_or_else(|| AppError::NotFound("platform_commission_rate not found".to_string()))?;

    let commission_rate: f64 = commission_rate_str
        .parse::<f64>()
        .map_err(|_| AppError::BadRequest("Invalid commission rate format".to_string()))?;

    let commission_fraction = commission_rate / 100.0;

    // 1. Calculate Gross Revenue from bookings casted as float8
    let revenue_row = sqlx::query(
        "SELECT COALESCE(SUM(total_price), 0.00)::float8 AS total_revenue, COUNT(*) AS total_bookings FROM core_bookings"
    )
    .fetch_one(&mut *rls_tx.tx)
    .await?;

    let total_revenue: f64 = revenue_row.get("total_revenue");
    let total_bookings: i64 = revenue_row.get("total_bookings");

    // 2. Count active suppliers count
    let active_vendors_row =
        sqlx::query("SELECT COUNT(*) AS total_active FROM vendors WHERE status = 'active'")
            .fetch_one(&mut *rls_tx.tx)
            .await?;

    let total_active_vendors: i64 = active_vendors_row.get("total_active");

    // 3. Count pending listing approvals
    let pending_row = sqlx::query(
        "SELECT COUNT(*) AS total_pending FROM vendor_products WHERE status = 'pending_approval'",
    )
    .fetch_one(&mut *rls_tx.tx)
    .await?;

    let total_pending: i64 = pending_row.get("total_pending");

    // 4. Count total registered users
    let total_users_row = sqlx::query("SELECT COUNT(*)::bigint AS total_users FROM global_users")
        .fetch_one(&mut *rls_tx.tx)
        .await?;

    let total_users_count: i64 = total_users_row.get("total_users");

    let range_val = params.range.as_deref().unwrap_or("last_12_months");
    let (interval_str, _) = match range_val {
        "this_month" => ("1 month", 1),
        "last_30_days" => ("1 month", 1),
        "last_6_months" => ("6 months", 6),
        _ => ("12 months", 12),
    };

    // 5. Query monthly revenue breakdown
    let monthly_revenue_rows = sqlx::query(&format!(
        "SELECT
            EXTRACT(MONTH FROM created_at)::int AS month_num,
            EXTRACT(YEAR FROM created_at)::int AS year_num,
            COALESCE(SUM(total_price), 0.00)::float8 AS revenue,
            COUNT(*)::bigint AS bookings_count
         FROM core_bookings
         WHERE created_at >= NOW() - INTERVAL '{}'
         GROUP BY year_num, month_num
         ORDER BY year_num ASC, month_num ASC",
        interval_str
    ))
    .fetch_all(&mut *rls_tx.tx)
    .await?;

    let users_trend_rows = sqlx::query(&format!(
        "SELECT
            EXTRACT(MONTH FROM created_at)::int AS month_num,
            EXTRACT(YEAR FROM created_at)::int AS year_num,
            COUNT(*)::bigint AS user_count
         FROM global_users
         WHERE created_at >= NOW() - INTERVAL '{}'
         GROUP BY year_num, month_num",
        interval_str
    ))
    .fetch_all(&mut *rls_tx.tx)
    .await?;

    let vendors_trend_rows = sqlx::query(&format!(
        "SELECT
            EXTRACT(MONTH FROM created_at)::int AS month_num,
            EXTRACT(YEAR FROM created_at)::int AS year_num,
            COUNT(*)::bigint AS vendor_count
         FROM vendors
         WHERE created_at >= NOW() - INTERVAL '{}'
         GROUP BY year_num, month_num",
        interval_str
    ))
    .fetch_all(&mut *rls_tx.tx)
    .await?;

    let current_date = chrono::Utc::now();
    let current_year = current_date
        .format("%Y")
        .to_string()
        .parse::<i32>()
        .unwrap_or(2026);
    let current_month = current_date
        .format("%m")
        .to_string()
        .parse::<i32>()
        .unwrap_or(5);

    let mut monthly_revenue = Vec::new();
    let mut bookings_trend = Vec::new();
    let mut users_trend = Vec::new();
    let mut vendors_trend = Vec::new();

    let mut prev_revenue = 0.0;

    let mut recent_6_rev = 0.0;
    let mut prev_6_rev = 0.0;

    for i in (0..12).rev() {
        let mut target_month = current_month - i;
        let mut target_year = current_year;
        while target_month <= 0 {
            target_month += 12;
            target_year -= 1;
        }

        let mut revenue = 0.0;
        let mut bookings: i64 = 0;
        for row in &monthly_revenue_rows {
            let m: i32 = row.get("month_num");
            let y: i32 = row.get("year_num");
            if m == target_month && y == target_year {
                revenue = row.get("revenue");
                bookings = row.get("bookings_count");
                break;
            }
        }

        let mut u_count: i64 = 0;
        for row in &users_trend_rows {
            let m: i32 = row.get("month_num");
            let y: i32 = row.get("year_num");
            if m == target_month && y == target_year {
                u_count = row.get("user_count");
                break;
            }
        }

        let mut v_count: i64 = 0;
        for row in &vendors_trend_rows {
            let m: i32 = row.get("month_num");
            let y: i32 = row.get("year_num");
            if m == target_month && y == target_year {
                v_count = row.get("vendor_count");
                break;
            }
        }

        let month_name_en = match target_month {
            1 => "Jan",
            2 => "Feb",
            3 => "Mar",
            4 => "Apr",
            5 => "May",
            6 => "Jun",
            7 => "Jul",
            8 => "Aug",
            9 => "Sep",
            10 => "Oct",
            11 => "Nov",
            12 => "Dec",
            _ => "Unknown",
        };

        let month_name_ar = match target_month {
            1 => "يناير",
            2 => "فبراير",
            3 => "مارس",
            4 => "أبريل",
            5 => "مايو",
            6 => "يونيو",
            7 => "يوليو",
            8 => "أغسطس",
            9 => "سبتمبر",
            10 => "أكتوبر",
            11 => "نوفمبر",
            12 => "ديسمبر",
            _ => "غير معروف",
        };

        let growth = if prev_revenue > 0.0 {
            ((revenue - prev_revenue) / prev_revenue) * 100.0
        } else {
            0.0
        };

        monthly_revenue.push(json!({
            "month_ar": month_name_ar,
            "month_en": month_name_en,
            "revenue": revenue,
            "bookings": bookings,
            "commission": revenue * commission_fraction,
            "growth": (growth * 10.0_f64).round() / 10.0_f64
        }));

        bookings_trend.push(bookings);
        users_trend.push(u_count);
        vendors_trend.push(v_count);

        if i < 6 {
            recent_6_rev += revenue;
        } else {
            prev_6_rev += revenue;
        }

        prev_revenue = revenue;
    }

    // actual yoy_growth mathematically
    let yoy_growth_val = if prev_6_rev > 0.0 {
        ((recent_6_rev - prev_6_rev) / prev_6_rev) * 100.0
    } else {
        if recent_6_rev > 0.0 {
            100.0
        } else {
            0.0
        }
    };

    let yoy_growth_str = if yoy_growth_val >= 0.0 {
        format!("+{:.1}%", yoy_growth_val)
    } else {
        format!("{:.1}%", yoy_growth_val)
    };

    // 6. Query recent system events (activity feed) — latest 10
    let recent_events_rows = sqlx::query(
        "SELECT
            se.id,
            se.event_type,
            se.message_ar,
            se.message_en,
            se.created_at,
            gu.email AS actor_email
         FROM system_events se
         LEFT JOIN global_users gu ON se.user_id = gu.id
         ORDER BY se.created_at DESC
         LIMIT 10",
    )
    .fetch_all(&mut *rls_tx.tx)
    .await?;

    let mut recent_activities = Vec::new();
    for row in recent_events_rows {
        let id: Uuid = row.get("id");
        let event_type: String = row.get("event_type");
        let message_ar: String = row.get("message_ar");
        let message_en: String = row.get("message_en");
        let created_at: chrono::DateTime<chrono::Utc> = row.get("created_at");
        let actor_email: Option<String> = row.get("actor_email");

        recent_activities.push(json!({
            "id": id.to_string(),
            "eventType": event_type,
            "messageAr": message_ar,
            "messageEn": message_en,
            "operatorEmail": actor_email.unwrap_or_else(|| "System".to_string()),
            "ip": "127.0.0.1",
            "createdAt": created_at.to_rfc3339()
        }));
    }

    // 7. Query support tickets metrics
    let support_row = sqlx::query(
        "SELECT COUNT(*) AS open_count FROM system_events WHERE event_type = 'system_alert' AND is_read = FALSE"
    )
    .fetch_one(&mut *rls_tx.tx)
    .await?;

    let open_support_tickets: i64 = support_row.get("open_count");

    // 8. Query real regional distribution from vendors, bookings, and inquiries city_id columns
    let regional_rows = sqlx::query(
        "WITH
         city_vendors AS (
             SELECT city_id, COUNT(*)::bigint AS vendor_count
             FROM vendors
             WHERE city_id IS NOT NULL
             GROUP BY city_id
         ),
         city_bookings AS (
             SELECT city_id,
                    COUNT(*)::bigint                         AS bookings_count,
                    COALESCE(SUM(total_price), 0)::float8   AS revenue
             FROM core_bookings
             WHERE city_id IS NOT NULL
             GROUP BY city_id
         ),
         city_inquiries AS (
             SELECT city_id, COUNT(*)::bigint AS inquiry_count
             FROM lead_inquiries
             WHERE city_id IS NOT NULL
             GROUP BY city_id
         ),
         all_cities AS (
             SELECT city_id FROM city_vendors
             UNION
             SELECT city_id FROM city_bookings
             UNION
             SELECT city_id FROM city_inquiries
         )
         SELECT
             ci.name_en                                              AS region,
             ci.name_ar                                              AS region_ar,
             ci.slug                                                 AS city_slug,
             COALESCE(cv.vendor_count,   0)::bigint                 AS vendor_count,
             COALESCE(cb.bookings_count, 0)::bigint                 AS bookings_count,
             COALESCE(cq.inquiry_count,  0)::bigint                 AS inquiry_count,
             COALESCE(cb.revenue,        0.0)::float8               AS revenue
         FROM all_cities ac
         JOIN cities ci ON ci.id = ac.city_id
         LEFT JOIN city_vendors   cv ON cv.city_id = ac.city_id
         LEFT JOIN city_bookings  cb ON cb.city_id = ac.city_id
         LEFT JOIN city_inquiries cq ON cq.city_id = ac.city_id
         ORDER BY bookings_count DESC, vendor_count DESC
         LIMIT 20",
    )
    .fetch_all(&mut *rls_tx.tx)
    .await?;

    let total_regional_bookings: i64 = regional_rows
        .iter()
        .map(|r| r.get::<i64, _>("bookings_count"))
        .sum();

    let mut regional_json = Vec::new();
    for row in &regional_rows {
        let region: String = row.get("region");
        let region_ar: String = row.get("region_ar");
        let city_slug: String = row.get("city_slug");
        let vendor_count: i64 = row.get("vendor_count");
        let bookings_count: i64 = row.get("bookings_count");
        let inquiry_count: i64 = row.get("inquiry_count");
        let revenue: f64 = row.get("revenue");

        let booking_pct = if total_regional_bookings > 0 {
            (bookings_count as f64 / total_regional_bookings as f64) * 100.0
        } else {
            0.0
        };

        regional_json.push(json!({
            "region":         region,
            "region_ar":      region_ar,
            "city_slug":      city_slug,
            "vendor_count":   vendor_count,
            "bookings_count": bookings_count,
            "inquiry_count":  inquiry_count,
            "revenue":        revenue,
            "booking_pct":    (booking_pct * 10.0_f64).round() / 10.0_f64
        }));
    }

    // 9. Query dynamic category distribution
    let category_rows = sqlx::query(
        "SELECT
            p.product_category AS category,
            COUNT(*)::bigint AS category_count,
            (COUNT(*)::float8 / (SELECT COUNT(*) FROM core_bookings)::float8 * 100.0) AS percentage
         FROM core_bookings cb
         JOIN vendor_products p ON cb.product_id = p.id
         GROUP BY p.product_category"
    )
    .fetch_all(&mut *rls_tx.tx)
    .await?;

    let mut category_json = Vec::new();
    for row in category_rows {
        let category: Option<String> = row.get("category");
        let category_count: i64 = row.get("category_count");
        let percentage: f64 = row.get("percentage");

        category_json.push(json!({
            "category": category.unwrap_or_default(),
            "category_count": category_count,
            "percentage": percentage
        }));
    }

    rls_tx.tx.commit().await?;

    Ok(Json(json!({
        "status": "success",
        "summary": {
            "total_revenue": total_revenue,
            "commission_estimate": total_revenue * commission_fraction,
            "commission_rate": commission_rate,
            "payout_totals": total_revenue * (1.0 - commission_fraction),
            "total_bookings_count": total_bookings,
            "active_vendors_count": total_active_vendors,
            "pending_approvals_count": total_pending,
            "total_users_count": total_users_count,
            "open_support_tickets": open_support_tickets,
            "monthly_revenue": monthly_revenue,
            "bookings_trend": bookings_trend,
            "users_trend": users_trend,
            "vendors_trend": vendors_trend,
            "yoy_growth": yoy_growth_str,
            "regional_distribution": regional_json,
            "category_distribution": category_json,
            "recent_activities": recent_activities
        }
    })))
}


async fn get_admin_dashboard_stream(
    _auth: RequireAdmin,
    mut rls_tx: RlsTx,
) -> Result<Json<Value>, AppError> {
    // 1. Query global transaction counts (total bookings)
    let global_transaction_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*)::bigint FROM core_bookings")
            .fetch_one(&mut *rls_tx.tx)
            .await?;

    // 2. Query active system-wide escrow volume total (deposits for Escrow_Verified or Booking_Active)
    let active_escrow_volume: f64 = sqlx::query_scalar(
        "SELECT COALESCE(SUM(deposit_paid), 0.00)::float8 FROM core_bookings 
         WHERE status IN ('Escrow_Verified', 'Booking_Active')",
    )
    .fetch_one(&mut *rls_tx.tx)
    .await?;

    // 3. Query recent transaction logs across the platform
    let transaction_rows = sqlx::query(
        "SELECT id, booking_number, status, total_price::float8 as price, deposit_paid::float8 as deposit, created_at::text 
         FROM core_bookings 
         ORDER BY created_at DESC 
         LIMIT 10"
    )
    .fetch_all(&mut *rls_tx.tx)
    .await?;

    let mut transaction_logs = Vec::new();
    for row in transaction_rows {
        transaction_logs.push(json!({
            "id": row.get::<Uuid, _>("id"),
            "bookingNumber": row.get::<String, _>("booking_number"),
            "status": row.get::<String, _>("status"),
            "totalPrice": row.get::<f64, _>("price"),
            "depositPaid": row.get::<f64, _>("deposit"),
            "createdAt": row.get::<String, _>("created_at"),
        }));
    }

    rls_tx
        .tx
        .commit()
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;

    Ok(Json(json!({
        "status": "success",
        "data": {
            "macroMetrics": {
                "globalTransactionCount": global_transaction_count,
                "activeEscrowVolumeTotal": active_escrow_volume,
            },
            "recentTransactions": transaction_logs
        }
    })))
}





#[derive(serde::Deserialize)]
pub struct AdminChatReplyInput {
    pub body: String,
}

async fn get_admin_vendor_chat_messages(
    mut rls_tx: RlsTx,
    _auth: RequireAdmin,
    Path(vendor_id_str): Path<String>,
) -> Result<Json<Value>, AppError> {
    let vendor_id = Uuid::parse_str(&vendor_id_str)
        .map_err(|_| AppError::BadRequest("Invalid vendor UUID format".to_string()))?;

    // 1. Get or Create chat
    let chat_id: Uuid = sqlx::query_scalar(
        "INSERT INTO vendor_admin_chats (vendor_id)
         VALUES ($1)
         ON CONFLICT (vendor_id) DO UPDATE SET updated_at = NOW()
         RETURNING id",
    )
    .bind(vendor_id)
    .fetch_one(&mut *rls_tx.tx)
    .await?;

    // 2. Fetch all messages in the thread
    let messages = sqlx::query(
        "SELECT id, sender, body, file_url, is_read, created_at
         FROM chat_messages
         WHERE chat_id = $1
         ORDER BY created_at ASC",
    )
    .bind(chat_id)
    .fetch_all(&mut *rls_tx.tx)
    .await?
    .into_iter()
    .map(|row| {
        let id: Uuid = row.get("id");
        let sender: String = row.get("sender");
        let body: String = row.get("body");
        let file_url: Option<String> = row.get("file_url");
        let is_read: bool = row.get("is_read");
        let created_at: chrono::DateTime<chrono::Utc> = row.get("created_at");
        json!({
            "id": id,
            "sender": sender,
            "body": body,
            "file_url": file_url,
            "is_read": is_read,
            "created_at": created_at,
        })
    })
    .collect::<Vec<Value>>();

    // 3. Mark vendor messages as read by admin
    sqlx::query(
        "UPDATE chat_messages
         SET is_read = TRUE
         WHERE chat_id = $1 AND sender = 'vendor' AND is_read = FALSE",
    )
    .bind(chat_id)
    .execute(&mut *rls_tx.tx)
    .await?;

    rls_tx.tx.commit().await?;

    Ok(Json(json!({
        "status": "success",
        "chat_id": chat_id,
        "messages": messages,
    })))
}

async fn post_admin_vendor_chat_reply(
    mut rls_tx: RlsTx,
    _auth: RequireAdmin,
    Path(vendor_id_str): Path<String>,
    Json(payload): Json<AdminChatReplyInput>,
) -> Result<Json<Value>, AppError> {
    if payload.body.trim().is_empty() {
        return Err(AppError::BadRequest(
            "Message body cannot be empty".to_string(),
        ));
    }

    let vendor_id = Uuid::parse_str(&vendor_id_str)
        .map_err(|_| AppError::BadRequest("Invalid vendor UUID format".to_string()))?;

    // 1. Get or Create chat
    let chat_id: Uuid = sqlx::query_scalar(
        "INSERT INTO vendor_admin_chats (vendor_id)
         VALUES ($1)
         ON CONFLICT (vendor_id) DO UPDATE SET updated_at = NOW()
         RETURNING id",
    )
    .bind(vendor_id)
    .fetch_one(&mut *rls_tx.tx)
    .await?;

    // 2. Insert message
    let message_id: Uuid = sqlx::query_scalar(
        "INSERT INTO chat_messages (chat_id, sender, body, is_read)
         VALUES ($1, 'admin', $2, FALSE)
         RETURNING id",
    )
    .bind(chat_id)
    .bind(&payload.body)
    .fetch_one(&mut *rls_tx.tx)
    .await?;

    // 3. Update chat timestamp
    sqlx::query("UPDATE vendor_admin_chats SET updated_at = NOW() WHERE id = $1")
        .bind(chat_id)
        .execute(&mut *rls_tx.tx)
        .await?;

    rls_tx.tx.commit().await?;

    Ok(Json(json!({
        "status": "success",
        "message_id": message_id,
        "message": "Message sent successfully"
    })))
}

// ─── VENDOR FEATURED MONETIZATION MANAGEMENT ───────────────────────────────



// ─── ADMIN BOOKINGS DIRECTORY ─────────────────────────────────────────────────

#[derive(serde::Deserialize)]
pub struct AuditLogsQuery {
    pub page: Option<i64>,
    pub limit: Option<i64>,
    pub event_type: Option<String>,
    pub operator_search: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
}

async fn list_audit_logs(
    _auth: RequireAdmin,
    mut rls_tx: RlsTx,
    axum::extract::Query(query): axum::extract::Query<AuditLogsQuery>,
) -> Result<Json<Value>, AppError> {
    tracing::info!("Admin querying system audit logs feed...");

    let page = query.page.unwrap_or(1).max(1);
    let limit = query.limit.unwrap_or(10).clamp(1, 100);
    let offset = (page - 1) * limit;

    let mut query_builder = String::from(
        "FROM system_events se LEFT JOIN global_users gu ON se.user_id = gu.id WHERE 1 = 1",
    );
    let mut param_idx = 1;
    let mut bindings = Vec::new();

    if let Some(ref et) = query.event_type {
        if et != "all" && !et.trim().is_empty() {
            query_builder.push_str(&format!(" AND se.event_type = ${}", param_idx));
            bindings.push(et.clone());
            param_idx += 1;
        }
    }

    if let Some(ref search_term) = query.operator_search {
        let clean_search = search_term.trim();
        if !clean_search.is_empty() {
            query_builder.push_str(&format!(" AND gu.email ILIKE ${}", param_idx));
            bindings.push(format!("%{}%", clean_search));
            param_idx += 1;
        }
    }

    if let Some(ref start) = query.start_date {
        if !start.trim().is_empty() {
            query_builder.push_str(&format!(
                " AND se.created_at >= ${}::timestamptz",
                param_idx
            ));
            bindings.push(start.clone());
            param_idx += 1;
        }
    }

    if let Some(ref end) = query.end_date {
        if !end.trim().is_empty() {
            query_builder.push_str(&format!(
                " AND se.created_at <= ${}::timestamptz",
                param_idx
            ));
            bindings.push(end.clone());
            param_idx += 1;
        }
    }

    // 1. Query total matching count
    let count_query = format!("SELECT COUNT(*)::bigint {}", query_builder);
    let mut sql_count = sqlx::query_scalar(&count_query);
    for b in &bindings {
        sql_count = sql_count.bind(b);
    }
    let total_count: i64 = sql_count.fetch_one(&mut *rls_tx.tx).await?;

    // 2. Query paginated audit feed
    let select_query = format!(
        "SELECT 
            se.id, se.event_type, se.message_ar, se.message_en, se.created_at,
            gu.email AS operator_email
         {}
         ORDER BY se.created_at DESC
         LIMIT ${} OFFSET ${}",
        query_builder,
        param_idx,
        param_idx + 1
    );

    let mut sql_select = sqlx::query(&select_query);
    for b in &bindings {
        sql_select = sql_select.bind(b);
    }
    sql_select = sql_select.bind(limit).bind(offset);
    let rows = sql_select.fetch_all(&mut *rls_tx.tx).await?;

    let mut logs_list = Vec::new();
    for row in rows {
        let id: Uuid = row.get("id");
        let event_type: String = row.get("event_type");
        let message_ar: String = row.get("message_ar");
        let message_en: String = row.get("message_en");
        let created_at: chrono::DateTime<chrono::Utc> = row.get("created_at");
        let operator_email: String = row.get("operator_email");

        logs_list.push(json!({
            "id": id.to_string(),
            "operatorEmail": operator_email,
            "eventType": event_type,
            "messageAr": message_ar,
            "messageEn": message_en,
            "ip": "127.0.0.1",
            "createdAt": created_at.to_rfc3339()
        }));
    }

    rls_tx.tx.commit().await?;

    let total_pages = ((total_count + limit - 1) / limit).max(1);

    Ok(Json(json!({
        "status": "success",
        "logs": logs_list,
        "total": total_count,
        "page": page,
        "totalPages": total_pages
    })))
}

// ─── PHASE 3 ADMIN SETTINGS HANDLERS ─────────────────────────────────────────

#[derive(serde::Deserialize)]
struct UpdateSettingsRequest {
    settings: std::collections::HashMap<String, String>,
}

async fn get_admin_settings(
    _auth: RequireAdmin,
    mut rls_tx: RlsTx,
) -> Result<Json<Value>, AppError> {
    let rows = sqlx::query("SELECT key, value FROM admin_settings")
        .fetch_all(&mut *rls_tx.tx)
        .await?;

    let mut settings = std::collections::HashMap::new();
    for row in rows {
        let k: String = row.get("key");
        let v: String = row.get("value");
        settings.insert(k, v);
    }

    rls_tx.tx.commit().await?;

    Ok(Json(json!({
        "status": "success",
        "settings": settings
    })))
}

async fn update_admin_settings(
    auth: RequireSuperAdmin,
    mut rls_tx: RlsTx,
    Json(payload): Json<UpdateSettingsRequest>,
) -> Result<Json<Value>, AppError> {
    let admin_uuid = Uuid::parse_str(&auth.user_id)
        .map_err(|_| AppError::BadRequest("Invalid admin user ID format".to_string()))?;

    for (k, v) in &payload.settings {
        sqlx::query(
            "INSERT INTO admin_settings (key, value, updated_by, updated_at)
             VALUES ($1, $2, $3, NOW())
             ON CONFLICT (key)
             DO UPDATE SET value = EXCLUDED.value, updated_by = EXCLUDED.updated_by, updated_at = NOW()"
        )
        .bind(k)
        .bind(v)
        .bind(admin_uuid)
        .execute(&mut *rls_tx.tx)
        .await?;
    }

    // Audit settings modification in system_events
    sqlx::query(
        "INSERT INTO system_events (user_id, event_type, message_ar, message_en)
         VALUES ($1, 'system_alert', $2, $3)",
    )
    .bind(admin_uuid)
    .bind("قام المسؤول بتحديث إعدادات المنصة")
    .bind("Admin updated platform settings configurations")
    .execute(&mut *rls_tx.tx)
    .await?;

    rls_tx.tx.commit().await?;

    Ok(Json(json!({
        "status": "success",
        "message": "Settings updated successfully"
    })))
}

// ─── PHASE 3 ADMIN NOTIFICATIONS CENTER ──────────────────────────────────────

#[derive(serde::Deserialize)]
struct CreateNotificationRequest {
    #[serde(rename = "titleAr")]
    title_ar: Option<String>,
    #[serde(rename = "titleEn")]
    title_en: Option<String>,
    #[serde(rename = "messageAr")]
    message_ar: String,
    #[serde(rename = "messageEn")]
    message_en: String,
    #[serde(rename = "targetAudience")]
    target_audience: String, // "all" or specific vendor UUID
}

async fn list_admin_notifications(
    _auth: RequireAdmin,
    mut rls_tx: RlsTx,
) -> Result<Json<Value>, AppError> {
    let rows = sqlx::query(
        "SELECT 
            se.id,
            se.event_type,
            se.message_ar,
            se.message_en,
            se.is_read,
            se.created_at,
            se.target_vendor_id,
            v.brand_name_en AS target_vendor_name_en,
            v.brand_name_ar AS target_vendor_name_ar,
            gu.email AS operator_email
         FROM system_events se
         LEFT JOIN global_users gu ON se.user_id = gu.id
         LEFT JOIN vendors v ON se.target_vendor_id = v.id
         WHERE se.event_type = 'system_alert'
         ORDER BY se.created_at DESC
         LIMIT 100",
    )
    .fetch_all(&mut *rls_tx.tx)
    .await?;

    let mut notifications = Vec::new();
    for row in rows {
        let id: Uuid = row.get("id");
        let event_type: String = row.get("event_type");
        let message_ar: String = row.get("message_ar");
        let message_en: String = row.get("message_en");
        let is_read: bool = row.get("is_read");
        let created_at: chrono::DateTime<chrono::Utc> = row.get("created_at");
        let target_vendor_id: Option<Uuid> = row.get("target_vendor_id");
        let target_vendor_name_en: Option<String> = row.get("target_vendor_name_en");
        let target_vendor_name_ar: Option<String> = row.get("target_vendor_name_ar");
        let operator_email: Option<String> = row.get("operator_email");

        // Map target audience
        let (audience_ar, audience_en) = match &target_vendor_name_en {
            Some(name_en) => {
                let name_ar = target_vendor_name_ar.as_deref().unwrap_or(name_en);
                (
                    format!("المورد: {}", name_ar),
                    format!("Vendor: {}", name_en),
                )
            }
            None => (
                "جميع الموردين والمستخدمين".to_string(),
                "All vendors and users".to_string(),
            ),
        };

        notifications.push(json!({
            "id": id.to_string(),
            "event_type": event_type,
            "message_ar": message_ar,
            "message_en": message_en,
            "is_read": is_read,
            "created_at": created_at.to_rfc3339(),
            "target_vendor_id": target_vendor_id.map(|u| u.to_string()),
            "audience_ar": audience_ar,
            "audience_en": audience_en,
            "operator_email": operator_email
        }));
    }

    rls_tx.tx.commit().await?;

    Ok(Json(json!({
        "status": "success",
        "notifications": notifications
    })))
}

async fn create_admin_notification(
    auth: RequireAdmin,
    mut rls_tx: RlsTx,
    Json(payload): Json<CreateNotificationRequest>,
) -> Result<Json<Value>, AppError> {
    let admin_uuid = Uuid::parse_str(&auth.user_id)
        .map_err(|_| AppError::BadRequest("Invalid admin user ID format".to_string()))?;

    let vendor_uuid =
        if payload.target_audience != "all" && !payload.target_audience.trim().is_empty() {
            let parsed = Uuid::parse_str(&payload.target_audience)
                .map_err(|_| AppError::BadRequest("Invalid target vendor ID format".to_string()))?;
            Some(parsed)
        } else {
            None
        };

    let target_user_uuid = if let Some(v_id) = vendor_uuid {
        let v_row = sqlx::query("SELECT user_id FROM vendors WHERE id = $1")
            .bind(v_id)
            .fetch_optional(&mut *rls_tx.tx)
            .await?;
        match v_row {
            Some(row) => {
                let u_id: Uuid = row.get("user_id");
                u_id
            }
            None => return Err(AppError::BadRequest("Target vendor not found".to_string())),
        }
    } else {
        admin_uuid
    };

    let final_message_ar = format!(
        "{}: {}",
        payload.title_ar.as_deref().unwrap_or("تنبيه نظام"),
        payload.message_ar
    );
    let final_message_en = format!(
        "{}: {}",
        payload.title_en.as_deref().unwrap_or("System Alert"),
        payload.message_en
    );

    // Insert the system notification
    sqlx::query(
        "INSERT INTO system_events (user_id, target_vendor_id, event_type, message_ar, message_en)
         VALUES ($1, $2, 'system_alert', $3, $4)",
    )
    .bind(target_user_uuid)
    .bind(vendor_uuid)
    .bind(&final_message_ar)
    .bind(&final_message_en)
    .execute(&mut *rls_tx.tx)
    .await?;

    // Log the audit event for notification creation
    sqlx::query(
        "INSERT INTO system_events (user_id, event_type, message_ar, message_en)
         VALUES ($1, 'system_alert', $2, $3)",
    )
    .bind(admin_uuid)
    .bind(format!("قام المسؤول بإرسال إشعار: {}", final_message_en))
    .bind(format!(
        "Admin dispatched system notification: {}",
        final_message_en
    ))
    .execute(&mut *rls_tx.tx)
    .await?;

    rls_tx.tx.commit().await?;

    Ok(Json(json!({
        "status": "success",
        "message": "Notification dispatched successfully"
    })))
}

// ─── PHASE 3 ADMIN MARKETING CAMPAIGNS HANDLER ────────────────────────────────



async fn list_conversations(
    mut rls_tx: RlsTx,
    _auth: RequireAdmin,
) -> Result<Json<Value>, AppError> {
    // Admin can see all conversations.
    let rows = sqlx::query(
        r#"SELECT 
            c.id, c.title, c.status, c.created_at::text, c.updated_at::text, c.product_id,
            u_client.id AS client_user_id, u_vendor.id AS vendor_user_id,
            cl.first_name AS client_first_name, cl.last_name AS client_last_name,
            cl.phone AS client_phone, cl.wedding_date::text AS client_wedding_date,
            u_client.email AS client_email,
            v.name_en AS vendor_name_en, v.name_ar AS vendor_name_ar,
            v.phone AS vendor_phone, u_vendor.email AS vendor_email,
            p.title AS product_name_en, p.title AS product_name_ar,
            p.base_price_sar::float8 AS product_price,
            (
                SELECT image_url FROM vendor_gallery 
                WHERE product_id = c.product_id AND is_cover = TRUE 
                LIMIT 1
            ) AS product_cover_image
         FROM conversations c
         LEFT JOIN conversation_participants cp_client ON c.id = cp_client.conversation_id 
             AND cp_client.user_id IN (SELECT id FROM global_users WHERE domain_type = 'Client')
         LEFT JOIN global_users u_client ON cp_client.user_id = u_client.id
         LEFT JOIN client_profiles cl ON u_client.id = cl.client_id
         LEFT JOIN conversation_participants cp_vendor ON c.id = cp_vendor.conversation_id 
             AND cp_vendor.user_id IN (SELECT id FROM global_users WHERE domain_type = 'Vendor')
         LEFT JOIN global_users u_vendor ON cp_vendor.user_id = u_vendor.id
         LEFT JOIN vendors v ON cp_vendor.user_id = v.user_id
         LEFT JOIN vendor_products p ON c.product_id = p.id
         ORDER BY c.updated_at DESC
         LIMIT 100"#,
    )
    .fetch_all(&mut *rls_tx.tx)
    .await?;

    let mut conversations = Vec::new();
    for row in rows {
        conversations.push(json!({
            "id": row.get::<Uuid, _>("id"),
            "title": row.get::<Option<String>, _>("title"),
            "status": row.get::<String, _>("status"),
            "createdAt": row.get::<String, _>("created_at"),
            "updatedAt": row.get::<String, _>("updated_at"),
            "productId": row.get::<Option<Uuid>, _>("product_id"),
            "clientUserId": row.get::<Option<Uuid>, _>("client_user_id"),
            "vendorUserId": row.get::<Option<Uuid>, _>("vendor_user_id"),
            "clientFirstName": row.get::<Option<String>, _>("client_first_name"),
            "clientLastName": row.get::<Option<String>, _>("client_last_name"),
            "clientPhone": row.get::<Option<String>, _>("client_phone"),
            "clientWeddingDate": row.get::<Option<String>, _>("client_wedding_date"),
            "clientEmail": row.get::<Option<String>, _>("client_email"),
            "vendorNameEn": row.get::<Option<String>, _>("vendor_name_en"),
            "vendorNameAr": row.get::<Option<String>, _>("vendor_name_ar"),
            "vendorPhone": row.get::<Option<String>, _>("vendor_phone"),
            "vendorEmail": row.get::<Option<String>, _>("vendor_email"),
            "productNameEn": row.get::<Option<String>, _>("product_name_en"),
            "productNameAr": row.get::<Option<String>, _>("product_name_ar"),
            "productPrice": row.get::<Option<f64>, _>("product_price"),
            "productCoverImage": row.get::<Option<String>, _>("product_cover_image"),
        }));
    }

    rls_tx.tx.commit().await?;

    Ok(Json(json!({ "status": "success", "data": conversations })))
}

async fn get_conversation_messages(
    _auth: RequireAdmin,
    Path(id): Path<Uuid>,
    mut rls_tx: RlsTx,
) -> Result<Json<Value>, AppError> {
    let rows = sqlx::query(
        r#"SELECT m.id, m.conversation_id, m.sender_id, COALESCE(m.body, '') AS body, m.created_at::text, m.updated_at::text,
                  COALESCE(
                      jsonb_agg(
                          jsonb_build_object('fileUrl', a.file_url)
                      ) FILTER (WHERE a.id IS NOT NULL),
                      '[]'
                  ) as attachments
           FROM messages m
           LEFT JOIN message_attachments a ON m.id = a.message_id
           WHERE m.conversation_id = $1
           GROUP BY m.id
           ORDER BY m.created_at ASC"#
    )
    .bind(id)
    .fetch_all(&mut *rls_tx.tx)
    .await?;

    let mut messages = Vec::new();
    for row in rows {
        messages.push(json!({
            "id": row.get::<Uuid, _>("id"),
            "conversationId": row.get::<Uuid, _>("conversation_id"),
            "senderId": row.get::<Uuid, _>("sender_id"),
            "body": row.get::<String, _>("body"),
            "createdAt": row.get::<String, _>("created_at"),
            "attachments": row.get::<Value, _>("attachments")
        }));
    }

    rls_tx.tx.commit().await?;

    Ok(Json(json!({ "status": "success", "data": messages })))
}

#[derive(serde::Deserialize)]
pub struct UpdateConversationStatusRequest {
    pub status: String,
}

async fn update_conversation_status(
    mut rls_tx: RlsTx,
    _auth: RequireAdmin,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateConversationStatusRequest>,
) -> Result<Json<Value>, AppError> {
    if payload.status != "active" && payload.status != "closed" && payload.status != "archived" {
        return Err(AppError::BadRequest("Invalid status".into()));
    }

    sqlx::query("UPDATE conversations SET status = $1, updated_at = NOW() WHERE id = $2")
        .bind(&payload.status)
        .bind(id)
        .execute(&mut *rls_tx.tx)
        .await?;

    rls_tx
        .tx
        .commit()
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;

    Ok(Json(json!({ "status": "success" })))
}

#[derive(serde::Deserialize)]
pub struct MarkReadRequest {
    pub ids: Option<Vec<Uuid>>,
}

async fn list_recent_notifications(
    _auth: RequireAdmin,
    mut rls_tx: RlsTx,
) -> Result<Json<Value>, AppError> {
    let rows = sqlx::query(
        "SELECT 
            id,
            event_type,
            message_ar,
            message_en,
            is_read,
            created_at
         FROM system_events
         ORDER BY created_at DESC
         LIMIT 10",
    )
    .fetch_all(&mut *rls_tx.tx)
    .await?;

    let mut list = Vec::new();
    for row in rows {
        let id: Uuid = row.get("id");
        let event_type: String = row.get("event_type");
        let message_ar: String = row.get("message_ar");
        let message_en: String = row.get("message_en");
        let is_read: bool = row.get("is_read");
        let created_at: chrono::DateTime<chrono::Utc> = row.get("created_at");

        list.push(json!({
            "id": id.to_string(),
            "event_type": event_type,
            "message_ar": message_ar,
            "message_en": message_en,
            "is_read": is_read,
            "created_at": created_at.to_rfc3339()
        }));
    }

    rls_tx.tx.commit().await?;

    Ok(Json(json!({
        "status": "success",
        "notifications": list
    })))
}

async fn get_unread_notifications_count(
    _auth: RequireAdmin,
    mut rls_tx: RlsTx,
) -> Result<Json<Value>, AppError> {
    let count: i64 =
        sqlx::query_scalar("SELECT COUNT(*)::bigint FROM system_events WHERE is_read = FALSE")
            .fetch_one(&mut *rls_tx.tx)
            .await?;

    rls_tx.tx.commit().await?;

    Ok(Json(json!({
        "status": "success",
        "unreadCount": count
    })))
}

async fn get_unread_messages_count(
    State(state): State<AppState>,
    _auth: RequireAdmin,
) -> Result<Json<Value>, AppError> {
    let inquiries_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM vendor_inquiries WHERE status = 'unread'")
        .fetch_one(&state.db)
        .await?;

    let chat_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM chat_messages WHERE sender != 'admin' AND is_read = FALSE")
        .fetch_one(&state.db)
        .await?;

    Ok(Json(json!({
        "status": "success",
        "inquiries": inquiries_count,
        "chats": chat_count,
        "total": inquiries_count + chat_count
    })))
}

async fn mark_notifications_read(
    _auth: RequireAdmin,
    mut rls_tx: RlsTx,
    Json(payload): Json<MarkReadRequest>,
) -> Result<Json<Value>, AppError> {
    if let Some(ids) = payload.ids {
        if !ids.is_empty() {
            sqlx::query("UPDATE system_events SET is_read = TRUE WHERE id = ANY($1)")
                .bind(&ids)
                .execute(&mut *rls_tx.tx)
                .await?;
        }
    } else {
        // Mark all as read
        sqlx::query("UPDATE system_events SET is_read = TRUE WHERE is_read = FALSE")
            .execute(&mut *rls_tx.tx)
            .await?;
    }

    rls_tx.tx.commit().await?;

    Ok(Json(json!({
        "status": "success",
        "message": "Notifications marked as read"
    })))
}

async fn delete_admin_message(
    mut rls_tx: RlsTx,
    _auth: RequireAdmin,
    Path(message_id): Path<Uuid>,
) -> Result<Json<Value>, AppError> {
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
        "message": "Message successfully redacted by admin"
    })))
}



/// PATCH /admin/users/:id/status
/// Updates a user's account status (e.g., active, suspended, banned)
#[derive(Deserialize)]
struct AfrahMessageRequest {
    body: String,
}

async fn send_afrah_message(
    auth: RequireAuth,
    mut rls_tx: RlsTx,
    axum::extract::Path(conversation_id): axum::extract::Path<Uuid>,
    Json(payload): Json<AfrahMessageRequest>,
) -> Result<Json<Value>, AppError> {
    if auth.role != DomainType::Admin {
        return Err(AppError::Forbidden(
            "Admin credentials required".to_string(),
        ));
    }

    let afrah_id: Uuid = sqlx::query_scalar(
        "SELECT id FROM global_users WHERE is_system_account = true AND display_name = 'Afrah' LIMIT 1"
    )
    .fetch_optional(&mut *rls_tx.tx)
    .await?
    .ok_or_else(|| AppError::Internal("Afrah system user not found".to_string()))?;

    let msg_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO messages (id, conversation_id, sender_id, body) VALUES ($1, $2, $3, $4)",
    )
    .bind(msg_id)
    .bind(conversation_id)
    .bind(afrah_id)
    .bind(&payload.body)
    .execute(&mut *rls_tx.tx)
    .await?;

    // Update conversation updated_at
    sqlx::query("UPDATE conversations SET updated_at = NOW() WHERE id = $1")
        .bind(conversation_id)
        .execute(&mut *rls_tx.tx)
        .await?;

    // Add a system event for client notification
    let client_id: Option<Uuid> = sqlx::query_scalar(
        "SELECT user_id FROM conversation_participants WHERE conversation_id = $1 AND user_id != $2"
    )
    .bind(conversation_id)
    .bind(afrah_id)
    .fetch_optional(&mut *rls_tx.tx)
    .await?;

    if let Some(uid) = client_id {
        sqlx::query(
            "INSERT INTO system_events (user_id, event_type, message_en, message_ar) VALUES ($1, 'concierge_reply', $2, $3)"
        )
        .bind(uid)
        .bind("Afrah has replied to your concierge request.")
        .bind("قامت أفراح بالرد على طلبك.")
        .execute(&mut *rls_tx.tx)
        .await?;
    }

    rls_tx
        .tx
        .commit()
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;

    Ok(Json(json!({
        "status": "success",
        "data": { "messageId": msg_id }
    })))
}

/// GET /api/v1/admin/audit/performance
/// Real-time telemetry dashboard for Zero-Trust Audit.
async fn get_audit_performance(
    auth: RequireSuperAdmin,
    mut rls_tx: RlsTx,
) -> Result<Json<Value>, AppError> {
    tracing::info!(
        target: "audit",
        actor_id = %auth.user_id,
        "SuperAdmin requested real-time performance telemetry"
    );

    // Aggregate telemetry from database (simulated real-time stats)
    let total_vendors: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM vendors")
        .fetch_one(&mut *rls_tx.tx)
        .await
        .unwrap_or(0);
    let active_vendors: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM vendors WHERE status = 'active'")
            .fetch_one(&mut *rls_tx.tx)
            .await
            .unwrap_or(0);

    // Inquiries telemetry
    let total_inquiries: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM client_inquiries")
        .fetch_one(&mut *rls_tx.tx)
        .await
        .unwrap_or(0);
    let inquiries_24h: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM client_inquiries WHERE event_date >= CURRENT_DATE - INTERVAL '1 day'",
    )
    .fetch_one(&mut *rls_tx.tx)
    .await
    .unwrap_or(0);

    // Audit log count
    let total_audit_logs: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM system_events")
        .fetch_one(&mut *rls_tx.tx)
        .await
        .unwrap_or(0);

    Ok(Json(json!({
        "status": "success",
        "telemetry": {
            "vendors": {
                "total": total_vendors,
                "active": active_vendors,
            },
            "inquiries": {
                "total": total_inquiries,
                "last_24h": inquiries_24h,
            },
            "system_events": {
                "total": total_audit_logs,
            },
            "timestamp": chrono::Utc::now().to_rfc3339(),
        }
    })))
}

// ─── SUBSCRIPTION REQUESTS ADMIN HANDLERS ────────────────────────────────────

#[derive(serde::Deserialize)]
pub struct OutboxFilterParams {
    pub status: Option<String>,
}

async fn list_outbox_events(
    State(state): State<AppState>,
    _auth: RequireAdmin,
    axum::extract::Query(params): axum::extract::Query<OutboxFilterParams>,
) -> Result<Json<Value>, AppError> {
    use sqlx::Row;

    let query_str = if params.status.is_some() {
        "SELECT id, event_type, aggregate_type, aggregate_id, payload, status, attempt_count, last_attempt_at, next_retry_at, created_at, updated_at, delivered_at, error_message 
         FROM notification_outbox 
         WHERE status = $1 
         ORDER BY created_at DESC"
    } else {
        "SELECT id, event_type, aggregate_type, aggregate_id, payload, status, attempt_count, last_attempt_at, next_retry_at, created_at, updated_at, delivered_at, error_message 
         FROM notification_outbox 
         ORDER BY created_at DESC"
    };

    let rows = if let Some(ref status) = params.status {
        sqlx::query(query_str)
            .bind(status)
            .fetch_all(&state.db)
            .await?
    } else {
        sqlx::query(query_str).fetch_all(&state.db).await?
    };

    let events: Vec<Value> = rows
        .into_iter()
        .map(|row| {
            let id: Uuid = row.get("id");
            let event_type: String = row.get("event_type");
            let aggregate_type: String = row.get("aggregate_type");
            let aggregate_id: Uuid = row.get("aggregate_id");
            let payload: Value = row.get("payload");
            let status: String = row.get("status");
            let attempt_count: i32 = row.get("attempt_count");
            let last_attempt_at: Option<chrono::DateTime<chrono::Utc>> = row.get("last_attempt_at");
            let next_retry_at: chrono::DateTime<chrono::Utc> = row.get("next_retry_at");
            let created_at: chrono::DateTime<chrono::Utc> = row.get("created_at");
            let updated_at: chrono::DateTime<chrono::Utc> = row.get("updated_at");
            let delivered_at: Option<chrono::DateTime<chrono::Utc>> = row.get("delivered_at");
            let error_message: Option<String> = row.get("error_message");

            json!({
                "id": id,
                "event_type": event_type,
                "aggregate_type": aggregate_type,
                "aggregate_id": aggregate_id,
                "payload": payload,
                "status": status,
                "attempt_count": attempt_count,
                "last_attempt_at": last_attempt_at,
                "next_retry_at": next_retry_at,
                "created_at": created_at,
                "updated_at": updated_at,
                "delivered_at": delivered_at,
                "error_message": error_message
            })
        })
        .collect();

    Ok(Json(json!({
        "status": "success",
        "events": events
    })))
}

async fn get_outbox_metrics(_auth: RequireAdmin) -> Result<Json<Value>, AppError> {
    use crate::services::outbox_worker::METRICS;
    use std::sync::atomic::Ordering;

    let processed = METRICS.total_processed.load(Ordering::Relaxed);
    let success = METRICS.total_success.load(Ordering::Relaxed);
    let failures = METRICS.total_failures.load(Ordering::Relaxed);
    let retries = METRICS.total_retries.load(Ordering::Relaxed);

    let email_latency_sum = METRICS.email_latency_sum_ms.load(Ordering::Relaxed);
    let email_count = METRICS.email_count.load(Ordering::Relaxed);
    let avg_email_latency_ms = if email_count > 0 {
        email_latency_sum as f64 / email_count as f64
    } else {
        0.0
    };

    let whatsapp_latency_sum = METRICS.whatsapp_latency_sum_ms.load(Ordering::Relaxed);
    let whatsapp_count = METRICS.whatsapp_count.load(Ordering::Relaxed);
    let avg_whatsapp_latency_ms = if whatsapp_count > 0 {
        whatsapp_latency_sum as f64 / whatsapp_count as f64
    } else {
        0.0
    };

    Ok(Json(json!({
        "status": "success",
        "metrics": {
            "total_processed": processed,
            "total_success": success,
            "total_failures": failures,
            "total_retries": retries,
            "avg_email_latency_ms": avg_email_latency_ms,
            "avg_whatsapp_latency_ms": avg_whatsapp_latency_ms
        }
    })))
}

// ─── BFF ENDPOINT: GET /api/v1/admin/vendors-context ─────────────────────────
