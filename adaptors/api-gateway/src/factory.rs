use crate::{
    core::drivers::{AuthMiddlewareState, AxumDriver, auth_middleware},
    domain::entities::{AuthStrategy, Config},
};
use axum::{
    Router,
    http::{HeaderName, HeaderValue},
    middleware,
};
use axum_reverse_proxy::{ProxyRouterExt, proxy_template};
use forma_core::domain::{
    TryCollectExt,
    entities::{FormaError, FormaErrorApp, FormaErrorConverter, FormaOptionConverter},
};
use reqwest::Method;
use std::{collections::HashMap, sync::Arc};
use tower_http::cors::CorsLayer;

pub fn build_api_route_v1(
    config: Config,
    driver: Arc<AxumDriver>,
) -> Result<Router<Arc<AxumDriver>>, FormaError> {
    let mut app = Router::new();

    let mut routers = HashMap::new();
    for (route_name, _) in config.route.iter() {
        routers.insert(route_name, Router::new());
    }

    for proxy in config.proxy {
        let upstream = config.upstream.get(&proxy.upstream).ok_or_forma_error()?;
        let router = routers.get(&proxy.route).ok_or_forma_error()?;
        let routed_router = router.clone().proxy_route(
            &proxy.path,
            proxy_template(format!(
                "{}://{}:{}{}",
                &upstream.scheme, &upstream.domain, &upstream.port, &proxy.forward
            )),
        );

        let router = routers.get_mut(&proxy.route).ok_or_forma_error()?;
        *router = routed_router;
    }

    for (route_name, route) in &config.route {
        let mut router = routers.get(&route_name).ok_or_forma_error()?.clone();

        match route.auth {
            AuthStrategy::Token | AuthStrategy::SemiToken => {
                router = router.route_layer(middleware::from_fn_with_state(
                    Arc::new(AuthMiddlewareState {
                        state: driver.clone(),
                        auth_strategy: route.auth,
                    }),
                    auth_middleware,
                ));
            }
            AuthStrategy::None => {}
        };

        if let Some(cors) = &route.cors {
            let layer = CorsLayer::new()
                .allow_credentials(cors.allow_credentials)
                .allow_headers(
                    cors.allow_headers
                        .iter()
                        .map(|v| {
                            v.parse().map_forma_err(
                                FormaErrorApp::InvalidType,
                                "Fail to parse allowed headers list",
                            )
                        })
                        .try_collect::<Vec<HeaderName>>()?,
                )
                .allow_methods(
                    cors.allow_methods
                        .iter()
                        .map(|v| {
                            v.parse().map_forma_err(
                                FormaErrorApp::InvalidType,
                                "Fail to parse allowed method list",
                            )
                        })
                        .try_collect::<Vec<Method>>()?,
                )
                .allow_origin(
                    cors.allow_origins
                        .iter()
                        .map(|v| {
                            v.parse().map_forma_err(
                                FormaErrorApp::InvalidType,
                                "Fail to parse allowed origins list",
                            )
                        })
                        .try_collect::<Vec<HeaderValue>>()?,
                );
            router = router.layer(layer);
        };

        app = app.merge(router);
    }

    return Ok(app);
}
