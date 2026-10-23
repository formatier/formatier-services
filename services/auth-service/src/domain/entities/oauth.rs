//! Data transfer types for OAuth provider responses and identities.
use serde::Deserialize;

#[derive(Deserialize)]
/// Google's token-exchange response fields used by this service.
pub struct GoogleOAuthTokenExchange {
    /// Access token returned by Google.
    pub access_token: String,
    /// Token lifetime in seconds.
    pub expires_in: u64,
}

/// Converts Google's token response to the provider-neutral token type.
impl Into<OAuthToken> for GoogleOAuthTokenExchange {
    /// Builds an [`OAuthToken`] from the response fields.
    ///
    /// # Arguments
    /// * `self` - Google's token-exchange response.
    ///
    /// # Returns
    /// The provider-neutral token.
    fn into(self) -> OAuthToken {
        OAuthToken {
            access_token: self.access_token,
            expires_in: self.expires_in,
        }
    }
}

#[derive(Deserialize)]
/// Google's OpenID Connect user profile fields used by this service.
pub struct GoogleOpenIDUserInfo {
    /// Stable subject identifier assigned by Google.
    pub sub: String,
    /// User's given name.
    pub given_name: String,
    /// Optional user family name.
    pub family_name: Option<String>,
    /// User's email address.
    pub email: String,
    /// Optional profile image URL.
    pub picture: Option<String>,
}

/// Converts a Google identity response to the provider-neutral profile type.
impl Into<OAuthUserInfo> for GoogleOpenIDUserInfo {
    /// Maps Google's response fields to the common OAuth identity.
    ///
    /// # Arguments
    /// * `self` - Google's OpenID user information.
    ///
    /// # Returns
    /// A provider-neutral OAuth user profile.
    fn into(self) -> OAuthUserInfo {
        OAuthUserInfo {
            provider_id: self.sub,
            email: self.email,
            username: None,
            avatar_url: self.picture,
            display_name: Some(
                format!(
                    "{} {}",
                    self.given_name,
                    self.family_name.unwrap_or("".into())
                )
                .trim_matches(' ')
                .into(),
            ),
            bio: None,
            website: None,
        }
    }
}

#[derive(Deserialize)]
/// Provider-neutral OAuth access token data.
pub struct OAuthToken {
    /// Access token used to authorize provider API requests.
    pub access_token: String,
    /// Token lifetime in seconds.
    pub expires_in: u64,
}

/// Provider-neutral user identity returned after OAuth authorization.
pub struct OAuthUserInfo {
    /// Stable identifier assigned by the provider.
    pub provider_id: String,
    /// User's email address.
    pub email: String,
    /// Optional provider username.
    pub username: Option<String>,
    /// Optional profile image URL.
    pub avatar_url: Option<String>,

    /// Optional display name.
    pub display_name: Option<String>,

    /// Optional biography.
    pub bio: Option<String>,
    /// Optional personal website URL.
    pub website: Option<String>,
}
