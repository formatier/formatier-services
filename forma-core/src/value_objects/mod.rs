//! Runtime environment configuration and service-level constants.
use std::{env, sync::LazyLock};

use crate::domain::entities::{FormaError, FormaErrorConverter, FormaErrorKind};

/// Identifies the runtime environment selected for this build.
pub enum RuntimeEnvironment {
    /// Development or debug runtime.
    Dev,
    /// Production or optimized runtime.
    Prod,
}

/// Reads and trims a string-valued environment variable.
///
/// # Arguments
/// * `key` - The environment variable name.
///
/// # Returns
/// The variable value with surrounding double-quote characters removed.
///
/// # Errors
/// Returns a [`FormaError`] if the variable is not set or cannot be read.
pub fn get_env(key: &str) -> Result<String, FormaError> {
    env::var(key)
        .map(|v| v.trim_matches('"').into())
        .map_forma_err(FormaErrorKind::Internal, "cannot load env")
}

#[cfg(debug_assertions)]
/// Runtime environment selected for debug builds.
pub const RUNTIME_ENV: RuntimeEnvironment = RuntimeEnvironment::Dev;

#[cfg(not(debug_assertions))]
/// Runtime environment selected for optimized builds.
pub const RUNTIME_ENV: RuntimeEnvironment = RuntimeEnvironment::Prod;

/// Service port loaded from `PORT`, defaulting to `3310`.
pub static PORT: LazyLock<u16> =
    LazyLock::new(|| get_env("PORT").unwrap_or("3310".into()).parse().unwrap());

/// Database name used for authentication data.
pub const AUTH_DB_KEY: &'static str = "auth_services";

/// Collection name for account records.
pub const AUTH_DB_ACCOUNT_COLLECTION_KEY: &'static str = "account";
/// Collection name for user records.
pub const AUTH_DB_USER_COLLECTION_KEY: &'static str = "user";
/// Collection name for session records.
pub const AUTH_DB_SESSION_COLLECTION_KEY: &'static str = "session";
