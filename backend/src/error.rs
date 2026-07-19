use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use std::fmt;

#[derive(Debug)]
pub enum AppError {
    // Auth errors
    InvalidCredentials,
    InvalidPhone,
    WeakPassword,
    UserExists,
    UserNotFound,
    MaxPhoneNumbersReached,
    OtpExpired,
    OtpInvalid,
    OtpMaxRetries,
    SessionExpired,
    Unauthorized,

    // Device errors
    MacAddressInvalid,
    DeviceNotFound,
    DeviceLocked,

    // Package errors
    PackageNotFound,
    PackageInactive,
    InvalidPackageData,

    // Payment errors
    PaymentFailed,
    PaymentNotFound,
    InsufficientBalance,
    PaymentDuplicate,

    // Quota errors
    QuotaExhausted,
    QuotaNotFound,

    // Database errors
    DatabaseError(String),

    // Validation errors
    ValidationError(String),

    // External API errors
    ExternalApiError(String),

    // Internal errors
    InternalError,
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidCredentials => write!(f, "Invalid username or password"),
            Self::InvalidPhone => write!(f, "Invalid phone number format"),
            Self::WeakPassword => write!(f, "Password does not meet strength requirements"),
            Self::UserExists => write!(f, "User already exists"),
            Self::UserNotFound => write!(f, "User not found"),
            Self::MaxPhoneNumbersReached => write!(f, "Maximum phone numbers (3) reached"),
            Self::OtpExpired => write!(f, "OTP has expired"),
            Self::OtpInvalid => write!(f, "Invalid OTP"),
            Self::OtpMaxRetries => write!(f, "Maximum OTP retries exceeded"),
            Self::SessionExpired => write!(f, "Session has expired"),
            Self::Unauthorized => write!(f, "Unauthorized"),
            Self::MacAddressInvalid => write!(f, "Invalid MAC address format"),
            Self::DeviceNotFound => write!(f, "Device not found"),
            Self::DeviceLocked => write!(f, "Device is locked"),
            Self::PackageNotFound => write!(f, "Package not found"),
            Self::PackageInactive => write!(f, "Package is inactive"),
            Self::InvalidPackageData => write!(f, "Invalid package data"),
            Self::PaymentFailed => write!(f, "Payment processing failed"),
            Self::PaymentNotFound => write!(f, "Payment not found"),
            Self::InsufficientBalance => write!(f, "Insufficient balance"),
            Self::PaymentDuplicate => write!(f, "Duplicate payment detected"),
            Self::QuotaExhausted => write!(f, "Quota exhausted"),
            Self::QuotaNotFound => write!(f, "Quota not found"),
            Self::DatabaseError(msg) => write!(f, "Database error: {}", msg),
            Self::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            Self::ExternalApiError(msg) => write!(f, "External API error: {}", msg),
            Self::InternalError => write!(f, "Internal server error"),
        }
    }
}

impl std::error::Error for AppError {}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_code, message) = match self {
            AppError::InvalidCredentials => (StatusCode::UNAUTHORIZED, "INVALID_CREDENTIALS", self.to_string()),
            AppError::InvalidPhone => (StatusCode::BAD_REQUEST, "INVALID_PHONE", self.to_string()),
            AppError::WeakPassword => (StatusCode::BAD_REQUEST, "WEAK_PASSWORD", self.to_string()),
            AppError::UserExists => (StatusCode::CONFLICT, "USER_EXISTS", self.to_string()),
            AppError::UserNotFound => (StatusCode::NOT_FOUND, "USER_NOT_FOUND", self.to_string()),
            AppError::MaxPhoneNumbersReached => (StatusCode::BAD_REQUEST, "MAX_PHONES_REACHED", self.to_string()),
            AppError::OtpExpired => (StatusCode::BAD_REQUEST, "OTP_EXPIRED", self.to_string()),
            AppError::OtpInvalid => (StatusCode::BAD_REQUEST, "OTP_INVALID", self.to_string()),
            AppError::OtpMaxRetries => (StatusCode::TOO_MANY_REQUESTS, "OTP_MAX_RETRIES", self.to_string()),
            AppError::SessionExpired => (StatusCode::UNAUTHORIZED, "SESSION_EXPIRED", self.to_string()),
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, "UNAUTHORIZED", self.to_string()),
            AppError::MacAddressInvalid => (StatusCode::BAD_REQUEST, "INVALID_MAC", self.to_string()),
            AppError::DeviceNotFound => (StatusCode::NOT_FOUND, "DEVICE_NOT_FOUND", self.to_string()),
            AppError::DeviceLocked => (StatusCode::FORBIDDEN, "DEVICE_LOCKED", self.to_string()),
            AppError::PackageNotFound => (StatusCode::NOT_FOUND, "PACKAGE_NOT_FOUND", self.to_string()),
            AppError::PackageInactive => (StatusCode::BAD_REQUEST, "PACKAGE_INACTIVE", self.to_string()),
            AppError::InvalidPackageData => (StatusCode::BAD_REQUEST, "INVALID_PACKAGE", self.to_string()),
            AppError::PaymentFailed => (StatusCode::BAD_REQUEST, "PAYMENT_FAILED", self.to_string()),
            AppError::PaymentNotFound => (StatusCode::NOT_FOUND, "PAYMENT_NOT_FOUND", self.to_string()),
            AppError::InsufficientBalance => (StatusCode::BAD_REQUEST, "INSUFFICIENT_BALANCE", self.to_string()),
            AppError::PaymentDuplicate => (StatusCode::CONFLICT, "PAYMENT_DUPLICATE", self.to_string()),
            AppError::QuotaExhausted => (StatusCode::FORBIDDEN, "QUOTA_EXHAUSTED", self.to_string()),
            AppError::QuotaNotFound => (StatusCode::NOT_FOUND, "QUOTA_NOT_FOUND", self.to_string()),
            AppError::DatabaseError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "DB_ERROR", "Database error occurred".to_string()),
            AppError::ValidationError(_) => (StatusCode::BAD_REQUEST, "VALIDATION_ERROR", self.to_string()),
            AppError::ExternalApiError(_) => (StatusCode::BAD_GATEWAY, "EXTERNAL_API_ERROR", self.to_string()),
            AppError::InternalError => (StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR", self.to_string()),
        };

        tracing::error!("API Error: {} - {}", error_code, message);

        let body = Json(json!({
            "error": {
                "code": error_code,
                "message": message,
            }
        }));

        (status, body).into_response()
    }
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        tracing::error!("SQLx error: {}", err);
        AppError::DatabaseError(err.to_string())
    }
}

impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        tracing::error!("Anyhow error: {}", err);
        AppError::InternalError
    }
}
