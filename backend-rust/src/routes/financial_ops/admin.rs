use crate::errors::AppError;
use crate::middleware::auth::{RequireAdmin, RequireSuperAdmin, RlsTx};
use crate::state::AppState;
use axum::{
    extract::Path,
    routing::{get, post},
    Json, Router,
};
use serde_json::{json, Value};
use uuid::Uuid;
use sqlx::Row;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/finance/commissions", get(get_finance_commissions))
        .route("/finance/summary", get(get_admin_finance_summary))
        .route("/payouts", get(list_admin_payout_requests))
        .route("/payouts/:id/approve", post(approve_admin_payout_request))
}

async fn get_finance_commissions(
    _auth: RequireSuperAdmin,
    mut rls_tx: RlsTx,
) -> Result<Json<Value>, AppError> {
    tracing::info!("Fetching finance commissions ledger...");

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

    let ledger_rows = sqlx::query(
        "SELECT \
            v.id AS vendor_id, \
            v.name_en AS vendor_name_en, \
            v.name_ar AS vendor_name_ar, \
            p.product_category AS product_category, \
            COUNT(b.id)::bigint AS bookings_count, \
            COALESCE(SUM(b.total_price), 0.00)::float8 AS total_revenue, \
            COUNT(CASE WHEN b.status IN ('pending', 'Draft_Inquiry', 'Pending_Vendor_Acceptance') THEN 1 END)::bigint AS pending_count \
         FROM vendors v \
         LEFT JOIN vendor_products p ON p.vendor_id = v.id \
         LEFT JOIN core_bookings b ON b.product_id = p.id \
         GROUP BY v.id, p.product_category"
    )
    .fetch_all(&mut *rls_tx.tx)
    .await?;

    let mut commissions = Vec::new();
    let mut total_earned = 0.0;
    let mut total_paid = 0.0;
    let mut total_pending = 0.0;

    for row in ledger_rows {
        let vendor_id: Uuid = row.get("vendor_id");
        let vendor_name_en: String = row.get("vendor_name_en");
        let vendor_name_ar: String = row.get("vendor_name_ar");
        let product_category: Option<String> = row.get("product_category");
        let bookings_count: i64 = row.get("bookings_count");
        let total_revenue: f64 = row.get("total_revenue");
        let pending_count: i64 = row.get("pending_count");

        let commission_earned = total_revenue * commission_fraction;
        let payout_status = if pending_count > 0 { "pending" } else { "paid" };

        total_earned += commission_earned;
        if payout_status == "paid" {
            total_paid += commission_earned;
        } else {
            total_pending += commission_earned;
        }

        commissions.push(json!({
            "vendor_id": vendor_id.to_string(),
            "vendor_name_en": vendor_name_en,
            "vendor_name_ar": vendor_name_ar,
            "product_category": product_category.unwrap_or_default(),
            "bookings_count": bookings_count,
            "total_revenue": total_revenue,
            "commission_rate": commission_rate,
            "commission_earned": commission_earned,
            "payout_status": payout_status
        }));
    }

    rls_tx.tx.commit().await?;

    Ok(Json(json!({
        "status": "success",
        "summary": {
            "total_earned": total_earned,
            "total_paid": total_paid,
            "total_pending": total_pending
        },
        "commissions": commissions
    })))
}

async fn get_admin_finance_summary(
    _auth: RequireAdmin,
    mut rls_tx: RlsTx,
) -> Result<Json<Value>, AppError> {
    let row = sqlx::query(
        "SELECT \
            COALESCE(SUM(amount), 0.0)::float8 AS gross_volume, \
            COALESCE(SUM(tax_amount), 0.0)::float8 AS tax_collected \
         FROM invoices WHERE status = 'Paid'",
    )
    .fetch_one(&mut *rls_tx.tx)
    .await?;

    let gross_volume: f64 = row.get("gross_volume");
    let tax_collected: f64 = row.get("tax_collected");

    let escrow_row = sqlx::query(
        "SELECT COALESCE(SUM(amount_held), 0.0)::float8 AS total_escrow FROM escrow_accounts WHERE status = 'Held'"
    )
    .fetch_one(&mut *rls_tx.tx)
    .await?;

    let total_escrow: f64 = escrow_row.get("total_escrow");

    rls_tx.tx.commit().await?;

    Ok(Json(json!({
        "status": "success",
        "data": {
            "grossVolume": gross_volume,
            "taxCollected": tax_collected,
            "totalEscrowHeld": total_escrow,
            "netPlatformEarnings": gross_volume * 0.10
        }
    })))
}

async fn list_admin_payout_requests(
    _auth: RequireAdmin,
    mut rls_tx: RlsTx,
) -> Result<Json<Value>, AppError> {
    let rows = sqlx::query(
        "SELECT p.id, p.vendor_id, p.amount::float8, p.status, p.bank_name, p.iban, p.created_at::text, v.name_ar AS vendor_name_ar, v.name_en AS vendor_name_en \
         FROM payout_requests p \
         JOIN vendors v ON p.vendor_id = v.id \
         ORDER BY p.created_at DESC"
    )
    .fetch_all(&mut *rls_tx.tx)
    .await?;

    let payouts: Vec<Value> = rows
        .iter()
        .map(|r| {
            json!({
                "id": r.get::<Uuid, _>("id").to_string(),
                "vendorId": r.get::<Uuid, _>("vendor_id").to_string(),
                "vendorNameAr": r.get::<String, _>("vendor_name_ar"),
                "vendorNameEn": r.get::<String, _>("vendor_name_en"),
                "amount": r.get::<f64, _>("amount"),
                "status": r.get::<String, _>("status"),
                "bankName": r.get::<Option<String>, _>("bank_name"),
                "iban": r.get::<Option<String>, _>("iban"),
                "createdAt": r.get::<String, _>("created_at"),
            })
        })
        .collect();

    rls_tx.tx.commit().await?;

    Ok(Json(json!({
        "status": "success",
        "data": payouts
    })))
}

async fn approve_admin_payout_request(
    _auth: RequireAdmin,
    mut rls_tx: RlsTx,
    Path(payout_id): Path<Uuid>,
) -> Result<Json<Value>, AppError> {
    sqlx::query("UPDATE payout_requests SET status = 'Approved', updated_at = NOW() WHERE id = $1")
        .bind(payout_id)
        .execute(&mut *rls_tx.tx)
        .await?;

    rls_tx.tx.commit().await?;

    Ok(Json(json!({
        "status": "success",
        "message": "Payout request approved successfully"
    })))
}
