//! Axum handlers for monitoring-service endpoints.
use std::sync::Arc;

use axum::{
    Json,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use forma_core::domain::entities::HealthzCheck;

/// Shared state for monitoring HTTP handlers.
pub struct AxumDriver;

pub async fn healthz(State(_driver): State<Arc<AxumDriver>>) -> Response {
    return (StatusCode::OK, Json(HealthzCheck::default())).into_response();
}
