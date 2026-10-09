mod core;
mod domain;
mod factory;
mod utils;
mod value_objects;

use crate::core::{adaptors::ServiceAdaptor, drivers::AxumDriver};
use forma_core::utils::init_rustls;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    init_rustls();

    let config = utils::load_config().unwrap();

    let service_adaptor = ServiceAdaptor::new(reqwest::Client::new(), &config.upstream);
    let service_adaptor = Arc::new(service_adaptor);
    let driver = Arc::new(AxumDriver::new(service_adaptor.clone()));

    let router = factory::build_api_route_v1(config, driver.clone()).unwrap();

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, router.with_state(driver))
        .await
        .unwrap();
}
