//! HTTP route construction for the interspace service.
use std::sync::Arc;

use axum::{Router, routing::get};

use crate::core::drivers::{self, AxumDriver};

/// Builds the interspace router and attaches shared driver state.
///
/// # Arguments
/// * `axum_driver_state` - Shared state consumed by route handlers.
///
/// # Returns
/// The configured interspace service router.
pub fn make_router(axum_driver_state: Arc<AxumDriver>) -> Router {
    let router = Router::new().route("/healtz", get(drivers::healtz));

    router.with_state(axum_driver_state)
}
