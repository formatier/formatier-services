//! Mail-server executable entry point.
use std::sync::Arc;

use crate::{core::drivers, factory::make_router};

mod core;
mod domain;
mod factory;

#[tokio::main]
/// Starts the HTTP server and serves mail-server routes.
async fn main() {
    let axum_driver = drivers::AxumDriver;
    let router = make_router(Arc::new(axum_driver));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, router).await.unwrap();
}
