//! Payment service executable entry point.
use std::{error::Error, sync::Arc};

use crate::{core::drivers, factory::make_router};

mod core;
mod factory;

#[tokio::main]
/// Starts the HTTP server and serves payment service routes.
async fn main() -> Result<(), Box<dyn Error>> {
    let axum_driver = drivers::AxumDriver;
    let router = make_router(Arc::new(axum_driver));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(listener, router).await?;
    Ok(())
}
