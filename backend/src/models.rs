use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// User models
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub password_hash: String,
    pub default_phone: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_active: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SignupRequest {
    pub username: String,
    pub phone: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    pub mac_address: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthResponse {
    pub token: String,
    pub user_id: Uuid,
    pub username: String,
    pub requires_otp: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VerifyOtpRequest {
    pub user_id: Uuid,
    pub otp: String,
}

// Device models
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Device {
    pub id: Uuid,
    pub user_id: Uuid,
    pub mac_address: String,
    pub device_name: Option<String>,
    pub created_at: DateTime<Utc>,
    pub is_active: bool,
}

// Phone number models
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PhoneNumber {
    pub id: Uuid,
    pub user_id: Uuid,
    pub phone: String,
    pub is_default: bool,
    pub verified: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AddPhoneRequest {
    pub phone: String,
}

// Package models
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Package {
    pub id: Uuid,
    pub name: String,
    pub price_ksh: i32,
    pub upload_speed_mbps: i32,
    pub download_speed_mbps: i32,
    pub time_limit_hours: Option<i32>,
    pub bandwidth_limit_gb: Option<i32>,
    pub description: String,
    pub concurrent_users: i32,
    pub quota_type: String, // "time_usage", "time_wall_clock", "bandwidth"
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreatePackageRequest {
    pub name: String,
    pub price_ksh: i32,
    pub upload_speed_mbps: i32,
    pub download_speed_mbps: i32,
    pub time_limit_hours: Option<i32>,
    pub bandwidth_limit_gb: Option<i32>,
    pub description: String,
    pub concurrent_users: i32,
    pub quota_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PackageResponse {
    pub id: Uuid,
    pub name: String,
    pub price_ksh: i32,
    pub time_limit_hours: Option<i32>,
    pub description: String,
}

// Quota models
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Quota {
    pub id: Uuid,
    pub user_id: Uuid,
    pub package_id: Uuid,
    pub device_id: Option<Uuid>,
    pub purchased_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub time_used_seconds: i64,
    pub bandwidth_used_gb: f64,
    pub status: String, // "active", "expired", "suspended"
    pub last_activity: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QuotaResponse {
    pub remaining_time_hours: Option<i32>,
    pub used_bandwidth_gb: f64,
    pub status: String,
    pub expires_at: DateTime<Utc>,
}

// Transaction models
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Transaction {
    pub id: Uuid,
    pub user_id: Uuid,
    pub phone: String,
    pub package_id: Uuid,
    pub amount_ksh: i32,
    pub status: String, // "pending", "completed", "failed"
    pub mpesa_transaction_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InitiatePaymentRequest {
    pub package_id: Uuid,
    pub phone: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PaymentStatusResponse {
    pub transaction_id: Uuid,
    pub status: String,
    pub amount: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MpesaCallbackRequest {
    #[serde(rename = "Body")]
    pub body: MpesaCallbackBody,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MpesaCallbackBody {
    #[serde(rename = "stkCallback")]
    pub stk_callback: Option<MpesaStkCallback>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MpesaStkCallback {
    #[serde(rename = "MerchantRequestID")]
    pub merchant_request_id: String,
    #[serde(rename = "CheckoutRequestID")]
    pub checkout_request_id: String,
    #[serde(rename = "ResultCode")]
    pub result_code: i32,
    #[serde(rename = "ResultDesc")]
    pub result_desc: String,
    #[serde(rename = "CallbackMetadata")]
    pub callback_metadata: Option<MpesaCallbackMetadata>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MpesaCallbackMetadata {
    #[serde(rename = "Item")]
    pub item: Vec<MpesaCallbackItem>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MpesaCallbackItem {
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Value")]
    pub value: serde_json::Value,
}

// OTP models
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct OtpRecord {
    pub id: Uuid,
    pub user_id: Uuid,
    pub otp_code: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub attempts: i32,
    pub is_verified: bool,
}

// Admin models
#[derive(Debug, Serialize, Deserialize)]
pub struct AdminLoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AdminStatsResponse {
    pub total_users: i64,
    pub active_sessions: i64,
    pub monthly_revenue_ksh: i64,
    pub total_bandwidth_gb: f64,
}

// RADIUS models
#[derive(Debug, Clone)]
pub struct RadiusSession {
    pub session_id: String,
    pub mac_address: String,
    pub user_id: Option<Uuid>,
    pub phone: Option<String>,
    pub quota_id: Option<Uuid>,
    pub authenticated: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RadiusAttributeRequest {
    pub user_name: String,
    pub mac_address: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RadiusAttributeResponse {
    pub upload_speed_kbps: u32,
    pub download_speed_kbps: u32,
    pub session_timeout: u32,
}
