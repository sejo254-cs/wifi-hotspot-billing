use crate::config::Config;
use crate::error::AppError;
use crate::models::*;
use crate::radius::attributes::*;
use crate::radius::packet::*;
use crate::utils;
use chrono::Utc;
use sqlx::PgPool;
use std::net::UdpSocket;
use tokio::task;
use uuid::Uuid;

pub async fn start(pool: PgPool, config: Config) -> Result<(), AppError> {
    let socket = UdpSocket::new(format!("{}:{}", config.radius_host, config.radius_port))
        .map_err(|e| {
            tracing::error!("Failed to bind RADIUS socket: {}", e);
            AppError::InternalError
        })?;

    socket
        .set_nonblocking(true)
        .map_err(|_| AppError::InternalError)?;

    tracing::info!(
        "RADIUS server listening on {}:{}",
        config.radius_host,
        config.radius_port
    );

    let socket = std::sync::Arc::new(socket);
    let pool = std::sync::Arc::new(pool);
    let config = std::sync::Arc::new(config);

    loop {
        let mut buffer = [0; 4096];
        match socket.recv_from(&mut buffer) {
            Ok((size, src_addr)) => {
                let socket = socket.clone();
                let pool = pool.clone();
                let config = config.clone();

                task::spawn(async move {
                    if let Err(e) = handle_radius_request(
                        &buffer[..size],
                        src_addr,
                        socket.as_ref(),
                        pool.as_ref(),
                        config.as_ref(),
                    )
                    .await
                    {
                        tracing::error!("RADIUS request error: {}", e);
                    }
                });
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            }
            Err(e) => {
                tracing::error!("Socket error: {}", e);
            }
        }
    }
}

async fn handle_radius_request(
    data: &[u8],
    src_addr: std::net::SocketAddr,
    socket: &UdpSocket,
    pool: &PgPool,
    config: &Config,
) -> Result<(), AppError> {
    let packet = RadiusPacket::from_bytes(data)
        .map_err(|e| {
            tracing::warn!("Invalid RADIUS packet: {}", e);
            AppError::InternalError
        })?;

    match packet.code {
        1 => handle_access_request(packet, src_addr, socket, pool, config).await,
        4 => handle_accounting_request(packet, src_addr, socket, pool, config).await,
        _ => {
            tracing::warn!("Unknown RADIUS code: {}", packet.code);
            Ok(())
        }
    }
}

async fn handle_access_request(
    req: RadiusPacket,
    src_addr: std::net::SocketAddr,
    socket: &UdpSocket,
    pool: &PgPool,
    config: &Config,
) -> Result<(), AppError> {
    tracing::debug!("Access-Request from {}", src_addr);

    let username = req
        .get_attribute_string(attribute_types::USER_NAME)
        .ok_or(AppError::ValidationError("No User-Name".to_string()))?;

    let mac_address = req
        .get_attribute_string(attribute_types::CALLING_STATION_ID)
        .ok_or(AppError::ValidationError("No Calling-Station-Id".to_string()))?;

    let mac = attribute_types::parse_mac_address(&mac_address)
        .ok_or(AppError::MacAddressInvalid)?;

    // Get device by MAC
    let device = crate::db::get_device_by_mac(pool, &mac)
        .await?;

    let response = if let Some(device) = device {
        // Get user
        if let Ok(Some(user)) = crate::db::get_user_by_id(pool, device.user_id).await {
            // Check active quota
            if let Ok(Some(quota)) = crate::db::get_active_quota(pool, user.id).await {
                // Get package details
                if let Ok(Some(package)) = crate::db::get_package(pool, quota.package_id).await {
                    build_access_accept(
                        req,
                        username.clone(),
                        &package,
                        config,
                    )
                } else {
                    build_access_reject(req, "Package not found", config)
                }
            } else {
                build_access_reject(req, "No active quota", config)
            }
        } else {
            build_access_reject(req, "User not found", config)
        }
    } else {
        build_access_reject(req, "Device not found", config)
    };

    let response_bytes = response.to_bytes();
    socket
        .send_to(&response_bytes, src_addr)
        .map_err(|e| {
            tracing::error!("Failed to send RADIUS response: {}", e);
            AppError::InternalError
        })?;

    Ok(())
}

async fn handle_accounting_request(
    req: RadiusPacket,
    src_addr: std::net::SocketAddr,
    socket: &UdpSocket,
    pool: &PgPool,
    config: &Config,
) -> Result<(), AppError> {
    tracing::debug!("Accounting-Request from {}", src_addr);

    // Build Accounting-Response
    let mut response = RadiusPacket::new(5); // Accounting-Response
    response.identifier = req.identifier;
    response.authenticator = req.authenticator;

    let response_bytes = response.to_bytes();
    socket
        .send_to(&response_bytes, src_addr)
        .map_err(|_| AppError::InternalError)?;

    Ok(())
}

fn build_access_accept(
    req: RadiusPacket,
    username: String,
    package: &crate::models::Package,
    config: &Config,
) -> RadiusPacket {
    let mut response = RadiusPacket::new(2); // Access-Accept
    response.identifier = req.identifier;
    response.authenticator = req.authenticator;

    // Add Session-Timeout
    if let Some(hours) = package.time_limit_hours {
        let timeout_seconds = hours as u32 * 3600;
        response.add_attribute(
            attribute_types::SESSION_TIMEOUT,
            &timeout_seconds.to_be_bytes(),
        );
    }

    // Add rate limits using MikroTik format
    let rate_limit = format!(
        "{}M/{}M",
        package.upload_speed_mbps, package.download_speed_mbps
    );
    response.add_attribute(attribute_types::FILTER_ID, rate_limit.as_bytes());

    // Add reply message
    response.add_attribute(
        attribute_types::REPLY_MESSAGE,
        b"Access granted",
    );

    tracing::info!(
        "Access-Accept for {} with {}Mbps upload / {}Mbps download",
        username,
        package.upload_speed_mbps,
        package.download_speed_mbps
    );

    response
}

fn build_access_reject(req: RadiusPacket, reason: &str, config: &Config) -> RadiusPacket {
    let mut response = RadiusPacket::new(3); // Access-Reject
    response.identifier = req.identifier;
    response.authenticator = req.authenticator;

    response.add_attribute(attribute_types::REPLY_MESSAGE, reason.as_bytes());

    tracing::warn!("Access-Reject: {}", reason);

    response
}
