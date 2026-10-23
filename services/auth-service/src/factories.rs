//! HTTP route construction for the authentication service.
use std::sync::Arc;

use axum::{Router, routing::post};

use crate::core::drivers;

/// Builds the version-one authentication routes.
///
/// # Returns
/// A router containing the version-one routes.
fn make_router_v1() -> Router<Arc<drivers::AxumDriver>> {
    let router =
        axum::Router::new().route("/signin/provider", post(drivers::signin_by_oauth_provider));
    router
}

/// Builds the authentication router and attaches shared driver state.
///
/// # Arguments
/// * `axum_driver_state` - Shared state consumed by route handlers.
///
/// # Returns
/// The configured authentication router.
pub fn make_router(axum_driver_state: Arc<drivers::AxumDriver>) -> Router {
    let router = axum::Router::new()
        .nest("/v1", make_router_v1())
        .with_state(axum_driver_state);
    router
}
