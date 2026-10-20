use std::{collections::HashMap, fmt::Display};

use forma_core::{
    domain::entities::{BsonTime, ChronoTime, Model, ModelPairs, Timestamp},
    inline_mod,
};
use forma_proc_macro::serde_migrator;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};

inline_mod!(oauth);
inline_mod!(dto);

#[derive(EnumString, PartialEq, Display)]
#[strum(serialize_all = "snake_case")]
pub enum TokenType {
    Access,
    Refresh,
    Initialization,
}

#[derive(Serialize, Deserialize, Copy, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum Provider {
    Google,
    Github,
}

impl Display for Provider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Provider::Google => write!(f, "google"),
            Provider::Github => write!(f, "github"),
        }
    }
}

#[derive(Serialize, Deserialize, Default)]
pub struct AccountV1 {
    pub email: String,
    pub user_name: Option<String>,
    pub password: Option<String>,

    pub providers: HashMap<Provider, AccountV1Provider>,
}

#[derive(Serialize, Deserialize)]
pub struct AccountV1Provider {
    pub provider_id: String,
}

#[derive(Serialize, Deserialize)]
pub struct UserV1 {
    pub display_name: String,
    pub name: Option<String>,
    pub middle_name: Option<String>,
    pub family_name: Option<String>,
    pub description: Option<String>,
    pub profile_bucket_id: Option<String>,
    pub formatier_verified: bool,

    pub badge: Box<[BadgeV1]>,
}

#[derive(Serialize, Deserialize)]
pub struct BadgeV1 {
    pub name: String,
    pub description: Option<String>,
    pub category: CategoryV1,
}

#[derive(Serialize, Deserialize)]
pub enum CategoryV1 {
    Achievement,
    Event,
    Challenge,
    Competition,
    Reward,
    FormatierVerified,
}

#[derive(Serialize, Deserialize)]
pub struct InitializationTokenClaims {
    #[serde(rename = "sub")]
    pub account_id: String,
}

#[derive(Serialize, Deserialize)]
pub struct AccessTokenClaims {
    #[serde(rename = "jti")]
    pub session_id: String,
    #[serde(rename = "sub")]
    pub account_id: String,

    pub email: String,
    pub user_id: String,

    pub scope: Box<[String]>,
}

#[derive(Serialize, Deserialize)]
pub struct RefreshTokenClaims {
    #[serde(rename = "jti")]
    pub session_id: String,
}

#[derive(Serialize, Deserialize)]
pub struct TokenClaims<T> {
    #[serde(rename = "iss")]
    pub issuer: Issuer,
    #[serde(rename = "aud")]
    pub audience: Box<[Audience]>,

    #[serde(rename = "exp")]
    pub expiration_time: Timestamp<ChronoTime>,
    #[serde(rename = "iat")]
    pub issue_at: Timestamp<ChronoTime>,

    pub claims: T,
}

#[derive(Serialize, Deserialize)]
pub enum Audience {
    #[serde(rename = "formatier-api")]
    FormatierAPI,
    #[serde(rename = "formatier-mail-server")]
    FormatierMailServer,
    #[serde(rename = "formatine-server")]
    FormatineServer,
    #[serde(rename = "formatier-bucket-server")]
    FormatierBucketServer,
}

#[derive(Serialize, Deserialize)]
pub enum Issuer {
    #[serde(rename = "formatier")]
    Formatier,
}

#[derive(Serialize, Deserialize)]
pub struct SessionV1 {
    pub os: String,
    pub os_version: String,
    pub browser: String,
    pub browser_version: String,
    pub vendor: String,
}

impl From<woothee::parser::WootheeResult<'_>> for SessionV1 {
    fn from(value: woothee::parser::WootheeResult) -> Self {
        Self {
            os: value.os.into(),
            os_version: value.os_version.into(),
            browser: value.browser_type.into(),
            browser_version: value.version.into(),
            vendor: value.vendor.into(),
        }
    }
}

serde_migrator! {
    #[derive(Serialize, Deserialize)]
    pub Account
    "v1": AccountV1;
}

serde_migrator! {
    #[derive(Serialize, Deserialize)]
    pub User
    "v1": UserV1;
}

serde_migrator! {
    #[derive(Serialize, Deserialize)]
    pub Session
    "v1": SessionV1;
}

#[derive(Serialize, Deserialize, Default)]
pub struct AccountMetadata {
    pub create_at: Timestamp<BsonTime>,
    pub update_at: Timestamp<BsonTime>,

    pub user_id: Option<ObjectId>,
}

#[derive(Serialize, Deserialize, Default)]
pub struct UserMetadata {
    pub create_at: Timestamp<BsonTime>,
    pub update_at: Timestamp<BsonTime>,
}

#[derive(Serialize, Deserialize, Default)]
pub struct SessionMetadata {
    pub create_at: Timestamp<BsonTime>,
    pub update_at: Timestamp<BsonTime>,

    pub account_id: ObjectId,
}

pub type AccountModel<S> = Model<Account, AccountMetadata, S>;
pub type UserModel<S> = Model<User, UserMetadata, S>;
pub type SessionModel<S> = Model<Session, SessionMetadata, S>;

pub type AccountModelPairs = ModelPairs<Account, AccountMetadata>;
pub type UserModelPairs = ModelPairs<User, UserMetadata>;
pub type SessionModelPairs = ModelPairs<Session, SessionMetadata>;
