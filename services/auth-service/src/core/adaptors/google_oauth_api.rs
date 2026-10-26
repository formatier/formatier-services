//! Google OAuth HTTP client implementation.
//! Exchanges Google OAuth codes and retrieves user identity data.
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
    pub fn new() -> Self {
        let client = reqwest::Client::new();
        Self { client }
    }
}

#[async_trait]
impl OAuthBlueprint for GoogleOAuthApi {
    /// Exchanges an authorization code for an OAuth token.
    async fn exchange_code(&self, code: &str) -> Result<OAuthToken, FormaError> {
        let mut google_api_url = url::Url::parse(value_objects::GOOGLE_OAUTH_TOKEN_EXCHANGE_URL)
            .map_forma_err(
                FormaErrorApp::InvalidType,
                "Could not parse the Google OAuth token-exchange URL",
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
                FormaError::new(
                    FormaErrorOAuth::Unauthorized,
                    "Could not exchange the OAuth token",
                )
            })?
            .json::<entities::GoogleOAuthTokenExchange>()
            .await
            .map_forma_err(
                FormaErrorExternalService::ConnectionError,
                "Could not parse the OAuth token response",
            )?;

        Ok(res.into())
    }

    /// Retrieves the Google user profile authorized by an OAuth token.
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
                FormaError::new(
                    FormaErrorOAuth::Unauthorized,
                    "Could not retrieve the Google user profile",
                )
            })?
            .json::<entities::GoogleOpenIDUserInfo>()
            .await
            .map_forma_err(
                FormaErrorApp::InvalidType,
                "Could not parse the Google user information response",
            )?;

        Ok(res.into())
    }
}
