use crate::errors::AppError;
use crate::middleware::auth::{RequireAdmin, RequireSuperAdmin, RlsTx};
use crate::state::AppState;
use crate::models::user::DomainType;
use axum::{
    extract::{Path, State},
    routing::{get, post, patch},
    Json, Router,
};
use serde_json::{json, Value};
use uuid::Uuid;
use sqlx::Row;
use serde::Deserialize;
use validator::Validate;
use crate::utils::crypto::hash_password;

#[derive(serde::Deserialize)]
pub struct AdminUsersQuery {
    pub page: Option<i64>,
    pub limit: Option<i64>,
    pub search: Option<String>,
}

#[derive(serde::Deserialize, Validate)]
pub struct CreateAdminInput {
    #[validate(length(min = 2, max = 100))]
    pub first_name: String,
    #[validate(length(min = 2, max = 100))]
    pub last_name: String,
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
    #[validate(length(min = 8, max = 20))]
    pub phone: String,
    #[validate(length(min = 8, message = "Password must be at least 8 characters long"))]
    pub password: String,
    pub role: String, // "admin" | "vendor" | "client"
}

#[derive(serde::Deserialize)]
pub struct UpdateUserStatusInput {
    pub status: String,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/users/create", post(create_admin_user))
        .route("/users", get(list_users))
        .route("/users/:id/status", patch(update_admin_user_status))
}

async fn create_admin_user(
    mut rls_tx: RlsTx,
    auth: RequireSuperAdmin,
    Json(payload): Json<CreateAdminInput>,
) -> Result<Json<Value>, AppError> {
    payload.validate()?;

    // Centralized adaptive password validation
    crate::utils::validation::validate_password(&payload.password)?;

    // 1. Hash password securely using bcrypt on blocking thread pool
    // NOTE: The UNIQUE constraint on global_users.email is the sole authoritative
    // duplicate gate. The legacy SELECT-before-INSERT check was a TOCTOU race and
    // has been removed. Duplicate collisions are caught by map_db_error below.
    let hashed_password = hash_password(payload.password.clone()).await?;

    // Determine domain type based on role string
    let domain_type = match payload.role.to_lowercase().as_str() {
        "admin" => crate::models::user::DomainType::Admin,
        "vendor" => crate::models::user::DomainType::Vendor,
        "client" => crate::models::user::DomainType::Client,
        _ => {
            return Err(AppError::BadRequest(format!(
                "Invalid role '{}'. Must be 'admin', 'vendor', or 'client'",
                payload.role
            )))
        }
    };

    let user_id: Uuid = sqlx::query_scalar(
        "INSERT INTO global_users (email, password_hash, domain_type) VALUES ($1, $2, $3) RETURNING id"
    )
    .bind(&payload.email)
    .bind(&hashed_password)
    .bind(domain_type)
    .fetch_one(&mut *rls_tx.tx)
    .await
    .map_err(crate::errors::map_db_error)?;

    match domain_type {
        crate::models::user::DomainType::Client => {
            sqlx::query(
                "INSERT INTO client_profiles (client_id, first_name, last_name, phone) VALUES ($1, $2, $3, $4)"
            )
            .bind(user_id)
            .bind(&payload.first_name)
            .bind(&payload.last_name)
            .bind(&payload.phone)
            .execute(&mut *rls_tx.tx)
            .await
            .map_err(crate::errors::map_db_error)?;
        }
        crate::models::user::DomainType::Vendor => {
            let base_slug = format!(
                "{}-{}",
                payload.first_name.to_lowercase().replace(' ', "-"),
                &Uuid::new_v4().to_string()[..8]
            );
            sqlx::query(
                "INSERT INTO vendors (user_id, name_ar, name_en, slug, status, phone, email) VALUES ($1, $2, $3, $4, 'active', $5, $6)"
            )
            .bind(user_id)
            .bind(&payload.first_name)
            .bind(format!("{} {}", payload.first_name, payload.last_name))
            .bind(&base_slug)
            .bind(&payload.phone)
            .bind(&payload.email)
            .execute(&mut *rls_tx.tx)
            .await
            .map_err(crate::errors::map_db_error)?;
        }
        crate::models::user::DomainType::Admin => {
            sqlx::query(
                "INSERT INTO client_profiles (client_id, first_name, last_name, phone) VALUES ($1, $2, $3, $4)"
            )
            .bind(user_id)
            .bind(&payload.first_name)
            .bind(&payload.last_name)
            .bind(&payload.phone)
            .execute(&mut *rls_tx.tx)
            .await
            .map_err(crate::errors::map_db_error)?;
        }
    }

    rls_tx.tx.commit().await?;

    let domain_str = match domain_type {
        crate::models::user::DomainType::Admin => "Admin",
        crate::models::user::DomainType::Vendor => "Vendor",
        crate::models::user::DomainType::Client => "Client",
    };

    // Emit structured audit record AFTER commit — captures the creating actor,
    // the new user's ID, their email, and the assigned domain role.
    tracing::info!(
        target: "audit",
        actor_id       = %auth.user_id,
        event          = "admin_user_provisioned",
        new_user_id    = %user_id,
        new_user_email = %payload.email,
        assigned_role  = %domain_str,
        "Admin user provisioning committed"
    );

    Ok(Json(json!({
        "status": "success",
        "message": "User provisioned successfully by Admin",
        "user": {
            "id": user_id.to_string(),
            "email": payload.email,
            "role": domain_str
        }
    })))
}


async fn list_users(
    _auth: RequireAdmin,
    mut rls_tx: RlsTx,
    axum::extract::Query(query): axum::extract::Query<AdminUsersQuery>,
) -> Result<Json<Value>, AppError> {
    tracing::info!("Admin listing global users...");

    let page = query.page.unwrap_or(1).max(1);
    let limit = query.limit.unwrap_or(10).clamp(1, 100);
    let offset = (page - 1) * limit;

    // 0. Query statistics
    let stats_row = sqlx::query(
        "SELECT 
            COUNT(*)::bigint as total_users,
            COUNT(*) FILTER (WHERE domain_type = 'Client')::bigint as total_clients,
            COUNT(*) FILTER (WHERE domain_type = 'Vendor')::bigint as total_vendors,
            COUNT(*) FILTER (WHERE domain_type = 'Admin')::bigint as total_admins,
            COUNT(*) FILTER (WHERE created_at >= DATE_TRUNC('month', CURRENT_DATE))::bigint as new_users_this_month
         FROM global_users"
    )
    .fetch_one(&mut *rls_tx.tx)
    .await?;

    let total_users: i64 = stats_row.get("total_users");
    let total_clients: i64 = stats_row.get("total_clients");
    let total_vendors: i64 = stats_row.get("total_vendors");
    let total_admins: i64 = stats_row.get("total_admins");
    let new_users_this_month: i64 = stats_row.get("new_users_this_month");

    let mut query_builder = String::from(
        "FROM global_users u
         LEFT JOIN client_profiles cp ON u.id = cp.client_id
         LEFT JOIN vendors v ON u.id = v.user_id
         WHERE 1=1",
    );

    let search_term = query.search.as_deref().unwrap_or("").trim();
    let has_search = !search_term.is_empty();

    if has_search {
        query_builder.push_str(" AND (u.email ILIKE $1 OR cp.first_name ILIKE $1 OR cp.last_name ILIKE $1 OR cp.phone ILIKE $1 OR v.name_en ILIKE $1 OR v.name_ar ILIKE $1 OR v.phone ILIKE $1)");
    }

    // 1. Count total matching records
    let count_query = format!("SELECT COUNT(*)::bigint {}", query_builder);
    let total_count: i64 = if has_search {
        sqlx::query_scalar(&count_query)
            .bind(format!("%{}%", search_term))
            .fetch_one(&mut *rls_tx.tx)
            .await?
    } else {
        sqlx::query_scalar(&count_query)
            .fetch_one(&mut *rls_tx.tx)
            .await?
    };

    // 2. Query paginated results
    let select_query = format!(
        "SELECT 
            u.id, u.email, u.domain_type::text AS domain_type, u.created_at,
            cp.first_name AS client_first_name, cp.last_name AS client_last_name, cp.phone AS client_phone, cp.wedding_date,
            v.name_en AS vendor_name_en, v.name_ar AS vendor_name_ar, v.phone AS vendor_phone,
            (SELECT COUNT(*) FROM core_bookings cb WHERE cb.client_id = u.id OR cb.vendor_id = u.id)::bigint AS bookings_count
         {}
         ORDER BY u.created_at DESC
         LIMIT ${} OFFSET ${}",
         query_builder,
         if has_search { 2 } else { 1 },
         if has_search { 3 } else { 2 }
    );

    let rows = if has_search {
        sqlx::query(&select_query)
            .bind(format!("%{}%", search_term))
            .bind(limit)
            .bind(offset)
            .fetch_all(&mut *rls_tx.tx)
            .await?
    } else {
        sqlx::query(&select_query)
            .bind(limit)
            .bind(offset)
            .fetch_all(&mut *rls_tx.tx)
            .await?
    };

    let mut users_list = Vec::new();
    for row in rows {
        let id: Uuid = row.get("id");
        let email: String = row.get::<Option<String>, _>("email").unwrap_or_default();
        let domain_type: String = row.get("domain_type");
        let created_at: chrono::DateTime<chrono::Utc> = row.get("created_at");
        let client_first_name: Option<String> = row.get("client_first_name");
        let client_last_name: Option<String> = row.get("client_last_name");
        let client_phone: Option<String> = row.get("client_phone");
        let wedding_date: Option<chrono::NaiveDate> = row.get("wedding_date");
        let vendor_name_en: Option<String> = row.get("vendor_name_en");
        let vendor_name_ar: Option<String> = row.get("vendor_name_ar");
        let vendor_phone: Option<String> = row.get("vendor_phone");
        let bookings_count: i64 = row.get("bookings_count");

        let (first_name, last_name, phone) = match domain_type.as_str() {
            "Vendor" => (
                vendor_name_en.clone().unwrap_or_default(),
                vendor_name_ar.clone().unwrap_or_default(),
                vendor_phone.clone().unwrap_or_default(),
            ),
            _ => (
                client_first_name.clone().unwrap_or_default(),
                client_last_name.clone().unwrap_or_default(),
                client_phone.clone().unwrap_or_default(),
            ),
        };

        users_list.push(json!({
            "id": id.to_string(),
            "email": email,
            "domain_type": domain_type,
            "first_name": first_name,
            "last_name": last_name,
            "phone": phone,
            "wedding_date": wedding_date.map(|d| d.to_string()),
            "status": "active",
            "bookings_count": bookings_count,
            "created_at": created_at.to_rfc3339()
        }));
    }

    rls_tx.tx.commit().await?;

    let total_pages = ((total_count + limit - 1) / limit).max(1);

    Ok(Json(json!({
        "status": "success",
        "users": users_list,
        "total": total_count,
        "page": page,
        "totalPages": total_pages,
        "stats": {
            "totalUsers": total_users,
            "totalClients": total_clients,
            "totalVendors": total_vendors,
            "totalAdmins": total_admins,
            "newUsersThisMonth": new_users_this_month
        }
    })))
}


async fn update_admin_user_status(
    mut rls_tx: RlsTx,
    auth: RequireSuperAdmin,
    Path(id): Path<String>,
    Json(input): Json<UpdateUserStatusInput>,
) -> Result<Json<Value>, AppError> {
    let target_uuid = Uuid::parse_str(&id)
        .map_err(|_| AppError::BadRequest("Invalid user UUID format".to_string()))?;

    let admin_uuid = Uuid::parse_str(&auth.user_id)
        .map_err(|_| AppError::BadRequest("Invalid admin user ID".to_string()))?;

    let allowed_statuses = ["active", "suspended", "banned", "pending"];
    if !allowed_statuses.contains(&input.status.as_str()) {
        return Err(AppError::BadRequest(format!(
            "Invalid user status '{}'. Must be one of: {}",
            input.status,
            allowed_statuses.join(", ")
        )));
    }

    let user_row = sqlx::query("SELECT email, domain_type, status FROM users WHERE id = $1")
        .bind(target_uuid)
        .fetch_optional(&mut *rls_tx.tx)
        .await?;

    let user = match user_row {
        Some(row) => row,
        None => return Err(AppError::NotFound("User not found".to_string())),
    };

    let user_email: String = user.get::<Option<String>, _>("email").unwrap_or_default();
    let current_status: String = user.get("status");
    let domain_type_str: String = user.get("domain_type");

    if current_status == input.status {
        return Err(AppError::BadRequest(format!(
            "User is already {}",
            input.status
        )));
    }

    let result =
        sqlx::query("UPDATE users SET status = $1, updated_at = CURRENT_TIMESTAMP WHERE id = $2")
            .bind(&input.status)
            .bind(target_uuid)
            .execute(&mut *rls_tx.tx)
            .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(
            "Failed to update user status".to_string(),
        ));
    }

    let mut vendor_uuid: Option<Uuid> = None;
    let mut client_uuid: Option<Uuid> = None;
    if domain_type_str == "Client" {
        client_uuid = Some(target_uuid);
    } else if domain_type_str == "Vendor" {
        if let Ok(Some(row)) = sqlx::query("SELECT id FROM vendors WHERE user_id = $1")
            .bind(target_uuid)
            .fetch_optional(&mut *rls_tx.tx)
            .await
        {
            vendor_uuid = Some(row.get("id"));
        }
    }

    // Status transition history
    sqlx::query(
        "INSERT INTO status_history (entity_type, entity_id, old_status, new_status, changed_by, reason, vendor_id, client_id)
         VALUES ('user', $1, $2, $3, $4, 'User status updated by admin', $5, $6)",
    )
    .bind(target_uuid)
    .bind(&current_status)
    .bind(&input.status)
    .bind(admin_uuid)
    .bind(vendor_uuid)
    .bind(client_uuid)
    .execute(&mut *rls_tx.tx)
    .await?;

    // Admin audit log
    sqlx::query(
        "INSERT INTO admin_audit_logs (entity_type, entity_id, actor_id, action, before_state, after_state)
         VALUES ('user', $1, $2, 'update_user_status', $3, $4)",
    )
    .bind(target_uuid)
    .bind(admin_uuid)
    .bind(json!({ "status": current_status }))
    .bind(json!({ "status": input.status }))
    .execute(&mut *rls_tx.tx)
    .await?;

    sqlx::query(
        "INSERT INTO system_events (user_id, event_type, message_en, message_ar)
         VALUES ($1, 'user_status_changed', $2, $3)",
    )
    .bind(admin_uuid)
    .bind(format!(
        "Admin updated user {} status from '{}' to '{}'",
        user_email, current_status, input.status
    ))
    .bind(format!(
        "قام المسؤول بتغيير حالة المستخدم {} من '{}' إلى '{}'",
        user_email, current_status, input.status
    ))
    .execute(&mut *rls_tx.tx)
    .await?;

    rls_tx.tx.commit().await?;

    Ok(Json(json!({
        "status": "success",
        "message": format!("User status updated to '{}'", input.status),
        "new_status": input.status
    })))
}

