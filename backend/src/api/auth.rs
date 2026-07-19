use crate::error::AppError;
use crate::models::*;
use crate::utils;
use crate::AppState;
use axum::{extract::State, Json};
use std::sync::Arc;
use uuid::Uuid;

pub async fn signup(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<SignupRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    // Validate inputs
    utils::validate_username(&payload.username)?;
    utils::validate_password(&payload.password)?;
    utils::validate_phone(&payload.phone)?;

    let normalized_phone = utils::normalize_phone(&payload.phone)?;

    // Check if user exists
    if utils::get_user_by_username(&state.pool, &payload.username)
        .await?
        .is_some()
    {
        return Err(AppError::UserExists);
    }

    // Hash password
    let password_hash = utils::hash_password(&payload.password)?;

    // Create user
    let user = crate::db::create_user(
        &state.pool,
        &payload.username,
        &password_hash,
        &normalized_phone,
    )
    .await?;

    // Add phone number
    crate::db::add_phone_number(&state.pool, user.id, &normalized_phone).await?;

    // Generate OTP
    let otp_code = utils::generate_otp();
    let _otp = crate::db::create_otp(&state.pool, user.id, &otp_code, state.config.otp_expiry_secs).await?;

    tracing::info!("OTP for user {}: {} (placeholder - SMS not configured)", user.username, otp_code);

    Ok(Json(AuthResponse {
        token: String::new(), // OTP required first
        user_id: user.id,
        username: user.username,
        requires_otp: true,
    }))
}

pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    // Validate MAC address
    utils::validate_mac_address(&payload.mac_address)?;
    let normalized_mac = utils::normalize_mac_address(&payload.mac_address)?;

    // Get device
    let device = crate::db::get_device_by_mac(&state.pool, &normalized_mac)
        .await?
        .ok_or(AppError::DeviceNotFound)?;

    // Get user
    let user = crate::db::get_user_by_id(&state.pool, device.user_id)
        .await?
        .ok_or(AppError::UserNotFound)?;

    // Verify password
    if !utils::verify_password(&payload.password, &user.password_hash)? {
        return Err(AppError::InvalidCredentials);
    }

    // Check quota status
    let quota = crate::db::get_active_quota(&state.pool, user.id).await?;
    
    if quota.is_none() {
        return Err(AppError::QuotaExhausted);
    }

    // Generate token
    let token = utils::generate_jwt(&user.id, &state.config.jwt_secret)?;

    Ok(Json(AuthResponse {
        token,
        user_id: user.id,
        username: user.username,
        requires_otp: false,
    }))
}

pub async fn verify_otp(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<VerifyOtpRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    // Get user
    let user = crate::db::get_user_by_id(&state.pool, payload.user_id)
        .await?
        .ok_or(AppError::UserNotFound)?;

    // Get latest OTP
    let otp = crate::db::get_latest_otp(&state.pool, payload.user_id)
        .await?
        .ok_or(AppError::OtpInvalid)?;

    // Verify OTP
    crate::db::verify_otp(&state.pool, otp.id, &payload.otp).await?;

    // Generate token
    let token = utils::generate_jwt(&user.id, &state.config.jwt_secret)?;

    Ok(Json(serde_json::json!({
        "token": token,
        "user_id": user.id,
        "username": user.username,
    })))
}

pub async fn logout() -> Result<Json<serde_json::Value>, AppError> {
    // In a real app, invalidate the token in a blacklist
    Ok(Json(serde_json::json!({
        "message": "Logged out successfully"
    })))
}
