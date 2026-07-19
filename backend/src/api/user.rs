use crate::error::AppError;
use crate::models::*;
use crate::utils;
use crate::AppState;
use axum::{extract::State, http::HeaderMap, Json};
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

    // Parse token (implement proper JWT parsing in production)
    let user_id_str = token.split('.').next().ok_or(AppError::Unauthorized)?;
    Uuid::parse_str(user_id_str).map_err(|_| AppError::Unauthorized)
}

pub async fn get_profile(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, AppError> {
    let user_id = extract_user_id(&headers)?;

    let user = crate::db::get_user_by_id(&state.pool, user_id)
        .await?
        .ok_or(AppError::UserNotFound)?;

    Ok(Json(json!({
        "id": user.id,
        "username": user.username,
        "default_phone": user.default_phone,
        "created_at": user.created_at,
    })))
}

pub async fn list_devices(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<Vec<Device>>, AppError> {
    let user_id = extract_user_id(&headers)?;

    let devices = crate::db::list_user_devices(&state.pool, user_id).await?;

    Ok(Json(devices))
}

pub async fn list_phones(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<Vec<PhoneNumber>>, AppError> {
    let user_id = extract_user_id(&headers)?;

    let phones = crate::db::list_user_phones(&state.pool, user_id).await?;

    Ok(Json(phones))
}

pub async fn add_phone(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<AddPhoneRequest>,
) -> Result<Json<PhoneNumber>, AppError> {
    let user_id = extract_user_id(&headers)?;

    utils::validate_phone(&payload.phone)?;
    let normalized_phone = utils::normalize_phone(&payload.phone)?;

    let phone_number = crate::db::add_phone_number(&state.pool, user_id, &normalized_phone).await?;

    Ok(Json(phone_number))
}

pub async fn remove_phone(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    let user_id = extract_user_id(&headers)?;

    let phone_id = payload["phone_id"]
        .as_str()
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or(AppError::ValidationError("Invalid phone_id".to_string()))?;

    crate::db::remove_phone_number(&state.pool, phone_id, user_id).await?;

    Ok(Json(json!({ "message": "Phone number removed" })))
}

pub async fn set_default_phone(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    let user_id = extract_user_id(&headers)?;

    let phone_id = payload["phone_id"]
        .as_str()
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or(AppError::ValidationError("Invalid phone_id".to_string()))?;

    crate::db::set_default_phone(&state.pool, phone_id, user_id).await?;

    Ok(Json(json!({ "message": "Default phone set" })))
}
