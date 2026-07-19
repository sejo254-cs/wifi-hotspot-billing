use crate::error::AppError;
use crate::models::*;
use crate::utils;
use crate::AppState;
use axum::{extract::Path, extract::State, Json};
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<AdminLoginRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    if payload.username != state.config.admin_username {
        return Err(AppError::InvalidCredentials);
    }

    if !utils::verify_password(&payload.password, &state.config.admin_password_hash)? {
        return Err(AppError::InvalidCredentials);
    }

    let token = utils::generate_jwt(&Uuid::nil(), &state.config.jwt_secret)?;

    Ok(Json(json!({
        "token": token,
        "username": payload.username,
        "message": "Admin logged in"
    })))
}

pub async fn create_package(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreatePackageRequest>,
) -> Result<Json<Package>, AppError> {
    if payload.name.len() > 50 || payload.name.is_empty() {
        return Err(AppError::ValidationError("Invalid package name".to_string()));
    }

    if payload.price_ksh <= 0 {
        return Err(AppError::ValidationError("Price must be positive".to_string()));
    }

    let package = crate::db::create_package(&state.pool, &payload).await?;

    Ok(Json(package))
}

pub async fn update_package(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(payload): Json<CreatePackageRequest>,
) -> Result<Json<Package>, AppError> {
    let package = crate::db::update_package(&state.pool, id, &payload).await?;
    Ok(Json(package))
}

pub async fn delete_package(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    crate::db::delete_package(&state.pool, id).await?;
    Ok(Json(json!({ "message": "Package deleted" })))
}

pub async fn list_users(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<User>>, AppError> {
    let users = sqlx::query_as::<_, User>(
        "SELECT * FROM users WHERE is_active = true ORDER BY created_at DESC LIMIT 1000"
    )
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(users))
}

pub async fn list_transactions(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<Transaction>>, AppError> {
    let transactions = sqlx::query_as::<_, Transaction>(
        "SELECT * FROM transactions ORDER BY created_at DESC LIMIT 1000"
    )
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(transactions))
}

pub async fn get_stats(
    State(state): State<Arc<AppState>>,
) -> Result<Json<AdminStatsResponse>, AppError> {
    let total_users = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM users WHERE is_active = true"
    )
    .fetch_one(&state.pool)
    .await?;

    let active_sessions = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM quotas WHERE status = 'active' AND expires_at > NOW()"
    )
    .fetch_one(&state.pool)
    .await?;

    let monthly_revenue: Option<i32> = sqlx::query_scalar(
        "SELECT COALESCE(SUM(amount_ksh), 0) FROM transactions WHERE status = 'completed' AND created_at > NOW() - INTERVAL '30 days'"
    )
    .fetch_one(&state.pool)
    .await?;

    let total_bandwidth: Option<f64> = sqlx::query_scalar(
        "SELECT COALESCE(SUM(bandwidth_used_gb), 0) FROM quotas"
    )
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(AdminStatsResponse {
        total_users,
        active_sessions,
        monthly_revenue_ksh: monthly_revenue.unwrap_or(0) as i64,
        total_bandwidth_gb: total_bandwidth.unwrap_or(0.0),
    }))
}
