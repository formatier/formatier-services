use std::sync::Arc;

use axum::{Router, routing::get};

use crate::core::drivers::{self, AxumDriver};

pub fn make_router(axum_driver_state: Arc<AxumDriver>) -> Router {
    let router = Router::new().route("/healtz", get(drivers::healtz));

    router.with_state(axum_driver_state)
}
