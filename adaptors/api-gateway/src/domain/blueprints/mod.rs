//! Contracts for communicating with upstream services.
use auth_service::domain::entities::{AppAuthenticateV1Reply, AppAuthenticateV1Request};
use forma_core::{
    domain::entities::{
        FormaError, FormaErrorApp, FormaErrorConverter, FormaErrorExt, FormaErrorExternalService,
        FormaErrorReqwestExt, Request, ResultReply,
    },
    inline_mod,
};
use url::Url;

/// Deserializes a service response body into its success or error result.
///
/// # Arguments
/// * `res` - HTTP response returned by the upstream service.
///
/// # Returns
/// The decoded success value.
///
/// # Errors
/// Returns a [`FormaError`] if the response body cannot be decoded or contains
/// an error reply.
async fn into_result<T>(res: reqwest::Response) -> Result<T, FormaError>
where
    T: for<'de> serde::Deserialize<'de>,
{
    res.json::<ResultReply<_>>()
        .await
        .map_forma_err(
            FormaErrorApp::InvalidType,
            "cannot parse response to ResultReply",
        )?
        .into()
}

#[async_trait::async_trait]
/// Defines common HTTP operations for the authentication service.
pub trait AuthServiceBlueprint {
    /// Returns the base URL of the authentication service.
    ///
    /// # Returns
    /// The service URL.
    fn get_url(&self) -> Url;

    /// Returns the client used to make authentication-service requests.
    ///
    /// # Returns
    /// A shared reference to the HTTP client.
    fn get_client(&self) -> &reqwest::Client;

    /// Validates an access token through the authentication service.
    ///
    /// # Arguments
    /// * `access_token` - Token to validate.
    ///
    /// # Returns
    /// The authenticated account and user identifiers.
    ///
    /// # Errors
    /// Returns a [`FormaError`] if the upstream request or response fails.
    async fn app_authenticate_v1(
        &self,
        access_token: &str,
    ) -> Result<AppAuthenticateV1Reply, FormaError> {
        let client = self.get_client();
        let mut url = self.get_url();
        url.set_path("/auth/middleware");

        let res = client
            .post(url)
            .body(
                Request {
                    data: AppAuthenticateV1Request {
                        access_token: access_token.into(),
                    },
                }
                .into_string()?,
            )
            .send()
            .await
            .map_reqwest_forma_err(
                FormaErrorExternalService::ConnectionError,
                "cannot connect to auth_service",
            )
            .with_service_breaker_request()?;

        into_result(res).await
    }
}

/// Combines authentication-service operations with the gateway service contract.
pub trait ServiceBlueprint: AuthServiceBlueprint {}
