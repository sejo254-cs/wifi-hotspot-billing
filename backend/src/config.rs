use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    // Database
    pub database_url: String,

    // Server
    pub axum_host: String,
    pub axum_port: u16,

    // RADIUS
    pub radius_host: String,
    pub radius_port: u16,
    pub radius_accounting_port: u16,
    pub radius_secret: String,

    // Admin
    pub admin_username: String,
    pub admin_password_hash: String,

    // M-Pesa
    pub mpesa_consumer_key: String,
    pub mpesa_consumer_secret: String,
    pub mpesa_business_short_code: String,
    pub mpesa_account_number: String,
    pub mpesa_passkey: String,
    pub mpesa_api_url: String,

    // Callbacks
    pub mpesa_callback_url: String,

    // Security
    pub jwt_secret: String,
    pub session_timeout_mins: u32,

    // OTP
    pub otp_expiry_secs: u32,
    pub otp_retry_limit: u32,

    // Cloudflare
    pub cloudflare_tunnel_url: String,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Config {
            database_url: env::var("DATABASE_URL")?,
            axum_host: env::var("AXUM_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            axum_port: env::var("AXUM_PORT")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(8080),
            radius_host: env::var("RADIUS_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            radius_port: env::var("RADIUS_PORT")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(1812),
            radius_accounting_port: env::var("RADIUS_ACCOUNTING_PORT")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(1813),
            radius_secret: env::var("RADIUS_SECRET")?,
            admin_username: env::var("ADMIN_USERNAME").unwrap_or_else(|_| "admin".to_string()),
            admin_password_hash: env::var("ADMIN_PASSWORD_HASH")?,
            mpesa_consumer_key: env::var("MPESA_CONSUMER_KEY")?,
            mpesa_consumer_secret: env::var("MPESA_CONSUMER_SECRET")?,
            mpesa_business_short_code: env::var("MPESA_BUSINESS_SHORT_CODE")?,
            mpesa_account_number: env::var("MPESA_ACCOUNT_NUMBER")?,
            mpesa_passkey: env::var("MPESA_PASSKEY")?,
            mpesa_api_url: env::var("MPESA_API_URL")?,
            mpesa_callback_url: env::var("MPESA_CALLBACK_URL")?,
            jwt_secret: env::var("JWT_SECRET")?,
            session_timeout_mins: env::var("SESSION_TIMEOUT_MINS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(1440),
            otp_expiry_secs: env::var("OTP_EXPIRY_SECS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(300),
            otp_retry_limit: env::var("OTP_RETRY_LIMIT")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(3),
            cloudflare_tunnel_url: env::var("CLOUDFLARE_TUNNEL_URL")
                .unwrap_or_else(|_| "https://admin.your-tunnel.trycloudflare.com".to_string()),
        })
    }
}
