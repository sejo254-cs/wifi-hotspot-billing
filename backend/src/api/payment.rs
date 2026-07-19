use crate::error::AppError;
use crate::models::*;
use crate::AppState;
use axum::{extract::Path, extract::State, http::HeaderMap, Json};
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

pub async fn initiate_payment(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<InitiatePaymentRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let user_id = extract_user_id(&headers)?;

    // Verify package exists
    let package = crate::db::get_package(&state.pool, payload.package_id)
        .await?
        .ok_or(AppError::PackageNotFound)?;

    // Create transaction
    let transaction = crate::db::create_transaction(
        &state.pool,
        user_id,
        &payload.phone,
        payload.package_id,
        package.price_ksh,
    )
    .await?;

    // Initiate M-Pesa payment (STK Push)
    // This is a placeholder - implement actual M-Pesa integration
    tracing::info!(
        "Initiating M-Pesa STK push for {} KSH to {}",
        package.price_ksh,
        payload.phone
    );

    Ok(Json(json!({
        "transaction_id": transaction.id,
        "status": "pending",
        "amount": package.price_ksh,
        "message": "STK push sent to your phone"
    })))
}

pub async fn payment_status(
    State(state): State<Arc<AppState>>,
    Path(transaction_id): Path<Uuid>,
) -> Result<Json<PaymentStatusResponse>, AppError> {
    let transaction = crate::db::get_transaction(&state.pool, transaction_id)
        .await?
        .ok_or(AppError::PaymentNotFound)?;

    Ok(Json(PaymentStatusResponse {
        transaction_id: transaction.id,
        status: transaction.status,
        amount: transaction.amount_ksh,
    }))
}

pub async fn mpesa_callback(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<MpesaCallbackRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    // Parse M-Pesa callback
    if let Some(stk) = &payload.body.stk_callback {
        tracing::info!("M-Pesa callback: ResultCode={}", stk.result_code);

        if stk.result_code == 0 {
            // Payment successful
            if let Some(metadata) = &stk.callback_metadata {
                for item in &metadata.item {
                    if item.name == "Amount" {
                        tracing::info!("Payment amount: {}", item.value);
                    }
                }
            }
        }
    }

    Ok(Json(json!({
        "ResultCode": 0,
        "ResultDesc": "Accepted"
    })))
}
