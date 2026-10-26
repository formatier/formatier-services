//! Database connection configuration loaded from the process environment.
use std::sync::LazyLock;

use forma_core::value_objects::get_env;

/// Database connection URL loaded from `DATABASE_URL`.
pub static DATABASE_URL: LazyLock<String> = LazyLock::new(|| get_env("DATABASE_URL").unwrap());

/// Cache connection URL loaded from `CACHE_URL`.
pub static CACHE_URL: LazyLock<String> = LazyLock::new(|| get_env("CACHE_URL").unwrap());

pub const ACCOUNT_TABLE: &str = "account";
pub const PROVIDER_TABLE: &str = "provider";
pub const SESSION_TABLE: &str = "session";
pub const USER_TABLE: &str = "user";