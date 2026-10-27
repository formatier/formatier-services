//! Axum middleware and HTTP handlers for the API gateway.
use auth_service::domain::entities::AppAuthenticateV1Reply;
use axum::{
    Json,
    extract::{Request, State},
    http::{HeaderMap, HeaderValue, header::AUTHORIZATION},
    middleware::Next,
    response::{IntoResponse, Response},
};
use axum_extra::extract::CookieJar;
use forma_core::domain::entities::{FormaError, FormaErrorAuth, FormaErrorKind};
use reqwest::StatusCode;
use std::sync::Arc;

use crate::domain::{blueprints::ServiceBlueprint, entities::AuthStrategy};

// TODO: Move to utils
fn extract_access_token(headers: &HeaderMap) -> Option<&str> {
    let header = headers
        .get(AUTHORIZATION)
        .or_else(|| headers.get("Authentication"))?;
    let mut parts = header.to_str().ok()?.split_ascii_whitespace();
    let scheme = parts.next()?;
    let token = parts.next()?;

    (scheme.eq_ignore_ascii_case("Bearer") && parts.next().is_none()).then_some(token)
}

fn remove_untrusted_identity_headers(headers: &mut HeaderMap) {
    headers.remove("X-Account-Id");
    headers.remove("X-User-Id");
}

/// Holds adapters shared by API gateway handlers.
pub struct AxumDriver {
    service_adapter: Arc<dyn ServiceBlueprint + Send + Sync>,
}

impl AxumDriver {
    /// Creates a driver using the supplied service adapter.
    pub fn new(service_adapter: Arc<dyn ServiceBlueprint + Send + Sync>) -> Self {
        Self { service_adapter }
    }
}

pub async fn get_cookies(
    State(_state): State<Arc<AxumDriver>>,
    jar: CookieJar,
) -> Json<Vec<String>> {
    let mut cookies_name = Vec::new();
    for cookie in jar.iter() {
        cookies_name.push(cookie.name().to_string());
    }

    Json(cookies_name)
}

/// Holds the driver and authentication policy used by the middleware.
pub struct AuthMiddlewareState {
    /// API gateway state used to contact the authentication service.
    pub state: Arc<AxumDriver>,
    /// Policy controlling whether a missing token is allowed.
    pub auth_strategy: AuthStrategy,
}

/// Authenticates a request when its configured policy requires a token.
/// A validated identity is forwarded to downstream handlers through request headers.
pub async fn auth_middleware(
    State(state): State<Arc<AuthMiddlewareState>>,
    mut req: Request,
    next: Next,
) -> Response {
    let access_token = extract_access_token(req.headers()).map(str::to_owned);
    remove_untrusted_identity_headers(req.headers_mut());
    if let Some(access_token) = access_token {
        let repl = state
            .state
            .service_adapter
            .app_authenticate_v1(&access_token)
            .await;

        match repl {
            Ok(AppAuthenticateV1Reply {
                account_id,
                user_id,
            }) => {
                let headers = req.headers_mut();
                let account_id = match HeaderValue::from_str(&account_id) {
                    Ok(value) => value,
                    Err(_) => {
                        return (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(FormaError::new(
                                FormaErrorKind::Unhandled,
                                "Could not parse account_id as a HeaderValue in the API gateway",
                            )),
                        )
                            .into_response();
                    }
                };
                let user_id = match HeaderValue::from_str(&user_id) {
                    Ok(value) => value,
                    Err(_) => {
                        return (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(FormaError::new(
                                FormaErrorKind::Unhandled,
                                "Could not parse user_id as a HeaderValue in the API gateway",
                            )),
                        )
                            .into_response();
                    }
                };

                headers.insert("X-Account-Id", account_id);
                headers.insert("X-User-Id", user_id);
            }
            Err(err) => {
                return (err.get_status(), Json(err)).into_response();
            }
        }
    } else {
        match state.auth_strategy {
            AuthStrategy::Token => {
                return (
                    StatusCode::UNAUTHORIZED,
                    Json(FormaError::new(
                        FormaErrorAuth::TokenInvalid,
                        "No valid token was provided",
                    )),
                )
                    .into_response();
            }
            _ => {}
        }
    }

    next.run(req).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_standard_bearer_authorization() {
        let headers = HeaderMap::from_iter([(
            AUTHORIZATION,
            HeaderValue::from_static("Bearer access-token"),
        )]);
        assert_eq!(extract_access_token(&headers), Some("access-token"));
    }

    #[test]
    fn accepts_legacy_authentication_header() {
        let headers = HeaderMap::from_iter([(
            axum::http::HeaderName::from_static("authentication"),
            HeaderValue::from_static("Bearer access-token"),
        )]);
        assert_eq!(extract_access_token(&headers), Some("access-token"));
    }

    #[test]
    fn rejects_invalid_bearer_headers() {
        for value in ["Barea token", "Basic token", "Bearer", "Bearer one two"] {
            let headers =
                HeaderMap::from_iter([(AUTHORIZATION, HeaderValue::from_str(value).unwrap())]);
            assert_eq!(extract_access_token(&headers), None);
        }
    }

    #[test]
    fn removes_client_supplied_identity_headers() {
        let mut headers = HeaderMap::new();
        headers.insert("X-Account-Id", HeaderValue::from_static("untrusted"));
        headers.insert("X-User-Id", HeaderValue::from_static("untrusted"));

        remove_untrusted_identity_headers(&mut headers);

        assert!(!headers.contains_key("X-Account-Id"));
        assert!(!headers.contains_key("X-User-Id"));
    }
}
