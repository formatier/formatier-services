//! Versioned request and response payloads for authentication endpoints.
use serde::{Deserialize, Serialize};

use super::{Account, User};

use super::Provider;

#[derive(Serialize, Deserialize)]
/// Request payload for validating an access token.
pub struct AppAuthenticateV1Request {
    /// Access token to authenticate.
    pub access_token: Box<str>,
}

#[derive(Serialize, Deserialize)]
/// Response payload containing the authenticated account and user identifiers.
pub struct AppAuthenticateV1Reply {
    /// Identifier of the authenticated account.
    pub account_id: Box<str>,
    /// Identifier of the authenticated user.
    pub user_id: Box<str>,
}

#[derive(Serialize, Deserialize)]
/// Request payload for signing in with an OAuth provider.
pub struct SigninByOAuthProviderV1Request {
    /// OAuth provider that issued the authorization code.
    pub provider: Provider,
    /// Authorization code returned by the provider.
    pub auth_code: Box<str>,
}

#[derive(Serialize, Deserialize)]
/// Response payload containing tokens returned by OAuth sign-in.
pub struct SigninByOAuthProviderV1Reply {
    /// Access or initialization token.
    pub access_token: Box<str>,
    /// Refresh token, omitted when account initialization is required.
    pub refresh_token: Option<Box<str>>,
}

#[derive(Serialize, Deserialize)]
/// Request payload for signing in with email and password.
pub struct SigninByCredentialsV1Request {
    /// Account email address.
    pub email: Box<str>,
    /// Account password.
    pub password: Box<str>,
}

#[derive(Serialize, Deserialize)]
/// Request payload for passwordless sign-in.
pub struct SigninByPasswordlessV1Request {
    /// Email address to use for passwordless sign-in.
    pub email: Box<str>,
}

#[derive(Serialize, Deserialize)]
/// Response payload for sign-in, including account and optional user data.
pub struct SigninV1Reply {
    /// Authenticated account.
    pub account: Account,
    /// Associated user, if the account has been initialized.
    pub user: Option<User>,
    /// Token used to authenticate subsequent requests.
    pub auth_token: Box<str>,
}

#[derive(Serialize, Deserialize)]
/// Response payload containing account and user profile data.
pub struct AccountV1Reply {
    /// Account profile.
    pub account: Account,
    /// User profile.
    pub user: User,
}
