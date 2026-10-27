//! Runtime environment configuration and service-level constants.
use std::env;

use crate::domain::entities::{FormaError, FormaErrorConverter, FormaErrorKind};

/// Identifies the runtime environment selected for this build.
pub enum RuntimeEnvironment {
    /// Development or debug runtime.
    Dev,
    /// Production or optimized runtime.
    Prod,
}

/// Reads and trims a string-valued environment variable.
pub fn get_env(key: &str) -> Result<String, FormaError> {
    env::var(key)
        .map(|v| v.trim_matches('"').into())
        .map_forma_err(FormaErrorKind::Internal, "cannot load environment variable")
}

#[cfg(debug_assertions)]
/// Runtime environment selected for debug builds.
pub const RUNTIME_ENV: RuntimeEnvironment = RuntimeEnvironment::Dev;

#[cfg(not(debug_assertions))]
/// Runtime environment selected for optimized builds.
pub const RUNTIME_ENV: RuntimeEnvironment = RuntimeEnvironment::Prod;

/// Loads the service port from `PORT`, defaulting to `3310` when it is unset.
pub fn port() -> Result<u16, FormaError> {
    let value = match env::var("PORT") {
        Ok(value) => Some(value),
        Err(env::VarError::NotPresent) => None,
        Err(env::VarError::NotUnicode(_)) => {
            return Err(FormaError::new(
                FormaErrorKind::Internal,
                "PORT must be valid Unicode",
            ));
        }
    };

    parse_port_value(value.as_deref())
}

fn parse_port_value(value: Option<&str>) -> Result<u16, FormaError> {
    match value {
        Some(value) => value.trim_matches('"').parse().map_err(|error| {
            FormaError::new(
                FormaErrorKind::Internal,
                format!("PORT must be a valid 16-bit port number: {error}"),
            )
        }),
        None => Ok(3310),
    }
}

/// Database name used for authentication data.
pub const AUTH_DB_KEY: &'static str = "auth_services";

/// Collection name for account records.
pub const AUTH_DB_ACCOUNT_COLLECTION_KEY: &'static str = "account";
/// Collection name for user records.
pub const AUTH_DB_USER_COLLECTION_KEY: &'static str = "user";
/// Collection name for session records.
pub const AUTH_DB_SESSION_COLLECTION_KEY: &'static str = "session";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn port_defaults_only_when_unset() {
        assert_eq!(parse_port_value(None).unwrap(), 3310);
        assert_eq!(parse_port_value(Some("3000")).unwrap(), 3000);
        assert_eq!(parse_port_value(Some("\"3000\"")).unwrap(), 3000);
        assert!(parse_port_value(Some("invalid")).is_err());
        assert!(parse_port_value(Some("70000")).is_err());
    }
}
