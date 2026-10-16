use std::{collections::HashMap, fmt::Display};

use serde::{Deserialize, Serialize};

use crate::value_objects::UpstreamKey;

#[derive(Deserialize, Serialize)]
pub struct Config {
    pub upstream: HashMap<UpstreamKey, Upstream>,
    pub route: HashMap<Box<str>, Route>,
    pub proxy: Box<[Proxy]>,
}

#[derive(Deserialize, Serialize)]
pub struct Upstream {
    pub domain: Box<str>,
    pub port: u16,
    pub scheme: Scheme,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Scheme {
    Http,
    Https,
}

impl Display for Scheme {
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
pub struct Route {
    pub auth: AuthStrategy,
    pub cors: Option<Cors>,
    pub rate_limiter: Option<RateLimiter>,
}

#[derive(Deserialize, Serialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum AuthStrategy {
    None,
    Token,
    SemiToken,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct Cors {
    pub allow_credentials: bool,
    pub allow_headers: Box<[Box<str>]>,
    pub allow_methods: Box<[Box<str>]>,
    pub allow_origins: Box<[Box<str>]>,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct RateLimiter {
    pub algorithm: RateLimiterAlgorithm,
    pub rps: u8,
    pub scope: RateLimiterScope,
}

#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum RateLimiterAlgorithm {
    Bucket,
    Window,
}

#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum RateLimiterScope {
    Global,
    Route,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct Proxy {
    pub path: Box<str>,
    pub forward: Box<str>,
    pub upstream: UpstreamKey,
    pub route: Box<str>,
}
