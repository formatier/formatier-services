//! Database connection configuration loaded from the process environment.
use std::sync::LazyLock;

use forma_core::value_objects::get_env;

/// Database connection URL loaded from `DATABASE_URL`.
pub static DATABASE_URL: LazyLock<String> = LazyLock::new(|| get_env("DATABASE_URL").unwrap());

/// Cache connection URL loaded from `CACHE_URL`.
pub static CACHE_URL: LazyLock<String> = LazyLock::new(|| get_env("CACHE_URL").unwrap());
