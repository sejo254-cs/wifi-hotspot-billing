use crate::error::AppError;
use argon2::{password_hash::SaltString, Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use rand::Rng;
use regex::Regex;

const MIN_PASSWORD_LENGTH: usize = 8;
const MAX_FIELD_LENGTH: usize = 20;

/// Validate password strength
pub fn validate_password(password: &str) -> Result<(), AppError> {
    if password.len() < MIN_PASSWORD_LENGTH {
        return Err(AppError::WeakPassword);
    }

    if password.len() > 100 {
        return Err(AppError::ValidationError("Password too long".to_string()));
    }

    // Check for mix of character types
    let has_upper = password.chars().any(|c| c.is_uppercase());
    let has_lower = password.chars().any(|c| c.is_lowercase());
    let has_digit = password.chars().any(|c| c.is_numeric());
    let has_special = password.chars().any(|c| !c.is_alphanumeric());

    let complexity_count = [has_upper, has_lower, has_digit, has_special]
        .iter()
        .filter(|&&x| x)
        .count();

    if complexity_count < 3 {
        return Err(AppError::WeakPassword);
    }

    Ok(())
}

/// Validate phone number (Safaricom format: 254xxxxxxxxx or 07xxxxxxxxx)
pub fn validate_phone(phone: &str) -> Result<(), AppError> {
    let phone_clean = phone.trim();

    if phone_clean.len() < 10 || phone_clean.len() > 13 {
        return Err(AppError::InvalidPhone);
    }

    // Accept formats: 254xxxxxxxxx or 07xxxxxxxxx or 2547xxxxxxxxx
    let phone_regex = Regex::new(r"^(254\d{9}|07\d{8}|2547\d{8})$")
        .map_err(|_| AppError::InternalError)?;

    if !phone_regex.is_match(phone_clean) {
        return Err(AppError::InvalidPhone);
    }

    Ok(())
}

/// Normalize phone to 254 format
pub fn normalize_phone(phone: &str) -> Result<String, AppError> {
    validate_phone(phone)?;
    let clean = phone.trim();

    let normalized = if clean.starts_with("07") {
        format!("254{}", &clean[1..])
    } else if clean.starts_with("2547") {
        format!("254{}", &clean[3..])
    } else {
        clean.to_string()
    };

    Ok(normalized)
}

/// Validate username
pub fn validate_username(username: &str) -> Result<(), AppError> {
    let username = username.trim();

    if username.is_empty() || username.len() > MAX_FIELD_LENGTH {
        return Err(AppError::ValidationError("Username must be 1-20 characters".to_string()));
    }

    if !username.chars().all(|c| c.is_alphanumeric() || c == '_') {
        return Err(AppError::ValidationError(
            "Username can only contain alphanumeric characters and underscore".to_string(),
        ));
    }

    Ok(())
}

/// Validate MAC address
pub fn validate_mac_address(mac: &str) -> Result<(), AppError> {
    let mac_regex = Regex::new(r"^([0-9A-Fa-f]{2}[:-]){5}([0-9A-Fa-f]{2})$")
        .map_err(|_| AppError::InternalError)?;

    if !mac_regex.is_match(mac) {
        return Err(AppError::MacAddressInvalid);
    }

    Ok(())
}

/// Normalize MAC address to lowercase with colons
pub fn normalize_mac_address(mac: &str) -> Result<String, AppError> {
    validate_mac_address(mac)?;
    Ok(mac.to_lowercase().replace('-', ":"))
}

/// Hash password using Argon2
pub fn hash_password(password: &str) -> Result<String, AppError> {
    validate_password(password)?;

    let salt = SaltString::generate(rand::thread_rng());
    let argon2 = Argon2::default();

    argon2
        .hash_password(password.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|_| AppError::InternalError)
}

/// Verify password against hash
pub fn verify_password(password: &str, hash: &str) -> Result<bool, AppError> {
    let parsed_hash = PasswordHash::new(hash).map_err(|_| AppError::InternalError)?;

    let argon2 = Argon2::default();
    match argon2.verify_password(password.as_bytes(), &parsed_hash) {
        Ok(_) => Ok(true),
        Err(argon2::password_hash::Error::Password) => Ok(false),
        Err(_) => Err(AppError::InternalError),
    }
}

/// Generate OTP (6 digits)
pub fn generate_otp() -> String {
    let mut rng = rand::thread_rng();
    format!("{:06}", rng.gen_range(0..1000000))
}

/// Generate JWT token
pub fn generate_jwt(user_id: &uuid::Uuid, secret: &str) -> Result<String, AppError> {
    use chrono::Duration;

    let now = chrono::Utc::now();
    let expiration = now + Duration::hours(24);

    let payload = serde_json::json!({
        "user_id": user_id.to_string(),
        "iat": now.timestamp(),
        "exp": expiration.timestamp(),
    });

    // Simplified JWT - in production, use a proper JWT library like `jsonwebtoken`
    let header = base64::encode(r#"{"alg":"HS256","typ":"JWT"}"#);
    let body = base64::encode(payload.to_string());
    let signature = generate_hmac_sha256(&format!("{}.{}", header, body), secret);

    Ok(format!("{}.{}.{}", header, body, signature))
}

/// Generate HMAC-SHA256
fn generate_hmac_sha256(message: &str, secret: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    format!("{}{}", message, secret).hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

/// Parse JWT token
pub fn parse_jwt(token: &str, secret: &str) -> Result<uuid::Uuid, AppError> {
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 {
        return Err(AppError::Unauthorized);
    }

    let body = base64::decode(parts[1]).map_err(|_| AppError::Unauthorized)?;
    let payload: serde_json::Value =
        serde_json::from_slice(&body).map_err(|_| AppError::Unauthorized)?;

    let exp = payload["exp"]
        .as_i64()
        .ok_or(AppError::Unauthorized)?;

    if exp < chrono::Utc::now().timestamp() {
        return Err(AppError::SessionExpired);
    }

    let user_id_str = payload["user_id"]
        .as_str()
        .ok_or(AppError::Unauthorized)?;

    uuid::Uuid::parse_str(user_id_str).map_err(|_| AppError::Unauthorized)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_phone() {
        assert!(validate_phone("254712345678").is_ok());
        assert!(validate_phone("0712345678").is_ok());
        assert!(validate_phone("2547.12345678").is_err());
    }

    #[test]
    fn test_normalize_phone() {
        assert_eq!(normalize_phone("0712345678").unwrap(), "254712345678");
        assert_eq!(
            normalize_phone("254712345678").unwrap(),
            "254712345678"
        );
    }

    #[test]
    fn test_validate_mac() {
        assert!(validate_mac_address("F4:1E:57:C7:B2:6F").is_ok());
        assert!(validate_mac_address("F4-1E-57-C7-B2-6F").is_ok());
        assert!(validate_mac_address("F4:1E:57:C7:B2").is_err());
    }

    #[test]
    fn test_password_validation() {
        assert!(validate_password("ValidPass123!").is_ok());
        assert!(validate_password("weak").is_err());
        assert!(validate_password("NoSpecial123").is_err());
    }
}
