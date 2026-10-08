use std::sync::Arc;

use axum::{Json, extract::State, http::StatusCode, response::{IntoResponse, Response}};
use forma_schema::HealthzCheck;

pub struct AxumDriver;

pub async fn healtz(State(driver): State<Arc<AxumDriver>>) -> Response {
    return (StatusCode::OK, Json(HealthzCheck::default())).into_response();
}