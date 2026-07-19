use crate::models::*;
use crate::AppState;
use axum::{
    extract::{Path, State},
    Json,
};
use std::sync::Arc;
use uuid::Uuid;

pub async fn list_packages(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<PackageResponse>>, crate::error::AppError> {
    let packages = crate::db::list_active_packages(&state.pool).await?;

    let responses = packages
        .into_iter()
        .map(|p| PackageResponse {
            id: p.id,
            name: p.name,
            price_ksh: p.price_ksh,
            time_limit_hours: p.time_limit_hours,
            description: p.description,
        })
        .collect();

    Ok(Json(responses))
}

pub async fn get_package(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<PackageResponse>, crate::error::AppError> {
    let package = crate::db::get_package(&state.pool, id)
        .await?
        .ok_or(crate::error::AppError::PackageNotFound)?;

    Ok(Json(PackageResponse {
        id: package.id,
        name: package.name,
        price_ksh: package.price_ksh,
        time_limit_hours: package.time_limit_hours,
        description: package.description,
    }))
}
