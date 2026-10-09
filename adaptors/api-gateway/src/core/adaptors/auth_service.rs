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

fn get_url(upstream: &HashMap<UpstreamKey, Upstream>, key: UpstreamKey) -> Url {
    let upstream = upstream.get(&key).unwrap();
    let mut url = Url::parse(&upstream.domain)
        .map_forma_err(FormaErrorExternalService::UrlError, "cannot parse url")
        .unwrap();
    url.set_port(Some(upstream.port)).unwrap();
    url.set_scheme(&upstream.scheme.to_string()).unwrap();

    url
}

pub struct ServiceAdaptor {
    client: reqwest::Client,

    auth_service_url: Url,
}

impl ServiceAdaptor {
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
    fn get_url(&self) -> Url {
        self.auth_service_url.clone()
    }

    fn get_client(&self) -> &reqwest::Client {
        &self.client
    }
}

impl ServiceBlueprint for ServiceAdaptor {}
