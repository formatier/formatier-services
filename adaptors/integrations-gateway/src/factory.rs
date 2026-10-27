//! HTTP route construction for the integrations gateway.
use std::sync::Arc;

use axum::{Router, routing::get};

use crate::core::drivers::{self, AxumDriver};

/// Builds the integrations gateway router and attaches shared driver state.
pub fn make_router(axum_driver_state: Arc<AxumDriver>) -> Router {
    let router = Router::new().route("/healthz", get(drivers::healthz));

    router.with_state(axum_driver_state)
}
