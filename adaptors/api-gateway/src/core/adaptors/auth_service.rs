//! HTTP client adapter for the authentication service.
use std::collections::HashMap;

use forma_core::domain::entities::{FormaError, FormaErrorConverter, FormaErrorExternalService};
use url::Url;

use crate::{
    domain::{
        blueprints::{AuthServiceBlueprint, ServiceBlueprint},
        entities::Upstream,
    },
    value_objects::UpstreamKey,
};

/// Builds an upstream URL from its configured scheme, host, and port.
fn get_url(upstream: &HashMap<UpstreamKey, Upstream>, key: UpstreamKey) -> Result<Url, FormaError> {
    let upstream = upstream.get(&key).ok_or(FormaError::new(
        FormaErrorExternalService::UrlError,
        "authentication service upstream is not configured",
    ))?;
    let url = format!(
        "{}://{}:{}",
        &upstream.scheme, &upstream.domain, &upstream.port
    );
    Url::parse(&url).map_forma_err(FormaErrorExternalService::UrlError, "Could not parse URL")
}

/// Provides HTTP access to configured upstream services.
pub struct ServiceAdaptor {
    client: reqwest::Client,

    auth_service_url: Url,
}

impl ServiceAdaptor {
    /// Creates the adapter and resolves the authentication service URL.
    pub fn new(
        client: reqwest::Client,
        upstream: &HashMap<UpstreamKey, Upstream>,
    ) -> Result<Self, FormaError> {
        let auth_service_url = get_url(upstream, UpstreamKey::AuthService)?;
        Ok(Self {
            client,
            auth_service_url,
        })
    }
}

#[async_trait::async_trait]
impl AuthServiceBlueprint for ServiceAdaptor {
    fn get_url(&self) -> Url {
        self.auth_service_url.clone()
    }

    fn get_client(&self) -> &reqwest::Client {
        &self.client
    }
}

impl ServiceBlueprint for ServiceAdaptor {}
