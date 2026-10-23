//! OAuth provider operations required by the authentication domain.
use async_trait::async_trait;
use forma_core::domain::entities::FormaError;

use crate::domain::entities::{OAuthToken, OAuthUserInfo};

#[async_trait]
/// Defines the OAuth operations used by authentication workflows.
pub trait OAuthBlueprint {
    /// Exchanges an authorization code for an OAuth token.
    ///
    /// # Arguments
    /// * `code` - Authorization code returned by the provider.
    ///
    /// # Returns
    /// The provider-issued OAuth token.
    ///
    /// # Errors
    /// Returns a [`FormaError`] if the provider request or response handling
    /// fails.
    async fn exchange_code(&self, code: &str) -> Result<OAuthToken, FormaError>;

    /// Retrieves user identity information using an OAuth token.
    ///
    /// # Arguments
    /// * `token` - Token authorizing the identity request.
    ///
    /// # Returns
    /// The authorized user's profile information.
    ///
    /// # Errors
    /// Returns a [`FormaError`] if the provider request or response handling
    /// fails.
    async fn get_authorized_user(&self, token: &OAuthToken) -> Result<OAuthUserInfo, FormaError>;
}
