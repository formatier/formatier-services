//! Configuration data structures for routes and upstream services.
use std::{collections::HashMap, fmt::Display};

use serde::{Deserialize, Serialize};

use crate::value_objects::UpstreamKey;

#[derive(Deserialize, Serialize)]
/// Complete API gateway routing configuration.
pub struct Config {
    /// Upstream services addressable by configuration key.
    pub upstream: HashMap<UpstreamKey, Upstream>,
    /// Named route policies.
    pub route: HashMap<Box<str>, Route>,
    /// Proxy routes exposed by the gateway.
    pub proxy: Box<[Proxy]>,
}

#[derive(Deserialize, Serialize)]
/// Network destination configuration for an upstream service.
pub struct Upstream {
    /// Upstream hostname or address.
    pub domain: Box<str>,
    /// Upstream TCP port.
    pub port: u16,
    /// URL scheme used to connect.
    pub scheme: Scheme,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
/// Supported schemes for upstream URLs.
pub enum Scheme {
    /// Unencrypted HTTP.
    Http,
    /// TLS-protected HTTPS.
    Https,
}

/// Formats the scheme using its URL spelling.
impl Display for Scheme {
    /// Writes the URL scheme to a formatter.
    ///
    /// # Arguments
    /// * `f` - Destination formatter.
    ///
    /// # Returns
    /// The formatter result.
    ///
    /// # Errors
    /// Returns a formatting error if writing fails.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Scheme::Http => {
                write!(f, "{}", "http")
            }
            Scheme::Https => {
                write!(f, "{}", "https")
            }
        }
    }
}

#[derive(Deserialize, Serialize, Clone)]
/// Authentication and optional request policies for a route.
pub struct Route {
    /// Authentication policy applied to the route.
    pub auth: AuthStrategy,
    /// Optional cross-origin request policy.
    pub cors: Option<Cors>,
    /// Optional request rate-limiting policy.
    pub rate_limiter: Option<RateLimiter>,
}

#[derive(Deserialize, Serialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
/// Authentication policy configured for a route.
pub enum AuthStrategy {
    /// Allows requests without token authentication.
    None,
    /// Requires a valid token.
    Token,
    /// Applies the route's semi-token policy.
    SemiToken,
}

#[derive(Deserialize, Serialize, Clone)]
/// Cross-origin resource-sharing policy for a route.
pub struct Cors {
    /// Whether cross-origin requests may include credentials.
    pub allow_credentials: bool,
    /// Request headers permitted by the policy.
    pub allow_headers: Box<[Box<str>]>,
    /// HTTP methods permitted by the policy.
    pub allow_methods: Box<[Box<str>]>,
    /// Origins permitted by the policy.
    pub allow_origins: Box<[Box<str>]>,
}

#[derive(Deserialize, Serialize, Clone)]
/// Request rate-limiting configuration.
pub struct RateLimiter {
    /// Algorithm used to enforce the limit.
    pub algorithm: RateLimiterAlgorithm,
    /// Allowed requests per second.
    pub rps: u8,
    /// Scope over which the limit is applied.
    pub scope: RateLimiterScope,
}

#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "snake_case")]
/// Available rate-limiting algorithms.
pub enum RateLimiterAlgorithm {
    /// Token-bucket algorithm.
    Bucket,
    /// Fixed-window algorithm.
    Window,
}

#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "snake_case")]
/// Scope used to group requests for rate limiting.
pub enum RateLimiterScope {
    /// Applies one limit across the whole gateway.
    Global,
    /// Applies a separate limit to each route.
    Route,
}

#[derive(Deserialize, Serialize, Clone)]
/// Maps an exposed path to a configured upstream path.
pub struct Proxy {
    /// Path exposed by the gateway.
    pub path: Box<str>,
    /// Path forwarded to the upstream.
    pub forward: Box<str>,
    /// Key identifying the upstream service.
    pub upstream: UpstreamKey,
    /// Name of the route policy applied to this proxy.
    pub route: Box<str>,
}
