use crate::error::AppError;
use crate::models::*;
use crate::AppState;
use axum::{extract::State, http::HeaderMap, Json};
use chrono::Utc;
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

fn extract_user_id(headers: &HeaderMap) -> Result<Uuid, AppError> {
    let auth_header = headers
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .ok_or(AppError::Unauthorized)?;

    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or(AppError::Unauthorized)?;

    let user_id_str = token.split('.').next().ok_or(AppError::Unauthorized)?;
    Uuid::parse_str(user_id_str).map_err(|_| AppError::Unauthorized)
}

pub async fn quota_status(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<QuotaResponse>, AppError> {
    let user_id = extract_user_id(&headers)?;

    let quota = crate::db::get_active_quota(&state.pool, user_id)
        .await?
        .ok_or(AppError::QuotaNotFound)?;

    let package = crate::db::get_package(&state.pool, quota.package_id)
        .await?
        .ok_or(AppError::PackageNotFound)?;

    let remaining_time = if let Some(limit) = package.time_limit_hours {
        let total_seconds = (limit as i64) * 3600;
        let remaining_seconds = total_seconds - quota.time_used_seconds;
        if remaining_seconds > 0 {
            Some((remaining_seconds / 3600) as i32)
        } else {
            Some(0)
        }
    } else {
        None
    };

    let status = if Utc::now() > quota.expires_at {
        "expired".to_string()
    } else {
        quota.status.clone()
    };

    Ok(Json(QuotaResponse {
        remaining_time_hours: remaining_time,
        used_bandwidth_gb: quota.bandwidth_used_gb,
        status,
        expires_at: quota.expires_at,
    }))
}
