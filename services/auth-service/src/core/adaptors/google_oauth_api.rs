//! Google OAuth HTTP client implementation.
//!
//! This adapter exchanges authorization codes for tokens and retrieves
//! identity information for authenticated users.
use async_trait::async_trait;
use axum::http::{HeaderMap, HeaderValue};
use forma_core::domain::entities::{
    FormaError, FormaErrorApp, FormaErrorConverter, FormaErrorExternalService, FormaErrorOAuth,
};

use crate::{
    domain::{
        blueprints::OAuthBlueprint,
        entities::{self, OAuthToken, OAuthUserInfo},
    },
    value_objects,
};

/// HTTP adapter for Google's OAuth and OpenID Connect endpoints.
pub struct GoogleOAuthApi {
    client: reqwest::Client,
}

impl GoogleOAuthApi {
    /// Creates an adapter with a new Reqwest client.
    ///
    /// # Returns
    /// A Google OAuth API adapter.
    pub fn new() -> Self {
        let client = reqwest::Client::new();
        Self { client }
    }
}

#[async_trait]
impl OAuthBlueprint for GoogleOAuthApi {
    /// Exchanges an authorization code for an OAuth token.
    ///
    /// # Arguments
    /// * `code` - The authorization code issued by Google.
    ///
    /// # Returns
    /// The parsed OAuth token.
    ///
    /// # Errors
    /// Returns a [`FormaError`] if URL construction, the HTTP request, or
    /// response parsing fails.
    async fn exchange_code(&self, code: &str) -> Result<OAuthToken, FormaError> {
        let mut google_api_url = url::Url::parse(value_objects::GOOGLE_OAUTH_TOKEN_EXCHANGE_URL)
            .map_forma_err(
                FormaErrorApp::InvalidType,
                "cannot parse google oauth token exchange url",
            )?;

        println!("{}", value_objects::GOOGLE_OAUTH_CLIENT_SECRET.as_str());

        let api_url = google_api_url
            .query_pairs_mut()
            .append_pair("client_id", value_objects::GOOGLE_OAUTH_CLIENT_ID.as_str())
            .append_pair(
                "client_secret",
                value_objects::GOOGLE_OAUTH_CLIENT_SECRET.as_str(),
            )
            .append_pair("code", &code)
            .append_pair("grant_type", "authorization_code")
            .append_pair("redirect_uri", value_objects::GOOGLE_OAUTH_REDIRECT_URI)
            .finish();

        let res = self
            .client
            .post(api_url.as_str())
            .send()
            .await
            .map_err(|_| {
                FormaError::new(FormaErrorOAuth::Unauthorized, "cannot exchange oauth token")
            })?
            .json::<entities::GoogleOAuthTokenExchange>()
            .await
            .map_forma_err(
                FormaErrorExternalService::ConnectionError,
                "cannot parse oauth token",
            )?;

        Ok(res.into())
    }

    /// Retrieves the Google user profile authorized by an OAuth token.
    ///
    /// # Arguments
    /// * `token` - The OAuth token used to authorize the request.
    ///
    /// # Returns
    /// The parsed user identity information.
    ///
    /// # Errors
    /// Returns a [`FormaError`] if the HTTP request or response parsing fails.
    async fn get_authorized_user(&self, token: &OAuthToken) -> Result<OAuthUserInfo, FormaError> {
        let mut headers = HeaderMap::new();
        headers.insert(
            "Authorization",
            HeaderValue::from_str(&format!("Bearer {}", token.access_token)).unwrap(),
        );
        let res = self
            .client
            .get(value_objects::GOOGLE_OAUTH_USER_INFO_URL)
            .headers(headers)
            .send()
            .await
            .map_err(|_| {
                FormaError::new(FormaErrorOAuth::Unauthorized, "cannot exchange oauth token")
            })?
            .json::<entities::GoogleOpenIDUserInfo>()
            .await
            .map_forma_err(
                FormaErrorApp::InvalidType,
                "cannot parse google oauth token exchange url",
            )?;

        Ok(res.into())
    }
}
