//! Axum handlers for plugin-service operational endpoints.
use std::sync::Arc;

use axum::{
    Json,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use forma_core::domain::entities::HealthzCheck;

/// Shared state for plugin HTTP handlers.
pub struct AxumDriver;

/// Returns the plugin service health-check response.
///
/// # Arguments
/// * `driver` - Shared Axum driver state.
///
/// # Returns
/// An HTTP 200 response containing the default health-check payload.
pub async fn healtz(State(driver): State<Arc<AxumDriver>>) -> Response {
    return (StatusCode::OK, Json(HealthzCheck::default())).into_response();
}
