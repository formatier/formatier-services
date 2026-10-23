//! HTTP client adapter for the authentication service.
use std::collections::HashMap;

use forma_core::domain::entities::{FormaErrorConverter, FormaErrorExternalService};
use url::Url;

use crate::{
    domain::{
        blueprints::{AuthServiceBlueprint, ServiceBlueprint},
        entities::Upstream,
    },
    value_objects::UpstreamKey,
};

/// Builds an upstream URL from its configured scheme, host, and port.
///
/// # Arguments
/// * `upstream` - Configured upstream endpoints.
/// * `key` - Key identifying the endpoint to resolve.
///
/// # Returns
/// The parsed URL for the selected upstream.
///
/// # Panics
/// Panics if the key is missing or the configured URL is invalid.
fn get_url(upstream: &HashMap<UpstreamKey, Upstream>, key: UpstreamKey) -> Url {
    let upstream = upstream.get(&key).unwrap();
    let url = format!(
        "{}://{}:{}",
        &upstream.scheme, &upstream.domain, &upstream.port
    );
    let url = Url::parse(&url)
        .map_forma_err(FormaErrorExternalService::UrlError, "cannot parse url")
        .unwrap();

    url
}

/// Provides HTTP access to configured upstream services.
pub struct ServiceAdaptor {
    client: reqwest::Client,

    auth_service_url: Url,
}

impl ServiceAdaptor {
    /// Creates the adapter and resolves the authentication-service URL.
    ///
    /// # Arguments
    /// * `client` - Reqwest client used for upstream requests.
    /// * `upstream` - Configured upstream endpoints.
    ///
    /// # Returns
    /// A service adapter.
    ///
    /// # Panics
    /// Panics if the authentication service is not configured or its URL is
    /// invalid.
    pub fn new(client: reqwest::Client, upstream: &HashMap<UpstreamKey, Upstream>) -> Self {
        let auth_service_url = get_url(&upstream, UpstreamKey::AuthService);
        Self {
            client,
            auth_service_url,
        }
    }
}

#[async_trait::async_trait]
impl AuthServiceBlueprint for ServiceAdaptor {
    /// Returns the configured authentication-service URL.
    ///
    /// # Returns
    /// A clone of the URL.
    fn get_url(&self) -> Url {
        self.auth_service_url.clone()
    }

    /// Returns the HTTP client used by this adapter.
    ///
    /// # Returns
    /// A shared reference to the Reqwest client.
    fn get_client(&self) -> &reqwest::Client {
        &self.client
    }
}

impl ServiceBlueprint for ServiceAdaptor {}
