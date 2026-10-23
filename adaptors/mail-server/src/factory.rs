//! HTTP route construction for the mail server.
use std::sync::Arc;

use axum::{Router, routing::get};

use crate::core::drivers::{self, AxumDriver};

/// Builds the mail-server router and attaches shared driver state.
///
/// # Arguments
/// * `axum_driver_state` - State shared with route handlers.
///
/// # Returns
/// The configured mail-server router.
pub fn make_router(axum_driver_state: Arc<AxumDriver>) -> Router {
    let router = Router::new().route("/healtz", get(drivers::healtz));

    router.with_state(axum_driver_state)
}
