use axum::{
    Json,
    extract::{Request, State},
    middleware::Next,
    response::{IntoResponse, Response},
};
use axum_extra::extract::CookieJar;
use forma_core::domain::entities::{FormaError, FormaErrorAuth, FormaErrorKind};
use forma_schema::auth_service::AppAuthenticateV1Reply;
use reqwest::StatusCode;
use std::sync::Arc;

use crate::domain::{blueprints::ServiceBlueprint, entities::AuthStrategy};

pub struct AxumDriver {
    service_adapter: Arc<dyn ServiceBlueprint + Send + Sync>,
}

impl AxumDriver {
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

pub struct AuthMiddlewareState {
    pub state: Arc<AxumDriver>,
    pub auth_strategy: AuthStrategy,
}

pub async fn auth_middleware(
    State(state): State<Arc<AuthMiddlewareState>>,
    mut req: Request,
    next: Next,
) -> Response {
    let access_token: Option<&str> = {
        let header = req.headers().get("Authentication");
        let access_token = header
            .map(|v| v.to_str().ok())
            .flatten()
            .map(|v| {
                let splitted: Vec<&str> = v.split(" ").collect();
                if splitted.len() != 2 {
                    return None;
                } else {
                    let auth_type = splitted.get(0).unwrap();
                    if *auth_type == "Barea" {
                        let access_token = splitted.get(1).unwrap();
                        return Some(*access_token);
                    } else {
                        return None;
                    }
                }
            })
            .flatten();

        access_token
    };
    if let Some(access_token) = access_token {
        let repl = state
            .state
            .service_adapter
            .app_authenticate_v1(access_token)
            .await;

        match repl {
            Ok(AppAuthenticateV1Reply {
                account_id,
                user_id,
            }) => {
                let headers = req.headers_mut();
                let account_id = account_id.parse();
                let user_id = user_id.parse();

                if account_id.is_err() || user_id.is_err() {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(FormaError::new(
                            FormaErrorKind::Unhandled,
                            "cannot parse account_id or user_id to HeaderValue in api-gateway",
                        )),
                    )
                        .into_response();
                }

                headers.insert("X-Account-Id", account_id.unwrap());
                headers.insert("X-User-Id", user_id.unwrap());
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
                        "tokens are not exits",
                    )),
                )
                    .into_response();
            }
            _ => {}
        }
    }

    next.run(req).await
}
