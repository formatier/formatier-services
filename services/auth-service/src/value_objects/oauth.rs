//! OAuth endpoint and client configuration.
use std::sync::LazyLock;

use forma_core::{use_env, value_objects::get_env};

/// Google's OAuth token-exchange endpoint.
pub const GOOGLE_OAUTH_TOKEN_EXCHANGE_URL: &str = "https://oauth2.googleapis.com/token";
/// Google's OpenID Connect user-information endpoint.
pub const GOOGLE_OAUTH_USER_INFO_URL: &str = "https://openidconnect.googleapis.com/v1/userinfo";

/// Google OAuth client identifier loaded from the environment.
pub static GOOGLE_OAUTH_CLIENT_ID: LazyLock<String> =
    LazyLock::new(|| get_env("GOOGLE_OAUTH_CLIENT_ID").unwrap());
/// Google OAuth client secret loaded from the environment.
pub static GOOGLE_OAUTH_CLIENT_SECRET: LazyLock<String> =
    LazyLock::new(|| get_env("GOOGLE_OAUTH_CLIENT_SECRET").unwrap());

// Selects the OAuth redirect URI based on the current build profile.
use_env!(
    GOOGLE_OAUTH_REDIRECT_URI,
    "http://localhost:3000/auth/callback/google",
    "https://www.formatier.com/auth/callback/google"
);
