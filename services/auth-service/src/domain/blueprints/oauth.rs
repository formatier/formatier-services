//! OAuth provider operations required by the authentication domain.
use async_trait::async_trait;
use forma_core::domain::entities::FormaError;

use crate::domain::entities::{OAuthToken, OAuthUserInfo};

#[async_trait]
/// Defines the OAuth operations used by authentication workflows.
pub trait OAuthBlueprint {
    /// Exchanges an authorization code for an OAuth token.
    async fn exchange_code(&self, code: &str) -> Result<OAuthToken, FormaError>;

    /// Retrieves user identity information using an OAuth token.
    async fn get_authorized_user(&self, token: &OAuthToken) -> Result<OAuthUserInfo, FormaError>;
}
