use crate::error::AppError;
use crate::models::*;
use chrono::{DateTime, Duration, Utc};
use sqlx::PgPool;
use uuid::Uuid;

// User operations
pub async fn create_user(
    pool: &PgPool,
    username: &str,
    password_hash: &str,
    phone: &str,
) -> Result<User, AppError> {
    let id = Uuid::new_v4();
    let now = Utc::now();

    let user = sqlx::query_as::<_, User>(
        r#"
        INSERT INTO users (id, username, password_hash, default_phone, created_at, updated_at, is_active)
        VALUES ($1, $2, $3, $4, $5, $6, true)
        RETURNING *
        "#,
    )
    .bind(&id)
    .bind(username)
    .bind(password_hash)
    .bind(phone)
    .bind(now)
    .bind(now)
    .fetch_one(pool)
    .await?;

    Ok(user)
}

pub async fn get_user_by_username(pool: &PgPool, username: &str) -> Result<Option<User>, AppError> {
    let user = sqlx::query_as::<_, User>(
        "SELECT * FROM users WHERE username = $1 AND is_active = true"
    )
    .bind(username)
    .fetch_optional(pool)
    .await?;

    Ok(user)
}

pub async fn get_user_by_id(pool: &PgPool, user_id: Uuid) -> Result<Option<User>, AppError> {
    let user = sqlx::query_as::<_, User>(
        "SELECT * FROM users WHERE id = $1 AND is_active = true"
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    Ok(user)
}

pub async fn get_user_by_phone(pool: &PgPool, phone: &str) -> Result<Option<User>, AppError> {
    let user = sqlx::query_as::<_, User>(
        "SELECT u.* FROM users u JOIN phone_numbers pn ON u.id = pn.user_id WHERE pn.phone = $1 AND u.is_active = true LIMIT 1"
    )
    .bind(phone)
    .fetch_optional(pool)
    .await?;

    Ok(user)
}

// Device operations
pub async fn create_device(
    pool: &PgPool,
    user_id: Uuid,
    mac_address: &str,
    device_name: Option<&str>,
) -> Result<Device, AppError> {
    let id = Uuid::new_v4();
    let now = Utc::now();

    let device = sqlx::query_as::<_, Device>(
        r#"
        INSERT INTO devices (id, user_id, mac_address, device_name, created_at, is_active)
        VALUES ($1, $2, $3, $4, $5, true)
        RETURNING *
        "#,
    )
    .bind(&id)
    .bind(&user_id)
    .bind(mac_address)
    .bind(device_name)
    .bind(now)
    .fetch_one(pool)
    .await?;

    Ok(device)
}

pub async fn get_device_by_mac(pool: &PgPool, mac_address: &str) -> Result<Option<Device>, AppError> {
    let device = sqlx::query_as::<_, Device>(
        "SELECT * FROM devices WHERE mac_address = $1 AND is_active = true"
    )
    .bind(mac_address)
    .fetch_optional(pool)
    .await?;

    Ok(device)
}

pub async fn list_user_devices(pool: &PgPool, user_id: Uuid) -> Result<Vec<Device>, AppError> {
    let devices = sqlx::query_as::<_, Device>(
        "SELECT * FROM devices WHERE user_id = $1 AND is_active = true ORDER BY created_at DESC"
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    Ok(devices)
}

// Phone number operations
pub async fn add_phone_number(
    pool: &PgPool,
    user_id: Uuid,
    phone: &str,
) -> Result<PhoneNumber, AppError> {
    // Check if user has 3 phone numbers
    let count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM phone_numbers WHERE user_id = $1"
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    if count >= 3 {
        return Err(AppError::MaxPhoneNumbersReached);
    }

    let id = Uuid::new_v4();
    let now = Utc::now();

    let phone_number = sqlx::query_as::<_, PhoneNumber>(
        r#"
        INSERT INTO phone_numbers (id, user_id, phone, is_default, verified, created_at)
        VALUES ($1, $2, $3, $4, false, $5)
        RETURNING *
        "#,
    )
    .bind(&id)
    .bind(&user_id)
    .bind(phone)
    .bind(count == 0) // First phone is default
    .bind(now)
    .fetch_one(pool)
    .await?;

    Ok(phone_number)
}

pub async fn list_user_phones(pool: &PgPool, user_id: Uuid) -> Result<Vec<PhoneNumber>, AppError> {
    let phones = sqlx::query_as::<_, PhoneNumber>(
        "SELECT * FROM phone_numbers WHERE user_id = $1 ORDER BY is_default DESC, created_at"
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    Ok(phones)
}

pub async fn remove_phone_number(pool: &PgPool, phone_id: Uuid, user_id: Uuid) -> Result<(), AppError> {
    let result = sqlx::query(
        "DELETE FROM phone_numbers WHERE id = $1 AND user_id = $2"
    )
    .bind(phone_id)
    .bind(user_id)
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::ValidationError("Phone number not found".to_string()));
    }

    Ok(())
}

pub async fn set_default_phone(pool: &PgPool, phone_id: Uuid, user_id: Uuid) -> Result<(), AppError> {
    let mut tx = pool.begin().await?;

    // Remove default from all phones
    sqlx::query("UPDATE phone_numbers SET is_default = false WHERE user_id = $1")
        .bind(user_id)
        .execute(&mut *tx)
        .await?;

    // Set this phone as default
    let result = sqlx::query("UPDATE phone_numbers SET is_default = true WHERE id = $1 AND user_id = $2")
        .bind(phone_id)
        .bind(user_id)
        .execute(&mut *tx)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::ValidationError("Phone number not found".to_string()));
    }

    tx.commit().await?;
    Ok(())
}

// Package operations
pub async fn create_package(
    pool: &PgPool,
    req: &crate::models::CreatePackageRequest,
) -> Result<Package, AppError> {
    let id = Uuid::new_v4();
    let now = Utc::now();

    let package = sqlx::query_as::<_, Package>(
        r#"
        INSERT INTO packages (
            id, name, price_ksh, upload_speed_mbps, download_speed_mbps,
            time_limit_hours, bandwidth_limit_gb, description, concurrent_users,
            quota_type, is_active, created_at, updated_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, true, $11, $12)
        RETURNING *
        "#,
    )
    .bind(&id)
    .bind(&req.name)
    .bind(req.price_ksh)
    .bind(req.upload_speed_mbps)
    .bind(req.download_speed_mbps)
    .bind(req.time_limit_hours)
    .bind(req.bandwidth_limit_gb)
    .bind(&req.description)
    .bind(req.concurrent_users)
    .bind(&req.quota_type)
    .bind(now)
    .bind(now)
    .fetch_one(pool)
    .await?;

    Ok(package)
}

pub async fn list_active_packages(pool: &PgPool) -> Result<Vec<Package>, AppError> {
    let packages = sqlx::query_as::<_, Package>(
        "SELECT * FROM packages WHERE is_active = true ORDER BY price_ksh ASC"
    )
    .fetch_all(pool)
    .await?;

    Ok(packages)
}

pub async fn get_package(pool: &PgPool, package_id: Uuid) -> Result<Option<Package>, AppError> {
    let package = sqlx::query_as::<_, Package>(
        "SELECT * FROM packages WHERE id = $1 AND is_active = true"
    )
    .bind(package_id)
    .fetch_optional(pool)
    .await?;

    Ok(package)
}

pub async fn update_package(
    pool: &PgPool,
    package_id: Uuid,
    req: &crate::models::CreatePackageRequest,
) -> Result<Package, AppError> {
    let now = Utc::now();

    let package = sqlx::query_as::<_, Package>(
        r#"
        UPDATE packages SET
            name = $1, price_ksh = $2, upload_speed_mbps = $3,
            download_speed_mbps = $4, time_limit_hours = $5,
            bandwidth_limit_gb = $6, description = $7, concurrent_users = $8,
            quota_type = $9, updated_at = $10
        WHERE id = $11
        RETURNING *
        "#,
    )
    .bind(&req.name)
    .bind(req.price_ksh)
    .bind(req.upload_speed_mbps)
    .bind(req.download_speed_mbps)
    .bind(req.time_limit_hours)
    .bind(req.bandwidth_limit_gb)
    .bind(&req.description)
    .bind(req.concurrent_users)
    .bind(&req.quota_type)
    .bind(now)
    .bind(package_id)
    .fetch_one(pool)
    .await?;

    Ok(package)
}

pub async fn delete_package(pool: &PgPool, package_id: Uuid) -> Result<(), AppError> {
    sqlx::query("UPDATE packages SET is_active = false WHERE id = $1")
        .bind(package_id)
        .execute(pool)
        .await?;

    Ok(())
}

// Quota operations
pub async fn create_quota(
    pool: &PgPool,
    user_id: Uuid,
    package_id: Uuid,
    device_id: Option<Uuid>,
) -> Result<Quota, AppError> {
    let package = get_package(pool, package_id)
        .await?
        .ok_or(AppError::PackageNotFound)?;

    let id = Uuid::new_v4();
    let now = Utc::now();
    let expires_at = if let Some(hours) = package.time_limit_hours {
        now + Duration::hours(hours as i64)
    } else {
        now + Duration::days(365)
    };

    let quota = sqlx::query_as::<_, Quota>(
        r#"
        INSERT INTO quotas (
            id, user_id, package_id, device_id, purchased_at, expires_at,
            time_used_seconds, bandwidth_used_gb, status, last_activity
        )
        VALUES ($1, $2, $3, $4, $5, $6, 0, 0, 'active', $7)
        RETURNING *
        "#,
    )
    .bind(&id)
    .bind(&user_id)
    .bind(&package_id)
    .bind(&device_id)
    .bind(now)
    .bind(&expires_at)
    .bind(now)
    .fetch_one(pool)
    .await?;

    Ok(quota)
}

pub async fn get_active_quota(pool: &PgPool, user_id: Uuid) -> Result<Option<Quota>, AppError> {
    let quota = sqlx::query_as::<_, Quota>(
        r#"
        SELECT * FROM quotas
        WHERE user_id = $1 AND status = 'active' AND expires_at > NOW()
        ORDER BY purchased_at DESC
        LIMIT 1
        "#,
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    Ok(quota)
}

pub async fn update_quota_usage(
    pool: &PgPool,
    quota_id: Uuid,
    time_seconds: i64,
    bandwidth_gb: f64,
) -> Result<(), AppError> {
    let now = Utc::now();

    sqlx::query(
        r#"
        UPDATE quotas SET
            time_used_seconds = time_used_seconds + $1,
            bandwidth_used_gb = bandwidth_used_gb + $2,
            last_activity = $3
        WHERE id = $4
        "#,
    )
    .bind(time_seconds)
    .bind(bandwidth_gb)
    .bind(now)
    .bind(quota_id)
    .execute(pool)
    .await?;

    Ok(())
}

// Transaction operations
pub async fn create_transaction(
    pool: &PgPool,
    user_id: Uuid,
    phone: &str,
    package_id: Uuid,
    amount_ksh: i32,
) -> Result<Transaction, AppError> {
    let id = Uuid::new_v4();
    let now = Utc::now();

    let transaction = sqlx::query_as::<_, Transaction>(
        r#"
        INSERT INTO transactions (
            id, user_id, phone, package_id, amount_ksh, status, created_at, updated_at
        )
        VALUES ($1, $2, $3, $4, $5, 'pending', $6, $7)
        RETURNING *
        "#,
    )
    .bind(&id)
    .bind(&user_id)
    .bind(phone)
    .bind(&package_id)
    .bind(amount_ksh)
    .bind(now)
    .bind(now)
    .fetch_one(pool)
    .await?;

    Ok(transaction)
}

pub async fn get_transaction(pool: &PgPool, transaction_id: Uuid) -> Result<Option<Transaction>, AppError> {
    let transaction = sqlx::query_as::<_, Transaction>(
        "SELECT * FROM transactions WHERE id = $1"
    )
    .bind(transaction_id)
    .fetch_optional(pool)
    .await?;

    Ok(transaction)
}

pub async fn update_transaction_status(
    pool: &PgPool,
    transaction_id: Uuid,
    status: &str,
    mpesa_transaction_id: Option<&str>,
) -> Result<(), AppError> {
    let now = Utc::now();

    sqlx::query(
        "UPDATE transactions SET status = $1, mpesa_transaction_id = $2, updated_at = $3 WHERE id = $4"
    )
    .bind(status)
    .bind(mpesa_transaction_id)
    .bind(now)
    .bind(transaction_id)
    .execute(pool)
    .await?;

    Ok(())
}

// OTP operations
pub async fn create_otp(
    pool: &PgPool,
    user_id: Uuid,
    otp_code: &str,
    expiry_secs: u32,
) -> Result<OtpRecord, AppError> {
    let id = Uuid::new_v4();
    let now = Utc::now();
    let expires_at = now + Duration::seconds(expiry_secs as i64);

    let otp = sqlx::query_as::<_, OtpRecord>(
        r#"
        INSERT INTO otp_records (id, user_id, otp_code, created_at, expires_at, attempts, is_verified)
        VALUES ($1, $2, $3, $4, $5, 0, false)
        RETURNING *
        "#,
    )
    .bind(&id)
    .bind(&user_id)
    .bind(otp_code)
    .bind(now)
    .bind(&expires_at)
    .fetch_one(pool)
    .await?;

    Ok(otp)
}

pub async fn get_latest_otp(pool: &PgPool, user_id: Uuid) -> Result<Option<OtpRecord>, AppError> {
    let otp = sqlx::query_as::<_, OtpRecord>(
        "SELECT * FROM otp_records WHERE user_id = $1 ORDER BY created_at DESC LIMIT 1"
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    Ok(otp)
}

pub async fn verify_otp(pool: &PgPool, otp_id: Uuid, otp_code: &str) -> Result<bool, AppError> {
    let otp = sqlx::query_as::<_, OtpRecord>(
        "SELECT * FROM otp_records WHERE id = $1"
    )
    .bind(otp_id)
    .fetch_optional(pool)
    .await?;

    let otp = otp.ok_or(AppError::OtpInvalid)?;

    // Check expiry
    if Utc::now() > otp.expires_at {
        return Err(AppError::OtpExpired);
    }

    // Check max retries
    if otp.attempts >= 3 {
        return Err(AppError::OtpMaxRetries);
    }

    // Check code
    if otp.otp_code != otp_code {
        sqlx::query("UPDATE otp_records SET attempts = attempts + 1 WHERE id = $1")
            .bind(otp_id)
            .execute(pool)
            .await?;
        return Err(AppError::OtpInvalid);
    }

    // Mark as verified
    sqlx::query("UPDATE otp_records SET is_verified = true WHERE id = $1")
        .bind(otp_id)
        .execute(pool)
        .await?;

    Ok(true)
}
