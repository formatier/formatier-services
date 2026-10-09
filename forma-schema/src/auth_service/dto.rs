use serde::{Deserialize, Serialize};

use crate::auth_service::{AccountV1, UserV1};

use super::ProviderV1;

#[derive(Serialize, Deserialize)]
pub struct AppAuthenticateV1Request {
    pub access_token: Box<str>,
}

#[derive(Serialize, Deserialize)]
pub struct AppAuthenticateV1Reply {
    pub account_id: Box<str>,
    pub user_id: Box<str>,
}

#[derive(Serialize, Deserialize)]
pub struct SigninByOAuthProviderV1Request {
    pub provider: ProviderV1,
    pub auth_code: Box<str>,
    pub user_agent: Box<str>,
}

#[derive(Serialize, Deserialize)]
pub struct SigninByOAuthProviderV1Reply {
    pub access_token: Box<str>,
    pub refresh_token: Option<Box<str>>,
}

#[derive(Serialize, Deserialize)]
pub struct SigninByCredentialsV1Request {
    pub email: Box<str>,
    pub password: Box<str>,
}

#[derive(Serialize, Deserialize)]
pub struct SigninByPasswordlessV1Request {
    pub email: Box<str>,
}

#[derive(Serialize, Deserialize)]
pub struct SigninV1Reply {
    pub account: AccountV1,
    pub user: Option<UserV1>,
    pub auth_token: Box<str>,
}
