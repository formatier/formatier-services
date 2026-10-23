//! Axum request handlers for authentication endpoints.
use std::sync::Arc;

use axum::{
    Json,
    body::Body,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use axum_extra::{TypedHeader, headers::UserAgent};

use crate::{
    core::use_cases::UseCase,
    domain::entities::{SigninByOAuthProviderV1Reply, SigninByOAuthProviderV1Request},
};

/// Pairs a status code with a serializable JSON response body.
struct JsonResponse<T>(StatusCode, T);

impl<T> IntoResponse for JsonResponse<T>
where
    T: Serialize,
{
    /// Converts the status and payload into an HTTP response.
    ///
    /// # Returns
    /// An HTTP response containing the serialized payload.
    fn into_response(self) -> Response {
        let body_string = serde_json::to_string(&self.1).unwrap_or("".into());
        let body = Body::from(body_string);

        let mut res = Response::new(body);
        *res.status_mut() = self.0;
        res
    }
}

/// Holds the authentication use case invoked by HTTP handlers.
pub struct AxumDriver {
    use_case: UseCase,
}

/// Handles OAuth sign-in requests.
///
/// # Arguments
/// * `state` - Shared authentication driver state.
/// * `req` - OAuth provider and authorization code.
/// * `user_agent` - User-agent header from the incoming request.
///
/// # Returns
/// An HTTP response containing tokens on success or an error payload on
/// failure.
pub async fn signin_by_oauth_provider(
    State(state): State<Arc<AxumDriver>>,
    req: Json<SigninByOAuthProviderV1Request>,
    TypedHeader(user_agent): TypedHeader<UserAgent>,
) -> impl IntoResponse {
    match state
        .use_case
        .signin_by_oauth_provider(req.provider.into(), &req.auth_code, user_agent.as_str())
        .await
    {
        Ok((access_token, refresh_token)) => JsonResponse(
            StatusCode::OK,
            SigninByOAuthProviderV1Reply {
                access_token: access_token.into(),
                refresh_token: refresh_token.map(|v| v.into()),
            },
        )
        .into_response(),
        Err(err) => JsonResponse(err.get_status(), err).into_response(),
    }
}

/// Handles middleware authentication and token refresh.
///
/// # Arguments
/// * `state` - Shared authentication driver state.
///
/// # Returns
/// The response produced by the authentication middleware workflow.
pub async fn middleware_auth(State(state): State<Arc<AxumDriver>>) -> impl IntoResponse {
    let token = state.use_case.decode_and_refresh_token(token, typ);
}

impl AxumDriver {
    /// Creates an HTTP driver backed by the supplied use case.
    ///
    /// # Arguments
    /// * `use_case` - Authentication workflow implementation.
    ///
    /// # Returns
    /// An Axum driver.
    pub fn new(use_case: UseCase) -> Self {
        Self { use_case }
    }
}
