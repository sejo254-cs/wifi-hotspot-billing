/// RADIUS Attribute Types (RFC 2865)
pub mod attribute_types {
    pub const USER_NAME: u8 = 1;
    pub const USER_PASSWORD: u8 = 2;
    pub const REPLY_MESSAGE: u8 = 18;
    pub const CALLING_STATION_ID: u8 = 31;  // MAC Address
    pub const SESSION_TIMEOUT: u8 = 27;
    pub const IDLE_TIMEOUT: u8 = 28;
    pub const SERVICE_TYPE: u8 = 6;
    pub const FILTER_ID: u8 = 11;
}

/// MikroTik Vendor Specific Attributes (VSA)
pub mod mikrotik_attrs {
    pub const VENDOR_ID: u32 = 14988;
    
    // MikroTik-specific attributes
    pub const RATE_LIMIT_TX: u8 = 4;      // Upload rate (tx)
    pub const RATE_LIMIT_RX: u8 = 5;      // Download rate (rx)
    pub const MARK: u8 = 9;                // Queue mark
    pub const CAP: u8 = 23;                // Bandwidth cap
}

pub fn build_rate_limit_string(upload_kbps: u32, download_kbps: u32) -> String {
    // MikroTik rate limit format: "upload=Xk/s download=Yk/s"
    format!("{}M/{}M", upload_kbps / 1024, download_kbps / 1024)
}

pub fn parse_mac_address(mac_str: &str) -> Option<String> {
    // Convert MAC from CALLING_STATION_ID format to normalized form
    let normalized = mac_str.to_uppercase().replace('-', ":");
    Some(normalized)
}
